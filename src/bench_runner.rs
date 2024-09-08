use std::{alloc::GlobalAlloc, cmp::Ordering};

use crate::{
    bench::{Bench, BenchResult, InputWithBenchmark, NamedBench},
    black_box, parse_args,
    report::report_group,
    BenchGroup, Config,
};
use core::mem::size_of;
use peakmem_alloc::*;
use yansi::Paint;

pub(crate) type Alloc = &'static dyn PeakMemAllocTrait;

/// Each bench is run N times in a inner loop.
/// The outer loop is fixed. In the outer loop the order of the benchmarks in a group is shuffled.
pub const NUM_RUNS: usize = 32;

/// The main struct to run benchmarks.
///
#[derive(Clone)]
pub struct BenchRunner {
    alloc: Option<Alloc>,
    cache_trasher: CacheTrasher,
    pub(crate) options: Config,
    /// The size of the input.
    /// Enables throughput reporting.
    input_size_in_bytes: Option<usize>,

    /// Name of the test
    pub(crate) name: Option<String>,
}

pub const EMPTY_INPUT: &() = &();

impl Default for BenchRunner {
    fn default() -> Self {
        Self::new()
    }
}

impl BenchRunner {
    /// Creates a new BenchRunner.
    pub fn new() -> Self {
        Self::new_with_options(parse_args())
    }

    /// Creates a new BenchRunner and prints the bench name.
    pub fn with_name<S: AsRef<str>>(name: S) -> Self {
        let mut new = Self::new();
        new.set_name(name.as_ref());
        new
    }

    /// Creates a new `BenchRunner` with custom options set.
    pub(crate) fn new_with_options(options: Config) -> Self {
        use yansi::Condition;
        yansi::whenever(Condition::TTY_AND_COLOR);

        BenchRunner {
            cache_trasher: CacheTrasher::new(1024 * 1024 * 16),
            options,
            alloc: None,
            input_size_in_bytes: None,
            name: None,
        }
    }

    /// Creates a new `BenchGroup`
    /// The group is a collection of benchmarks that are run together.
    pub fn new_group<'a>(&self) -> BenchGroup<'a> {
        BenchGroup::new(self.clone())
    }
    /// Creates a new named `BenchGroup`
    /// The group is a collection of benchmarks that are run together.
    ///
    /// The name of the group could be for example the label of the input on which the group is
    /// run.
    pub fn new_group_with_name<'a, S: Into<String>>(&self, name: S) -> BenchGroup<'a> {
        BenchGroup::with_name(self.clone(), name)
    }

    /// Set the name of the current test.
    /// It is also used to distinguish when writing the results to disk.
    pub fn set_name<S: AsRef<str>>(&mut self, name: S) {
        println!("{}", name.as_ref().black().on_red().invert().bold());
        self.name = Some(name.as_ref().to_string());
    }

