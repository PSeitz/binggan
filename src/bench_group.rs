use std::time::Duration;

use crate::stats::compute_stats;
use crate::GLOBAL;
use peakmem_alloc::*;
use std::fmt::Debug;
use yansi::Paint;

pub struct BenchGroup<I> {
    name: String,
    inputs: Vec<(String, I)>,
    benches: Vec<Bench<I>>,
    interleave: bool,
}

type CallBench<I> = Box<dyn FnMut(&I)>;
struct Bench<I> {
    name: String,
    fun: CallBench<I>,
    results: Vec<BenchResult>,
}
impl<I> Bench<I> {
    #[inline]
    fn exec_bench(&mut self, input: &(String, I), input_idx: usize) {
        GLOBAL.reset_peak_memory();
        let start = std::time::Instant::now();
        (self.fun)(&input.1);
        let elapsed = start.elapsed();
        let mem = GLOBAL.get_peak_memory();
        let bench_result = BenchResult::new(elapsed, mem, input_idx);
        // Push the result to the results vector
        unsafe {
            let end = self.results.as_mut_ptr().add(self.results.len());
            std::ptr::write(end, bench_result);
            self.results.set_len(self.results.len() + 1);
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct BenchResult {
    pub duration_ns: u64,
    pub memory_consumption: usize,
    pub input_idx: usize,
}
impl BenchResult {
    fn new(duration: Duration, memory_consumption: usize, input_idx: usize) -> Self {
        BenchResult {
            duration_ns: duration.as_nanos() as u64, // u64 is more than enough
            memory_consumption,
            input_idx,
        }
    }
}

const NUM_RUNS: usize = 128;
impl<I> BenchGroup<I> {
    /// Run the benchmarks interleaved, i.e. one iteration of each bench after another
    /// This may lead to better results, it may also lead to worse results.
    /// It very much depends on the benches and the environment you would like to simulate.
    ///
    pub fn set_interleave(&mut self, interleave: bool) {
        self.interleave = interleave;
    }
    pub fn new(name: String) -> Self {
        BenchGroup {
            inputs: Vec::new(),
            name,
            benches: Vec::new(),
            interleave: true,
        }
    }
    /// The inputs are a vector of tuples, where the first element is the name of the input and the
    /// second element is the input itself.
    pub fn new_with_inputs<S: Into<String>>(name: String, inputs: Vec<(S, I)>) -> Self {
        BenchGroup {
            inputs: inputs
                .into_iter()
                .map(|(name, input)| (name.into(), input))
                .collect(),
            name,
            benches: Vec::new(),
            interleave: true,
        }
    }
    pub fn register<F: FnMut(&I) + 'static, S: Into<String>>(&mut self, name: S, fun: F) {
        self.benches.push(Bench {
            name: name.into(),
            fun: Box::new(fun),
            results: Vec::with_capacity(NUM_RUNS * self.inputs.len()),
        });
    }

    pub fn run(&mut self) {
        self.warm_up();
        if self.interleave {
            self.run_interleaved();
        } else {
            self.run_sequential();
        }
    }

    fn run_sequential(&mut self) {
        for (input_idx, input) in self.inputs.iter().enumerate() {
            for bench in &mut self.benches {
                for iteration in 0..NUM_RUNS {
                    alloca::with_alloca(
                        iteration, // we increase the byte offset by 1 for each iteration
                        |_memory: &mut [core::mem::MaybeUninit<u8>]| {
                            bench.exec_bench(input, input_idx);
                        },
                    );
                }
            }
        }
    }

    fn run_interleaved(&mut self) {
        // inner loop is 4 times, so we divide by 4
        for (input_idx, input) in self.inputs.iter().enumerate() {
            for iteration in 0..(NUM_RUNS / 4) {
                // We interleaved the benches to address benchmarking randomness
                //
                // This has the drawback, that one bench will affect another one.
                // TODO: We should have probably groups of benches, which are interleaved, but not
                // between groups.
                for bench in &mut self.benches {
                    // We use alloca to address memory layout randomness issues
                    // So the whole stack moves down by 1 byte for each iteration

                    // We loop 4 times on a single bench, since one bench could e.g. flush all the
                    // memory caches, which may or may not be there in a real worl environment.
                    // We want to capture both cases, hot loops and interleaved, to see how a bench performs under both
                    // conditions.
                    for _inner_iter in 0..4 {
                        alloca::with_alloca(
                            iteration, // we increase the byte offset by 1 for each iteration
                            |_memory: &mut [core::mem::MaybeUninit<u8>]| {
                                bench.exec_bench(input, input_idx);
                            },
                        );
                    }
                }
            }
        }
    }

    pub fn warm_up(&mut self) {
        for input in &self.inputs {
            for bench in &mut self.benches {
                let start = std::time::Instant::now();
                (bench.fun)(&input.1);
                let _elapsed = start.elapsed();
            }
        }
    }

    pub fn report(&mut self) {
        if self.benches.is_empty() {
            return;
        }
        if !self.name.is_empty() {
            println!("{}", self.name.black().on_red().invert().italic());
        }

        // group by input (index is stored in the BenchResult)
        // Terrible datastructure, but keeps ordering (maybe replace with BTreeMap)
        let mut results_by_input: Vec<Vec<Vec<&BenchResult>>> = vec![Vec::new(); self.inputs.len()];
        for (bench_idx, bench) in self.benches.iter().enumerate() {
            for result in &bench.results {
                let bench_results_of_one_input = &mut results_by_input[result.input_idx];
                if bench_results_of_one_input.len() <= bench_idx {
                    bench_results_of_one_input.resize(bench_idx + 1, Vec::new());
                }
                bench_results_of_one_input[bench_idx].push(result);
            }
        }

        let max_bench_name_len = self
            .benches
            .iter()
            .map(|bench| bench.name.len())
            .max()
            .unwrap_or(0)
            + 5;

        for (input_idx, bench_results) in results_by_input.iter().enumerate() {
            let input_name = self.inputs[input_idx].0.clone();
            println!("{}", input_name.black().on_yellow().invert().italic());

            for (bench_idx, results) in bench_results.iter().enumerate() {
                let bench = &self.benches[bench_idx];
                let stats = compute_stats(results).unwrap();
                println!(
                    "{:width$}: {}",
                    bench.name,
                    stats,
                    width = max_bench_name_len,
                );
            }
        }
    }
}
