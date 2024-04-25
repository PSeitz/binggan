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
    pub iterations_per_input: Vec<usize>,
    pub results: Vec<BenchResult>,
    pub profiler: Option<PerfProfiler>,
}
impl<I: BenchInputSize> Bench<I> {
    pub fn new(name: String, enable_perf: bool, fun: CallBench<I>) -> Self {
        Bench {
            name,
            fun,
            iterations_per_input: Vec::with_capacity(16), // should be the number of inputs
            results: Vec::with_capacity(NUM_RUNS),
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
    pub input_id: u8,
}
impl BenchResult {
    fn new(duration_ns: u64, memory_consumption: usize, input_id: u8) -> Self {
        BenchResult {
            duration_ns,
            memory_consumption,
            input_id,
        }
    }
}

impl<I> Bench<I> {
    #[inline]
    /// Each input has its own number of iterations.
    pub fn sample_and_set_iter(&mut self, input: &Input<I>) {
        self.iterations_per_input.resize(input.id as usize + 1, 1);
        {
            // Preliminary test if function is very slow
            // This could receive some more thought
            let start = std::time::Instant::now();
            (self.fun)(&input.data);
            let elapsed_ms = start.elapsed().as_millis() as u64;
            const MAX_MS: u64 = 50;
            if elapsed_ms > MAX_MS {
                self.iterations_per_input[input.id as usize] =
                    (MAX_MS / elapsed_ms).max(1) as usize;
                return;
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
        self.iterations_per_input[input.id as usize] = (num_iter as usize).max(4);
    }
    #[inline]
    pub fn exec_bench(&mut self, input: &Input<I>, alloc: &Option<Alloc>) {
        if let Some(alloc) = alloc {
            alloc.reset_peak_memory();
        }
        if let Some(profiler) = &mut self.profiler {
            profiler.enable();
        }
        let num_iter = self.iterations_per_input[input.id as usize];
        let start = std::time::Instant::now();
        for _ in 0..num_iter {
            (self.fun)(&input.data);
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
        let bench_result =
            BenchResult::new(elapsed.as_nanos() as u64 / num_iter as u64, mem, input.id);
        self.results.push(bench_result);
    }
}
