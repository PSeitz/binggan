use crate::{
    bench_group::{Alloc, Input, NUM_RUNS},
    black_box,
    profiler::{PerfProfiler, Profiler},
    BenchInputSize,
};

type CallBench<I> = Box<dyn Fn(&I)>;

pub(crate) struct Bench<I> {
    pub name: String,
    pub fun: CallBench<I>,
}
impl<I: BenchInputSize> Bench<I> {
    pub fn new(name: String, fun: CallBench<I>) -> Self {
        Bench { name, fun }
    }
}

/// Bundle of input and benchmark for running benchmarks
pub(crate) struct InputWithBenchmark<'a, I> {
    pub(crate) input: &'a Input<I>,
    pub(crate) input_size_in_bytes: Option<usize>,
    pub(crate) bench: &'a Bench<I>,
    pub(crate) results: Vec<BenchResult>,
    pub profiler: Option<PerfProfiler>,
    pub num_iter: usize,
}
impl<'a, I: BenchInputSize> InputWithBenchmark<'a, I> {
    pub fn new(
        input: &'a Input<I>,
        input_size_in_bytes: Option<usize>,
        bench: &'a Bench<I>,
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

    #[inline]
    /// Each input has its own number of iterations.
    pub fn sample_and_set_iter(&mut self, input: &Input<I>) {
        self.num_iter = self.bench.sample_and_get_iter(input);
        self.results.reserve(NUM_RUNS * self.num_iter);
    }
    #[inline]
    pub fn exec_bench(&mut self, alloc: &Option<Alloc>) {
        let res = self
            .bench
            .exec_bench(self.input, alloc, &mut self.profiler, self.num_iter);
        self.results.push(res);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct BenchResult {
    pub duration_ns: u64,
    pub memory_consumption: usize,
}
impl BenchResult {
    fn new(duration_ns: u64, memory_consumption: usize) -> Self {
        BenchResult {
            duration_ns,
            memory_consumption,
        }
    }
}

impl<I> Bench<I> {
    #[inline]
    /// Each input has its own number of iterations.
    pub fn sample_and_get_iter(&self, input: &Input<I>) -> usize {
        {
            // Preliminary test if function is very slow
            // This could receive some more thought
            let start = std::time::Instant::now();
            (self.fun)(&input.data);
            let elapsed_ms = start.elapsed().as_millis() as u64;
            const MAX_MS: u64 = 50;
            if elapsed_ms > MAX_MS {
                return (MAX_MS / elapsed_ms).max(1) as usize;
            }
        }

        let start = std::time::Instant::now();
        for _ in 0..100 {
            (self.fun)(&input.data);
            black_box(());
        }
        let elapsed_ns = start.elapsed().as_nanos();
        let per_iter_ns = (elapsed_ns / 100) * NUM_RUNS as u128;

        // We want to run the benchmark for 100ms
        let num_iter = 100_000_000 / per_iter_ns;
        (num_iter as usize).max(4)
    }
    #[inline]
    pub fn exec_bench(
        &self,
        input: &Input<I>,
        alloc: &Option<Alloc>,
        profiler: &mut Option<PerfProfiler>,
        num_iter: usize,
    ) -> BenchResult {
        if let Some(alloc) = alloc {
            alloc.reset_peak_memory();
        }
        if let Some(profiler) = profiler {
            profiler.enable();
        }
        let start = std::time::Instant::now();
        for _ in 0..num_iter {
            (self.fun)(&input.data);
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
        
        BenchResult::new(elapsed.as_nanos() as u64 / num_iter as u64, mem)
    }
}
