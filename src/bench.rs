use crate::{
    bench_id::BenchId,
    black_box,
    output_value::OutputValue,
    plugins::{alloc::*, *},
    stats::*,
};

/// The trait which typically wraps a InputWithBenchmark and allows to hide the generics.
pub trait Bench<'a> {
    /// Returns the number of iterations the benchmark should do
    fn get_num_iter(&self) -> Option<usize>;
    fn set_num_iter(&mut self, num_iter: usize);
    /// Sample the number of iterations the benchmark should do
    fn sample_num_iter(&mut self) -> usize;
    fn exec_bench(&mut self, events: &mut PluginManager);
    fn get_results(&mut self, events: &mut PluginManager) -> BenchResult;
    fn clear_results(&mut self);
}

pub(crate) type CallBench<'a, I, O> = Box<dyn FnMut(&'a I) -> Option<O>>;

pub(crate) struct NamedBench<'a, I, O> {
    pub bench_id: BenchId,
    pub fun: CallBench<'a, I, O>,
    pub num_group_iter: usize,
}
impl<'a, I, O> NamedBench<'a, I, O> {
    pub fn new(bench_id: BenchId, fun: CallBench<'a, I, O>, num_group_iter: usize) -> Self {
        Self {
            bench_id,
            fun,
            num_group_iter,
        }
    }
}

/// The result of a single benchmark.
#[derive(Debug, Clone)]
pub struct BenchResult {
    /// The bench id uniquely identifies the benchmark.
    /// It is a combination of the group name, input name and benchmark name.
    pub bench_id: BenchId,
    /// The aggregated statistics of the benchmark run.
    pub stats: BenchStats,
    /// The aggregated statistics of the previous run.
    pub old_stats: Option<BenchStats>,
    /// The performance counter values of the benchmark run. (Linux only)
    pub perf_counter: Option<CounterValues>,
    /// The performance counter values of the previous benchmark run. (Linux only)
    pub old_perf_counter: Option<CounterValues>,
    /// The size of the input in bytes if available.
    pub input_size_in_bytes: Option<usize>,
    /// The size of the output returned by the bench. Enables reporting.
    pub output_value: Option<String>,
    /// Memory tracking is enabled and the peak memory consumption is reported.
    pub tracked_memory: bool,
}

/// Bundle of input and benchmark for running benchmarks
pub(crate) struct InputWithBenchmark<'a, I, O> {
    pub(crate) input: &'a I,
    pub(crate) input_size_in_bytes: Option<usize>,
    pub(crate) bench: NamedBench<'a, I, O>,
    pub(crate) results: Vec<RunResult<O>>,
    pub num_iter: Option<usize>,
}

impl<'a, I, O> InputWithBenchmark<'a, I, O> {
    pub fn new(
        input: &'a I,
        input_size_in_bytes: Option<usize>,
        bench: NamedBench<'a, I, O>,
        num_iter: Option<usize>,
    ) -> Self {
        InputWithBenchmark {
            input,
            input_size_in_bytes,
            results: Vec::with_capacity(bench.num_group_iter),
            bench,
            num_iter,
        }
    }
}

