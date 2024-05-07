use crate::{bench_input_group::*, bench_runner::NamedInput, black_box, profiler::*, stats::*};

pub trait Bench<'a> {
    fn get_input_name(&self) -> &str;
    fn sample_and_set_iter(&mut self);
    fn exec_bench(&mut self, alloc: &Option<Alloc>);
    fn get_results(&mut self, group_name: &Option<String>) -> BenchResult;
    fn clear_results(&mut self);
}

type CallBench<'a, I> = Box<dyn Fn(&'a I)>;

pub(crate) struct NamedBench<'a, I> {
    pub name: String,
    pub fun: CallBench<'a, I>,
}
impl<'a, I> NamedBench<'a, I> {
    pub fn new(name: String, fun: CallBench<'a, I>) -> Self {
        Self { name, fun }
    }
}

/// Bundle of input and benchmark for running benchmarks
pub(crate) struct InputWithBenchmark<'a, I> {
    pub(crate) input: NamedInput<'a, I>,
    pub(crate) input_size_in_bytes: Option<usize>,
    pub(crate) bench: NamedBench<'a, I>,
    pub(crate) results: Vec<RunResult>,
    pub profiler: Option<PerfProfiler>,
    pub num_iter: usize,
}
impl<'a, I> InputWithBenchmark<'a, I> {
    pub fn new(
        input: NamedInput<'a, I>,
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
    pub bench_id: String,
    /// The name of the benchmark.
    pub bench_name: String,
    /// The name of the input.
    #[allow(dead_code)]
    pub input_name: String,
    /// The aggregated statistics of the benchmark run.
    pub stats: BenchStats,
    /// The performance counter values of the benchmark run. (Linux only)
    pub perf_counter: Option<CounterValues>,
    /// The size of the input in bytes if available.
    pub input_size_in_bytes: Option<usize>,
}

impl<'a, I> Bench<'a> for InputWithBenchmark<'a, I> {
    #[inline]
    fn get_input_name(&self) -> &str {
        &self.input.name
    }
    #[inline]
    fn sample_and_set_iter(&mut self) {
        self.num_iter = self.bench.sample_and_get_iter(&self.input);
        self.results.reserve(NUM_RUNS * self.num_iter);
    }
    #[inline]
    fn exec_bench(&mut self, alloc: &Option<Alloc>) {
        let res = self
            .bench
            .exec_bench(&self.input, alloc, &mut self.profiler, self.num_iter);
        self.results.push(res);
    }

    fn get_results(&mut self, group_name: &Option<String>) -> BenchResult {
        let bench_id = format!(
            "{}_{}_{}",
            group_name.as_ref().unwrap_or(&"".to_string()),
            self.input.name,
            self.bench.name
        )
        .replace('/', "-");
        let stats = compute_stats(&self.results);
        let perf_counter: Option<CounterValues> = self
            .profiler
            .as_mut()
            .and_then(|profiler| profiler.finish(NUM_RUNS as u64 * self.num_iter as u64).ok());
        BenchResult {
            bench_id,
            stats,
            perf_counter,
            input_size_in_bytes: self.input_size_in_bytes,
            bench_name: self.bench.name.clone(),
            input_name: self.input.name.to_string(),
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
}
impl RunResult {
    fn new(duration_ns: u64, memory_consumption: usize) -> Self {
        RunResult {
            duration_ns,
            memory_consumption,
        }
    }
}

impl<'a, I> NamedBench<'a, I> {
    #[inline]
    /// Each input has its own number of iterations.
    pub fn sample_and_get_iter(&self, input: &NamedInput<'a, I>) -> usize {
        {
            // Preliminary test if function is very slow
            // This could receive some more thought
            let start = std::time::Instant::now();
            (self.fun)(input.data);
            let elapsed_ms = start.elapsed().as_millis() as u64;
            const MAX_MS: u64 = 5;
            if elapsed_ms > MAX_MS {
                return (MAX_MS / elapsed_ms).max(1) as usize;
            }
        }

        let start = std::time::Instant::now();
        for _ in 0..64 {
            (self.fun)(input.data);
            black_box(());
        }
        let elapsed_ns = start.elapsed().as_nanos();
        let per_iter_ns = (elapsed_ns / 100) * NUM_RUNS as u128;

        // We want to run the benchmark for 100ms
        let num_iter = 100_000_000 / per_iter_ns;
        // We want to run the benchmark for at least 1 iterations
        (num_iter as usize).max(1)
    }
    #[inline]
    pub fn exec_bench(
        &self,
        input: &NamedInput<'a, I>,
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
        for _ in 0..num_iter {
            (self.fun)(input.data);
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

        RunResult::new(elapsed.as_nanos() as u64 / num_iter as u64, mem)
    }
}
