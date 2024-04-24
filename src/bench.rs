use crate::{
    bench_group::{Alloc, NUM_RUNS},
    black_box,
    profiler::{PerfProfiler, Profiler},
    BenchInputSize,
};

type CallBench<I> = Box<dyn Fn(&I)>;

pub(crate) struct Bench<I> {
    pub name: String,
    pub fun: CallBench<I>,
    pub results: Vec<BenchResult>,
    pub num_iter: usize,
    pub profiler: Option<PerfProfiler>,
}
impl<I: BenchInputSize> Bench<I> {
    pub fn new(name: String, enable_perf: bool, fun: CallBench<I>) -> Self {
        Bench {
            name,
            fun,
            results: Vec::with_capacity(NUM_RUNS),
            num_iter: 1,
            profiler: if enable_perf {
                PerfProfiler::new().ok()
            } else {
                None
            },
        }
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
    pub fn sample_and_set_iter(&mut self, input: &I) {
        {
            // Preliminary test if function is very slow
            let start = std::time::Instant::now();
            (self.fun)(input);
            let elapsed_ms = start.elapsed().as_millis() as u64;
            const MAX_MS: u64 = 50;
            if elapsed_ms > MAX_MS {
                self.num_iter = (MAX_MS / elapsed_ms).max(1) as usize;
                return;
            }
        }

        let start = std::time::Instant::now();
        for _ in 0..100 {
            (self.fun)(input);
            black_box(());
        }
        let elapsed_ns = start.elapsed().as_nanos();
        let per_iter_ns = (elapsed_ns / 100) * NUM_RUNS as u128;

        // We want to run the benchmark for 100ms
        let num_iter = 100_000_000 / per_iter_ns;
        self.num_iter = (num_iter as usize).max(4);
    }
    #[inline]
    pub fn exec_bench(&mut self, input: &(String, I), alloc: &Option<Alloc>) {
        if let Some(alloc) = alloc {
            alloc.reset_peak_memory();
        }
        if let Some(profiler) = &mut self.profiler {
            profiler.enable();
        }
        let start = std::time::Instant::now();
        for _ in 0..self.num_iter {
            (self.fun)(&input.1);
        }
        let elapsed = start.elapsed();
        if let Some(profiler) = &mut self.profiler {
            profiler.disable();
        }
        let mem = if let Some(alloc) = alloc {
            alloc.get_peak_memory()
        } else {
            0
        };
        let bench_result = BenchResult::new(elapsed.as_nanos() as u64 / self.num_iter as u64, mem);
        self.results.push(bench_result);
    }
}