impl<I, O: OutputValue> InputWithBenchmark<'_, I, O> {
    fn get_num_iter_or_fail(&self) -> usize {
        self.num_iter
            .expect("Number of iterations not set. Call set_num_iter before running the benchmark.")
    }
}
impl<'a, I, O: OutputValue> Bench<'a> for InputWithBenchmark<'a, I, O> {
    #[inline]
    fn sample_num_iter(&mut self) -> usize {
        self.bench.sample_and_get_iter(self.input)
    }
    fn get_num_iter(&self) -> Option<usize> {
        self.num_iter
    }
    fn set_num_iter(&mut self, num_iter: usize) {
        self.num_iter = Some(num_iter);
    }

    #[inline]
    fn exec_bench(&mut self, events: &mut PluginManager) {
        let num_iter = self.get_num_iter_or_fail();
        let res = self.bench.exec_bench(self.input, num_iter, events);
        self.results.push(res);
    }

    fn get_results(&mut self, events: &mut PluginManager) -> BenchResult {
        let num_iter = self.get_num_iter_or_fail();
        let total_num_iter = self.bench.num_group_iter as u64 * num_iter as u64;
        let memory_consumption: Option<&Vec<usize>> = events
            .downcast_plugin::<AllocPerBench>(ALLOC_EVENT_LISTENER_NAME)
            .and_then(|counters| counters.get_by_bench_id(&self.bench.bench_id));
        let stats = compute_stats(&self.results, memory_consumption);
        let tracked_memory = memory_consumption.is_some();

        let perf_counter = get_perf_counter(events, &self.bench.bench_id, total_num_iter);
        let output_value = (self.bench.fun)(self.input);
        BenchResult {
            bench_id: self.bench.bench_id.clone(),
            stats,
            perf_counter,
            input_size_in_bytes: self.input_size_in_bytes,
            tracked_memory,
            output_value: output_value.and_then(|el| el.format()),
            old_stats: None,
            old_perf_counter: None,
        }
    }

    fn clear_results(&mut self) {
        self.results.clear();
    }
}

fn get_perf_counter(
    _events: &mut PluginManager,
    _bench_id: &BenchId,
    _total_num_iter: u64,
) -> Option<CounterValues> {
    #[cfg(target_os = "linux")]
    {
        _events
            .downcast_plugin::<PerfCounterPerBench>(PERF_CNT_EVENT_LISTENER_NAME)
            .and_then(|counters| {
                counters
                    .get_by_bench_id_mut(_bench_id)
                    .and_then(|perf_cnt| perf_cnt.finish(_total_num_iter).ok())
            })
    }
    #[cfg(not(target_os = "linux"))]
    {
        None
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
/// The result of a single benchmark run. This is already aggregated since a single bench may be
/// run multiple times to improve the accuracy.
/// There are multiple runs in a group for each benchmark which will be collected to a vector
pub struct RunResult<O> {
    pub duration_ns: u64,
    pub output: Option<O>,
}
impl<O> RunResult<O> {
    fn new(duration_ns: u64, output: Option<O>) -> Self {
        RunResult {
            duration_ns,
            output,
        }
    }
}

impl<'a, I, O> NamedBench<'a, I, O> {
    #[inline]
    /// Each group has its own number of iterations. This is not the final num_iter
    pub fn sample_and_get_iter(&mut self, input: &'a I) -> usize {
        // We want to run the benchmark for 500ms
        const TARGET_MS_PER_BENCH: u64 = 1000;
        const TARGET_NS_PER_BENCH: u128 = TARGET_MS_PER_BENCH as u128 * 1_000_000;
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
        let per_iter_ns = elapsed_ns / 64;
        // The test is run multiple times in the group.
        let per_iter_ns_group_run = self.num_group_iter as u128 * per_iter_ns;

        let num_iter = TARGET_NS_PER_BENCH / per_iter_ns_group_run;
        // We want to run the benchmark for at least 1 iterations
        (num_iter as usize).max(1)
    }
    #[inline]
    pub fn exec_bench(
        &mut self,
        input: &'a I,
        num_iter: usize,
        events: &mut PluginManager,
    ) -> RunResult<O> {
        events.emit(PluginEvents::BenchStart {
            bench_id: &self.bench_id,
        });
        let start = std::time::Instant::now();
        let mut res = None;
        for _ in 0..num_iter {
            res = black_box((self.fun)(input));
        }
        let elapsed = start.elapsed();

        let run_result = RunResult::new(elapsed.as_nanos() as u64 / num_iter as u64, res);
        events.emit(PluginEvents::BenchStop {
            bench_id: &self.bench_id,
            duration: run_result.duration_ns,
        });
        run_result
    }
}