    /// Set the peak mem allocator to be used for the benchmarks.
    /// This will report the peak memory consumption of the benchmarks.
    pub fn set_alloc<A: GlobalAlloc + 'static>(&mut self, alloc: &'static PeakMemAlloc<A>) {
        self.alloc = Some(alloc);
    }

    /// Enables throughput reporting. The throughput will be valid for all inputs that are
    /// registered afterwards.
    pub fn set_input_size(&mut self, input_size: usize) {
        self.input_size_in_bytes = Some(input_size);
    }

    /// Run a single function. This will directly execute and report the function and therefore does
    /// not support interleaved execution.
    ///
    /// The return value of the function will be reported as the `OutputValue` if it is `Some`.
    pub fn bench_function<F, S: Into<String>>(&mut self, name: S, f: F) -> &mut Self
    where
        F: Fn(&()) -> Option<u64> + 'static,
    {
        let named_bench = NamedBench::new(name.into(), Box::new(f));
        let bundle = InputWithBenchmark::new(
            EMPTY_INPUT,
            self.input_size_in_bytes,
            named_bench,
            self.options.enable_perf,
        );

        self.run_group(None, &mut [Box::new(bundle)]);
        self
    }

    /// Configure the benchmark.
    ///
    /// See the [Config] struct for more information.
    pub fn config(&mut self) -> &mut Config {
        &mut self.options
    }

    /// Run the benchmarks and report the results.
    pub fn run_group<'a>(
        &self,
        group_name: Option<&str>,
        group: &mut [Box<dyn Bench<'a> + 'a>],
    ) -> Vec<BenchResult> {
        if group.is_empty() {
            return Vec::new();
        }

        if let Some(name) = &group_name {
            println!("{}", name.black().on_yellow().invert().bold());
        }

        const MAX_GROUP_SIZE: usize = 5;
        if self.options.verbose && group.len() > MAX_GROUP_SIZE {
            println!(
                "Group is quite big, splitting into chunks of {} elements",
                MAX_GROUP_SIZE
            );
        }

        // If the group is quite big, we don't want to create too big chunks, which causes
        // slow tests, therefore a chunk is at most 5 elements large.
        for group in group.chunks_mut(MAX_GROUP_SIZE) {
            Self::warm_up_group_and_set_iter(group, self.options.num_iter, self.options.verbose);

            if self.options.interleave {
                Self::run_interleaved(
                    group,
                    &self.alloc,
                    self.options.cache_trasher.then_some(&self.cache_trasher),
                );
            } else {
                Self::run_sequential(group, &self.alloc);
            }
        }
        // We sort at the end, so the alignment is correct (could be calculated up front)
        let test_name = format!(
            "{}_{}",
            self.name.as_deref().unwrap_or_default(),
            group_name.unwrap_or_default()
        );

        report_group(&test_name, group, self.alloc.is_some());

        // TODO: clearing should be optional, to check the results yourself, e.g. in CI
        //for bench in group {
        //bench.clear_results();
        //}
        group
            .iter_mut()
            .map(|b| b.get_results(&test_name))
            .collect()
    }

    fn run_sequential<'a>(benches: &mut [Box<dyn Bench<'a> + 'a>], alloc: &Option<Alloc>) {
        for bench in benches {
            for iteration in 0..NUM_RUNS {
                alloca::with_alloca(
                    iteration, // we increase the byte offset by 1 for each iteration
                    |_memory: &mut [core::mem::MaybeUninit<u8>]| {
                        bench.exec_bench(alloc);
                        black_box(());
                    },
                );
            }
        }
    }

    fn run_interleaved<'a>(
        benches: &mut [Box<dyn Bench<'a> + 'a>],
        alloc: &Option<Alloc>,
        cache_trasher: Option<&CacheTrasher>,
    ) {
        let mut bench_indices: Vec<usize> = (0..benches.len()).collect();
        for iteration in 0..NUM_RUNS {
            // We interleave the benches to address benchmarking randomness
            //
            // This has the drawback, that one bench will affect another one.
            shuffle(&mut bench_indices, iteration as u64);

            for bench_idx in bench_indices.iter() {
                if let Some(cache_trasher) = cache_trasher {
                    cache_trasher.issue_read();
                }
                let bench = &mut benches[*bench_idx];
                // We use alloca to address memory layout randomness issues
                // So the whole stack moves down by 1 byte for each iteration

                // We loop multiple times on a single bench, since one bench could e.g. flush all the
                // memory caches, which may or may not be like this in a real world environment.
                // We want to capture both cases, hot loops and interleaved, to see how a bench performs under both
                // conditions.
                #[cfg(any(target_family = "unix", target_family = "windows"))]
                {
                    alloca::with_alloca(
                        iteration, // we increase the byte offset by 1 for each iteration
                        |_memory: &mut [core::mem::MaybeUninit<u8>]| {
                            bench.exec_bench(alloc);
                            black_box(());
                        },
                    );
                }
                #[cfg(not(any(target_family = "unix", target_family = "windows")))]
                {
                    black_box(bench.exec_bench(alloc));
                }
            }
        }
    }

    fn warm_up_group_and_set_iter<'b>(
        benches: &mut [Box<dyn Bench<'b> + 'b>],
        num_iter: Option<usize>,
        verbose: bool,
    ) {
        if let Some(num_iter) = num_iter {
            if verbose {
                println!("Manually set num iterations to {}", num_iter);
            }

            for input_and_bench in benches {
                input_and_bench.set_num_iter(num_iter);
            }
            return;
        }
        // In order to make the benchmarks in a group comparable, it is imperative to call them
        // the same numer of times
        let (min_num_iter, max_num_iter) =
            minmax(benches.iter_mut().map(|b| b.sample_num_iter())).unwrap();

        if verbose {
            println!(
                "Estimated iters in group between {} to {}",
                min_num_iter, max_num_iter
            );
        }
        // If the difference between min and max_num_iter is more than 10x, we just set
        // max_num_iter to 10x of min. This is done to avoid having too lon running benchmarks
        let max_num_iter = max_num_iter.min(min_num_iter * 10);
        // We round up, so that we may get the same number of iterations between runs
        let max_num_iter = round_up(max_num_iter as u64) as usize;
        if verbose {
            println!("Set common iterations of {} for group", max_num_iter);
        }

        for input_and_bench in benches {
            input_and_bench.set_num_iter(max_num_iter);
        }
    }
}

