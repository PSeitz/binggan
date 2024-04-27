use std::alloc::GlobalAlloc;

use crate::{
    bench::{Bench, InputWithBenchmark},
    parse_args,
    report::report_group,
    BenchInputSize, Options,
};
use peakmem_alloc::*;
use yansi::Paint;

pub(crate) type Alloc = &'static dyn PeakMemAllocTrait;
pub(crate) const NUM_RUNS: usize = 256;

/// The main struct to create benchmarks.
///
/// BenchGroup is a collection of benchmarks that are run with the same inputs.
/// It is self-contained and can be run independently.
pub struct BenchGroup<I: BenchInputSize = ()> {
    name: Option<String>,
    inputs: Vec<Input<I>>,
    benches: Vec<Bench<I>>,
    alloc: Option<Alloc>,
    cache_trasher: CacheTrasher,
    options: Options,
    /// The closure to get the size of the input.
    throughput: Option<Box<dyn Fn(&I) -> usize>>,
}

pub(crate) struct Input<I> {
    pub(crate) name: String,
    pub(crate) data: I,
}

impl BenchGroup<()> {
    /// Create a new BenchGroup with no inputs.
    pub fn new() -> Self {
        Self::new_with_inputs(vec![("", ())])
    }
}

fn matches(input: &str, filter: &Option<String>, exact: bool) -> bool {
    let Some(filter) = filter else { return true };
    if exact {
        input == filter
    } else {
        input.contains(filter)
    }
}

impl<I: BenchInputSize> BenchGroup<I> {
    /// Sets name of the group and returns the group.
    pub fn name<S: Into<String>>(mut self, name: S) -> Self {
        self.name = Some(name.into());
        self
    }
    /// The inputs are a vector of tuples, where the first element is the name of the input and the
    /// second element is the input itself.
    pub fn new_with_inputs<S: Into<String>>(inputs: Vec<(S, I)>) -> Self {
        Self::new_with_inputs_and_options(inputs, parse_args())
    }
    /// The inputs are a vector of tuples, where the first element is the name of the input and the
    /// second element is the input itself.
    pub(crate) fn new_with_inputs_and_options<S: Into<String>>(
        inputs: Vec<(S, I)>,
        mut options: Options,
    ) -> Self {
        use yansi::Condition;
        yansi::whenever(Condition::TTY_AND_COLOR);

        let mut inputs: Vec<Input<I>> = inputs
            .into_iter()
            .map(|(name, input)| Input {
                name: name.into(),
                data: input,
            })
            .collect();
        let filter_targets_input = inputs
            .iter()
            .any(|input| matches(&input.name, &options.filter, options.exact));
        // If the filter is filtering an input, we filter and remove the filter
        if filter_targets_input && options.filter.is_some() {
            inputs.retain(|input| matches(&input.name, &options.filter, options.exact));
            options.filter = None;
        }
        BenchGroup {
            inputs,
            benches: Vec::new(),
            cache_trasher: CacheTrasher::new(1024 * 1024 * 16),
            options,
            alloc: None,
            name: None,
            throughput: None,
        }
    }
    /// Set the peak mem allocator to be used for the benchmarks.
    /// This will report the peak memory consumption of the benchmarks.
    pub fn set_alloc<A: GlobalAlloc + 'static>(&mut self, alloc: &'static PeakMemAlloc<A>) {
        self.alloc = Some(alloc);
    }
    /// Enable perf profiling + report
    ///
    /// The numbers are reported with the following legend:
    /// ```bash
    /// L1dA: L1 data access
    /// L1dM: L1 data misses
    /// Br: branches
    /// BrM: missed branches
    /// ```
    /// e.g.
    /// ```bash
    /// fibonacci    Memory: 0 B       Avg: 135ns      Median: 136ns     132ns          140ns    
    ///              L1dA: 809.310     L1dM: 0.002     Br: 685.059       BrM: 0.010     
    /// baseline     Memory: 0 B       Avg: 1ns        Median: 1ns       1ns            1ns      
    ///              L1dA: 2.001       L1dM: 0.000     Br: 6.001         BrM: 0.000     
    /// ```
    pub fn enable_perf(&mut self) {
        self.options.enable_perf = true;
    }

    /// Enables throughput reporting.
    /// The passed closure should return the size of the input in bytes.
    pub fn throughput<F>(&mut self, f: F)
    where
        F: Fn(&I) -> usize + 'static,
    {
        self.throughput = Some(Box::new(f));
    }

    /// Set the name of the group.
    /// The name is printed before the benchmarks are run.
    /// It is also used to distinguish when writing the results to disk.
    pub fn set_name(&mut self, name: String) {
        self.name = Some(name);
    }

    /// Set the options to the given value.
    /// This will overwrite all current options.
    ///
    /// See the Options struct for more information.
    pub fn set_options(&mut self, options: Options) {
        self.options = options;
    }

    /// Sets the interleave option to the given value.
    pub fn set_interleave(&mut self, interleave: bool) {
        self.options.interleave = interleave;
    }

    /// Sets the filter, which is used to filter the benchmarks by name.
    /// The filter is fetched from the command line arguments.
    ///
    /// It can also match an input name.
    pub fn set_filter(&mut self, filter: Option<String>) {
        self.options.filter = filter;
    }

    /// Register a benchmark with the given name and function.
    pub fn register<F, S: Into<String>>(&mut self, name: S, fun: F)
    where
        F: for<'a> Fn(&'a I) + 'static,
    {
        let name = name.into();
        if !matches(&name, &self.options.filter, self.options.exact) {
            return;
        }
        self.benches.push(Bench::new(name, Box::new(fun)));
    }

    /// Run the benchmarks and report the results.
    pub fn run(&mut self) {
        if self.benches.is_empty() {
            return;
        }

        if let Some(name) = &self.name {
            println!("{}", name.black().on_red().invert().bold());
        }
        // Build the (group_name, benches) groups
        let mut groups: Vec<(&str, Vec<InputWithBenchmark<I>>)> = Vec::new();

        for idx in 0..self.inputs.len() {
            let input = &self.inputs[idx];
            let input_size_in_bytes = self.throughput.as_ref().map(|f| f(&input.data));

            let mut bench_bundle: Vec<InputWithBenchmark<I>> =
                Vec::with_capacity(self.benches.len());
            for bench in self.benches.iter() {
                let bundle = InputWithBenchmark::new(
                    input,
                    input_size_in_bytes,
                    bench,
                    self.options.enable_perf,
                );

                bench_bundle.push(bundle);
            }
            groups.push((&input.name, bench_bundle));
        }

        for (input_name, mut bench_bundle) in groups {
            Self::warm_up(&mut bench_bundle);
            if self.options.interleave {
                Self::run_interleaved(&mut bench_bundle, &self.alloc, &self.cache_trasher);
            } else {
                Self::run_sequential(&mut bench_bundle, &self.alloc);
            }

            let title = &input_name;
            report_group(&self.name, title, &mut bench_bundle, self.alloc.is_some());
        }
    }

    fn run_sequential(benches: &mut [InputWithBenchmark<I>], alloc: &Option<Alloc>) {
        for bench_bundle in benches {
            for iteration in 0..NUM_RUNS {
                alloca::with_alloca(
                    iteration, // we increase the byte offset by 1 for each iteration
                    |_memory: &mut [core::mem::MaybeUninit<u8>]| {
                        bench_bundle.exec_bench(alloc);
                    },
                );
            }
        }
    }

    fn run_interleaved(
        benches: &mut [InputWithBenchmark<I>],
        alloc: &Option<Alloc>,
        cache_trasher: &CacheTrasher,
    ) {
        for iteration in 0..NUM_RUNS {
            // We interleave the benches to address benchmarking randomness
            //
            // This has the drawback, that one bench will affect another one.
            let mut indices: Vec<usize> = (0..benches.len()).collect();
            shuffle(&mut indices, iteration as u64);

            std::thread::yield_now();

            cache_trasher.issue_read();

            for idx in indices {
                let bench = &mut benches[idx];
                // We use alloca to address memory layout randomness issues
                // So the whole stack moves down by 1 byte for each iteration

                // We loop multiple times on a single bench, since one bench could e.g. flush all the
                // memory caches, which may or may not be like this in a real world environment.
                // We want to capture both cases, hot loops and interleaved, to see how a bench performs under both
                // conditions.
                alloca::with_alloca(
                    iteration, // we increase the byte offset by 1 for each iteration
                    |_memory: &mut [core::mem::MaybeUninit<u8>]| {
                        bench.exec_bench(alloc);
                    },
                );
            }
        }
    }

    fn warm_up(benches: &mut [InputWithBenchmark<I>]) {
        // Measure and print the time it took
        for input_and_bench in benches {
            input_and_bench.sample_and_set_iter(input_and_bench.input);
        }
    }
}

