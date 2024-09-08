use crate::{
    bench_id::BenchId, bench_input_group::*, bench_runner::NUM_RUNS, black_box, profiler::*,
    stats::*,
};

/// The trait which typically wraps a InputWithBenchmark and allows to hide the generics.
pub trait Bench<'a> {
    fn set_num_iter(&mut self, num_iter: usize);
    /// Sample the number of iterations the benchmark should do
    fn sample_num_iter(&mut self) -> usize;
    fn exec_bench(&mut self, alloc: &Option<Alloc>);
    fn get_results(&mut self) -> BenchResult;
    fn clear_results(&mut self);
}

pub(crate) type CallBench<'a, I> = Box<dyn FnMut(&'a I) -> Option<u64>>;

pub(crate) struct NamedBench<'a, I> {
    pub bench_id: BenchId,
    pub fun: CallBench<'a, I>,
}
impl<'a, I> NamedBench<'a, I> {
    pub fn new(bench_id: BenchId, fun: CallBench<'a, I>) -> Self {
        Self { bench_id, fun }
    }
}

/// Bundle of input and benchmark for running benchmarks
pub(crate) struct InputWithBenchmark<'a, I> {
    pub(crate) input: &'a I,
    pub(crate) input_size_in_bytes: Option<usize>,
    //pub(crate) bench_id: NamedBench<'a, I>,
    pub(crate) bench: NamedBench<'a, I>,
    pub(crate) results: Vec<RunResult>,
    pub profiler: Option<PerfProfiler>,
    pub num_iter: usize,
}

impl<'a, I> InputWithBenchmark<'a, I> {
    pub fn new(
        input: &'a I,
        input_size_in_bytes: Option<usize>,
        bench: NamedBench<'a, I>,
        enable_perf: bool,
    ) -> Self {
        InputWithBenchmark {
            input,
            input_size_in_bytes,
            bench,
            results: Vec::new(),
            num_iter: 1,
            profiler: if enable_perf {
                PerfProfiler::new().ok()
            } else {
                None
            },
        }
    }
}
/// The result of a benchmark run.
pub struct BenchResult {
    /// The bench id uniquely identifies the benchmark.
    /// It is a combination of the group name, input name and benchmark name.
    pub bench_id: BenchId,
    /// The aggregated statistics of the benchmark run.
    pub stats: BenchStats,
    /// The performance counter values of the benchmark run. (Linux only)
    pub perf_counter: Option<CounterValues>,
    /// The size of the input in bytes if available.
    pub input_size_in_bytes: Option<usize>,
    /// The size of the output returned by the bench. Enables reporting.
    pub output_value: Option<u64>,
}

impl<'a, I> Bench<'a> for InputWithBenchmark<'a, I> {
    #[inline]
    fn sample_num_iter(&mut self) -> usize {
        self.bench.sample_and_get_iter(self.input)
    }
    fn set_num_iter(&mut self, num_iter: usize) {
        self.num_iter = num_iter;
        self.results.reserve(NUM_RUNS * self.num_iter);
    }

    #[inline]
    fn exec_bench(&mut self, alloc: &Option<Alloc>) {
        let res = self
            .bench
            .exec_bench(self.input, alloc, &mut self.profiler, self.num_iter);
        self.results.push(res);
    }

    fn get_results(&mut self) -> BenchResult {
        let stats = compute_stats(&self.results, self.num_iter);
        let perf_counter: Option<CounterValues> = self
            .profiler
            .as_mut()
            .and_then(|profiler| profiler.finish(NUM_RUNS as u64 * self.num_iter as u64).ok());
        let output_value = (self.bench.fun)(self.input);
        BenchResult {
            bench_id: self.bench.bench_id.clone(),
            stats,
            perf_counter,
            input_size_in_bytes: self.input_size_in_bytes,
            output_value,
        }
    }

    fn clear_results(&mut self) {
        self.results.clear();
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
/// The result of a single benchmark run.
/// There are multiple runs for each benchmark which will be collected to a vector
pub struct RunResult {
    pub duration_ns: u64,
    pub memory_consumption: usize,
    pub output: Option<u64>,
}
impl RunResult {
    fn new(duration_ns: u64, memory_consumption: usize, output: Option<u64>) -> Self {
        RunResult {
            duration_ns,
            memory_consumption,
            output,
        }
    }
}

impl<'a, I> NamedBench<'a, I> {
    #[inline]
    /// Each group has its own number of iterations. This is not the final num_iter
    pub fn sample_and_get_iter(&mut self, input: &'a I) -> usize {
        // We want to run the benchmark for 100ms
        const TARGET_MS_PER_BENCH: u64 = 100;
        {
            // Preliminary test if function is very slow
            let start = std::time::Instant::now();
            #[allow(clippy::unit_arg)]
            black_box((self.fun)(input));
            let elapsed_ms = start.elapsed().as_millis() as u64;
            if elapsed_ms > TARGET_MS_PER_BENCH {
                return 1;
            }
        }

        let start = std::time::Instant::now();
        for _ in 0..64 {
            #[allow(clippy::unit_arg)]
            black_box((self.fun)(input));
        }
        let elapsed_ns = start.elapsed().as_nanos();
        let per_iter_ns = (elapsed_ns / 100) * NUM_RUNS as u128;

        let num_iter = TARGET_MS_PER_BENCH as u128 * 1_000_000 / per_iter_ns;
        // We want to run the benchmark for at least 1 iterations
        (num_iter as usize).max(1)
    }
    #[inline]
    pub fn exec_bench(
        &mut self,
        input: &'a I,
        alloc: &Option<Alloc>,
        profiler: &mut Option<PerfProfiler>,
        num_iter: usize,
    ) -> RunResult {
        if let Some(alloc) = alloc {
            alloc.reset_peak_memory();
        }
        if let Some(profiler) = profiler {
            profiler.enable();
        }
        let start = std::time::Instant::now();
        let mut res = None;
        for _ in 0..num_iter {
            res = black_box((self.fun)(input));
        }
        let elapsed = start.elapsed();
        if let Some(profiler) = profiler {
            profiler.disable();
        }
        let mem = if let Some(alloc) = alloc {
            alloc.get_peak_memory()
        } else {
            0
        };

        RunResult::new(elapsed.as_nanos() as u64 / num_iter as u64, mem, res)
    }
}