// Trying to get a stable number of iterations between runs
fn round_up(num: u64) -> u64 {
    if num == 1 {
        return 1;
    }
    if num == 2 {
        return 2;
    }
    if num < 10 {
        return 10;
    }

    let mut divisor: u64 = 10;
    while num >= divisor * 10 {
        divisor *= 10;
    }

    ((num + divisor - 1) / divisor) * divisor
}

pub fn minmax<I, T>(mut vals: I) -> Option<(T, T)>
where
    I: Iterator<Item = T>,
    T: Copy + PartialOrd,
{
    let first_el = vals.find(|val| {
        // We use this to make sure we skip all NaN values when
        // working with a float type.
        val.partial_cmp(val) == Some(Ordering::Equal)
    })?;
    let mut min_so_far: T = first_el;
    let mut max_so_far: T = first_el;
    for val in vals {
        if val.partial_cmp(&min_so_far) == Some(Ordering::Less) {
            min_so_far = val;
        }
        if val.partial_cmp(&max_so_far) == Some(Ordering::Greater) {
            max_so_far = val;
        }
    }
    Some((min_so_far, max_so_far))
}

/// Performs a dummy reads from memory to spoil given amount of CPU cache
///
/// Uses cache aligned data arrays to perform minimum amount of reads possible to spoil the cache
#[derive(Clone)]
struct CacheTrasher {
    cache_lines: Vec<CacheLine>,
}
impl Default for CacheTrasher {
    fn default() -> Self {
        Self::new(1024 * 1024 * 16) // 16MB
    }
}

impl CacheTrasher {
    fn new(bytes: usize) -> Self {
        let n = bytes / size_of::<CacheLine>();
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

    if indices.len() == 2 {
        indices.swap(0, 1);
        return;
    }

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn function_name_test() {
        let mut bench_indices = vec![0, 1];
        let mut all = Vec::new();
        for iteration in 0..4 {
            // We interleave the benches to address benchmarking randomness
            //
            // This has the drawback, that one bench will affect another one.
            shuffle(&mut bench_indices, iteration as u64);
            all.push(bench_indices.clone());
        }
        assert_eq!(all, vec![vec![1, 0], vec![0, 1], vec![1, 0], vec![0, 1]]);
    }

    #[test]
    fn test_round_up() {
        assert_eq!(round_up(125), 200);
        assert_eq!(round_up(12), 20);
        assert_eq!(round_up(1256), 2000);
        assert_eq!(round_up(78945), 80000);
        assert_eq!(round_up(1000), 1000);
        assert_eq!(round_up(1001), 2000);
        assert_eq!(round_up(999), 1000);
        assert_eq!(round_up(9), 10); // Check for single digit numbers
    }
}
