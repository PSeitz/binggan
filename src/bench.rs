use std::sync::atomic;

use crate::{
    bench_id::BenchId,
    black_box,
    output_value::OutputValue,
    plugins::{alloc::*, *},
    stats::*,
};
use quanta::Clock;

/// The trait which typically wraps a InputWithBenchmark and allows to hide the generics.
pub trait Bench<'a> {
    /// Returns the number of iterations the benchmark should do
    fn get_num_iter(&self) -> Option<usize>;
    fn set_num_iter(&mut self, num_iter: usize, plugins: &mut PluginManager);
    /// Sample the number of iterations the benchmark should do
    fn sample_num_iter(&mut self) -> usize;
    fn exec_bench(&mut self, plugins: &mut PluginManager);
    fn get_results(&mut self, plugins: &mut PluginManager) -> BenchResult;
    fn clear_results(&mut self);
}

pub(crate) type CallBench<'a, I, O> = Box<dyn FnMut(&'a I) -> O + 'a>;

pub(crate) struct NamedBench<'a, I, O> {
    pub bench_id: BenchId,
    pub fun: CallBench<'a, I, O>,
    pub num_group_iter: usize,
    clock: Clock,
    adjust_for_single_threaded_cpu_scheduling: bool,
}
impl<'a, I, O: OutputValue> NamedBench<'a, I, O> {
    pub fn new(
        bench_id: BenchId,
        fun: CallBench<'a, I, O>,
        num_group_iter: usize,
        adjust_for_single_threaded_cpu_scheduling: bool,
    ) -> Self {
        Self {
            bench_id,
            fun,
            num_group_iter,
            clock: Clock::new(),
            adjust_for_single_threaded_cpu_scheduling,
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
    pub perf_counter: Option<PerfCounterValues>,
    /// The performance counter values of the previous benchmark run. (Linux only)
    pub old_perf_counter: Option<PerfCounterValues>,
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
    fn set_num_iter(&mut self, num_iter: usize, _plugins: &mut PluginManager) {
        self.num_iter = Some(num_iter);
    }

    #[inline]
    fn exec_bench(&mut self, plugins: &mut PluginManager) {
        let num_iter = self.get_num_iter_or_fail();
        let res = self.bench.exec_bench(self.input, num_iter, plugins);
        self.results.push(res);
    }

    fn get_results(&mut self, plugins: &mut PluginManager) -> BenchResult {
        let num_iter = self.get_num_iter_or_fail();
        let total_num_iter = self.bench.num_group_iter as u64 * num_iter as u64;
        let memory_consumption: Option<&Vec<usize>> = plugins
            .downcast_plugin::<PeakMemAllocPlugin>(ALLOC_EVENT_LISTENER_NAME)
            .and_then(|counters| counters.get_by_bench_id(&self.bench.bench_id));
        let stats = compute_stats(&self.results, memory_consumption);
        let tracked_memory = memory_consumption.is_some();

        let perf_counter = get_perf_counter(plugins, &self.bench.bench_id, total_num_iter);
        let output_value = (self.bench.fun)(self.input);
        BenchResult {
            bench_id: self.bench.bench_id.clone(),
            stats,
            perf_counter,
            input_size_in_bytes: self.input_size_in_bytes,
            tracked_memory,
            output_value: output_value.format(),
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
) -> Option<PerfCounterValues> {
    #[cfg(target_os = "linux")]
    {
        _events
            .downcast_plugin::<PerfCounterPlugin>(PERF_CNT_EVENT_LISTENER_NAME)
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
    pub output: O,
}
impl<O> RunResult<O> {
    fn new(duration_ns: u64, output: O) -> Self {
        RunResult {
            duration_ns,
            output,
        }
    }
}

impl<'a, I, O: OutputValue> NamedBench<'a, I, O> {
    #[inline]
    /// Each group has its own number of iterations. This is not the final num_iter
    pub fn sample_and_get_iter(&mut self, input: &'a I) -> usize {
        // We want to run the benchmark for 500ms
        const TARGET_MS_PER_BENCH: u64 = 500;
        const TARGET_NS_PER_BENCH: u128 = TARGET_MS_PER_BENCH as u128 * 1_000_000;
        {
            // Preliminary test if function is very slow
            let start = self.clock.raw();
            #[allow(clippy::unit_arg)]
            black_box((self.fun)(input));
            let elapsed_ms = self.clock.delta_as_nanos(start, self.clock.raw()) / 1_000_000;
            if elapsed_ms > TARGET_MS_PER_BENCH {
                return 1;
            }
        }

        let start = self.clock.raw();
        for _ in 0..64 {
            #[allow(clippy::unit_arg)]
            black_box((self.fun)(input));
        }
        let elapsed_ns = self.clock.delta_as_nanos(start, self.clock.raw());
        if elapsed_ns == 0 {
            return 1;
        }
        let per_iter_ns = u128::from(elapsed_ns) / 64;
        if per_iter_ns == 0 {
            return 1;
        }
        // The test is run multiple times in the group.
        let per_iter_ns_group_run = self.num_group_iter as u128 * per_iter_ns;
        if per_iter_ns_group_run == 0 {
            return 1;
        }

        let num_iter = TARGET_NS_PER_BENCH / per_iter_ns_group_run;
        // We want to run the benchmark for at least 1 iterations
        (num_iter as usize).max(1)
    }
    #[inline]
    pub fn exec_bench(
        &mut self,
        input: &'a I,
        num_iter: usize,
        plugins: &mut PluginManager,
    ) -> RunResult<O> {
        plugins.emit(PluginEvents::BenchStart {
            bench_id: &self.bench_id,
        });
        debug_assert!(num_iter > 0);

        // Defer dropping outputs so destructor cost is not part of the measured time.
        let run_result = if O::defer_drop() {
            // Accumulate raw deltas and scale once at the end.
            // Scaling is linear, so `scale(sum(delta)) == sum(scale(delta))`.
            let mut sum_raw = 0u64;
            let mut adjuster = if self.adjust_for_single_threaded_cpu_scheduling {
                SingleThreadedCpuSchedulingAdjuster::start(&self.clock)
            } else {
                None
            };
            let mut res: Option<O> = None;
            // In this mode, we measure each iteration separately to avoid destructor cost.
            // There may be some overhead, but it should be outweighed by benchmarks that allocate
            for _ in 0..num_iter {
                // We drop the value first to avoid measuring destructor time
                // and to avoid keeping multiple outputs in memory.
                atomic::compiler_fence(atomic::Ordering::SeqCst);
                black_box(res.take());
                atomic::compiler_fence(atomic::Ordering::SeqCst);
                let start = self.clock.raw();
                atomic::compiler_fence(atomic::Ordering::SeqCst);
                let val = black_box((self.fun)(input));
                atomic::compiler_fence(atomic::Ordering::SeqCst);
                let end = self.clock.raw();
                sum_raw = sum_raw.saturating_add(end.saturating_sub(start));
                res = Some(val);
            }
            let sum_ns = self.clock.delta_as_nanos(0, sum_raw);
            let adjusted_ns = adjuster
                .as_mut()
                .and_then(|adjuster| adjuster.finish(sum_ns, &self.clock))
                .unwrap_or(sum_ns);
            let duration_ns = adjusted_ns / num_iter as u64;
            RunResult::new(duration_ns, res.unwrap())
        } else {
            let start = self.clock.raw();
            let mut adjuster = if self.adjust_for_single_threaded_cpu_scheduling {
                SingleThreadedCpuSchedulingAdjuster::start_with_wall(start)
            } else {
                None
            };
            let mut res: Option<O> = None;
            for _ in 0..num_iter {
                res = Some(black_box((self.fun)(input)));
            }
            let end = self.clock.raw();
            let elapsed_ns = self.clock.delta_as_nanos(start, end);
            let adjusted_ns = adjuster
                .as_mut()
                .and_then(|adjuster| adjuster.finish_with_wall(elapsed_ns, end, &self.clock))
                .unwrap_or(elapsed_ns);
            let duration_ns = adjusted_ns / num_iter as u64;
            RunResult::new(duration_ns, res.unwrap())
        };

        plugins.emit(PluginEvents::BenchStop {
            bench_id: &self.bench_id,
            duration: run_result.duration_ns,
        });
        run_result
    }
}

/// Adjusts measured wall time by subtracting time the single thread was not scheduled.
///
/// Uses wall time from `quanta::Clock` and per-thread CPU time from
/// `clock_gettime(CLOCK_THREAD_CPUTIME_ID)` on Linux. That clock reports
/// CPU time consumed by the calling thread only (does not advance while
/// the thread is off-CPU or blocked), so `wall - cpu` approximates time
/// spent descheduled. This is subtracted from the measured duration and
/// assumes a single-threaded benchmark.
struct SingleThreadedCpuSchedulingAdjuster {
    wall_start_raw: u64,
    cpu_start_ns: u64,
}

impl SingleThreadedCpuSchedulingAdjuster {
    fn start(clock: &Clock) -> Option<Self> {
        Self::start_with_wall(clock.raw())
    }

    fn start_with_wall(wall_start_raw: u64) -> Option<Self> {
        let cpu_start_ns = thread_cpu_time_ns()?;
        Some(Self {
            wall_start_raw,
            cpu_start_ns,
        })
    }

    fn finish(&mut self, elapsed_ns: u64, clock: &Clock) -> Option<u64> {
        self.finish_with_wall(elapsed_ns, clock.raw(), clock)
    }

    fn finish_with_wall(
        &mut self,
        elapsed_ns: u64,
        wall_end_raw: u64,
        clock: &Clock,
    ) -> Option<u64> {
        let cpu_end_ns = thread_cpu_time_ns()?;
        let wall_ns = clock.delta_as_nanos(self.wall_start_raw, wall_end_raw);
        let cpu_ns = cpu_end_ns.saturating_sub(self.cpu_start_ns);
        // The difference between wall time and thread CPU time is time not scheduled.
        let unscheduled_ns = wall_ns.saturating_sub(cpu_ns);
        // Subtract unscheduled time from the measured duration.
        Some(elapsed_ns.saturating_sub(unscheduled_ns))
    }
}

#[cfg(target_os = "linux")]
fn thread_cpu_time_ns() -> Option<u64> {
    let mut ts = libc::timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };
    let res = unsafe { libc::clock_gettime(libc::CLOCK_THREAD_CPUTIME_ID, &mut ts) };
    if res == 0 {
        let secs = ts.tv_sec as u64;
        let nanos = ts.tv_nsec as u64;
        Some(secs.saturating_mul(1_000_000_000).saturating_add(nanos))
    } else {
        None
    }
}

#[cfg(not(target_os = "linux"))]
fn thread_cpu_time_ns() -> Option<u64> {
    None
}