/// Performs a dummy reads from memory to spoil given amount of CPU cache
///
/// Uses cache aligned data arrays to perform minimum amount of reads possible to spoil the cache
struct CacheTrasher {
    cache_lines: Vec<CacheLine>,
}
impl Default for CacheTrasher {
    fn default() -> Self {
        Self::new(1024 * 1024 * 16)
    }
}

impl CacheTrasher {
    fn new(bytes: usize) -> Self {
        let n = bytes / std::mem::size_of::<CacheLine>();
        let cache_lines = vec![CacheLine::default(); n];
        Self { cache_lines }
    }

    fn issue_read(&self) {
        for line in &self.cache_lines {
            // Because CacheLine is aligned on 64 bytes it is enough to read single element from the array
            // to spoil the whole cache line
            unsafe { std::ptr::read_volatile(&line.0[0]) };
        }
    }
}

#[repr(C)]
#[repr(align(64))]
#[derive(Default, Clone, Copy)]
struct CacheLine([u16; 32]);

fn shuffle(indices: &mut [usize], seed: u64) {
    let mut rng = SimpleRng::new(seed);

    for i in (1..indices.len()).rev() {
        let j = rng.rand() as usize % (i + 1);
        indices.swap(i, j);
    }
}

struct SimpleRng {
    state: u64,
}

impl SimpleRng {
    fn new(seed: u64) -> Self {
        SimpleRng { state: seed }
    }

    fn rand(&mut self) -> u64 {
        self.state = self.state.wrapping_mul(6364136223846793005).wrapping_add(1);
        self.state
    }
}
