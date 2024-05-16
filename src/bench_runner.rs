use std::{alloc::GlobalAlloc, borrow::Cow, cmp::Ordering};

use crate::{
    bench::{Bench, InputWithBenchmark, NamedBench},
    black_box, parse_args,
    report::report_group,
    Options,
};
use peakmem_alloc::*;
use yansi::Paint;

pub(crate) type Alloc = &'static dyn PeakMemAllocTrait;
pub(crate) const NUM_RUNS: usize = 32;

/// The main struct to create benchmarks.
///
/// BenchRunner is a collection of benchmarks.
/// It is self-contained and can be run independently.
pub struct BenchRunner<'a> {
    /// Name of the benchmark group.
    name: Option<String>,
    pub(crate) benches: Vec<Box<dyn Bench<'a> + 'a>>,
    alloc: Option<Alloc>,
    cache_trasher: CacheTrasher,
    pub(crate) options: Options,
    /// The size of the input.
    /// Enables throughput reporting.
    input_size_in_bytes: Option<usize>,
    /// Manully set the number of iterations each benchmark is called.
    ///
    /// This disables the automatic detection of the number of iterations.
    num_iter: Option<usize>,
}

/// Input
#[derive(Debug, Clone)]
pub struct NamedInput<'a, I> {
    pub(crate) name: Cow<'a, str>,
    pub(crate) data: &'a I,
}

const EMPTY_INPUT: NamedInput<()> = NamedInput {
    name: Cow::Borrowed(""),
    data: &(),
};

impl<'a> Default for BenchRunner<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> BenchRunner<'a> {
    /// The inputs are a vector of tuples, where the first element is the name of the input and the
    /// second element is the input itself.
    pub fn new() -> Self {
        Self::new_with_options(parse_args())
    }
    /// The inputs are a vector of tuples, where the first element is the name of the input and the
    /// second element is the input itself.
    pub(crate) fn new_with_options(options: Options) -> Self {
        use yansi::Condition;
        yansi::whenever(Condition::TTY_AND_COLOR);

        BenchRunner {
            benches: Vec::new(),
            cache_trasher: CacheTrasher::new(1024 * 1024 * 16),
            options,
            alloc: None,
            name: None,
            input_size_in_bytes: None,
            num_iter: None,
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

    /// Enables throughput reporting. The throughput will be valid for all inputs that are
    /// registered afterwards.
    pub fn set_input_size(&mut self, input_size: usize) {
        self.input_size_in_bytes = Some(input_size);
    }

    /// Manully set the number of iterations each benchmark is called.
    ///
    /// This disables the automatic detection of the number of iterations.
    pub fn set_num_iter(&mut self, num_iter: usize) {
        self.num_iter = Some(num_iter);
    }

    /// Set the name of the group.
    /// The name is printed before the benchmarks are run.
    /// It is also used to distinguish when writing the results to disk.
    pub fn set_name<S: Into<String>>(&mut self, name: S) {
        self.name = Some(name.into());
    }

    /// Set the options to the given value.
    /// This will overwrite all current options.
    ///
    /// See the Options struct for more information.
    pub fn set_options(&mut self, options: Options) {
        self.options = options;
    }

    /// Sets the interleave option to the given value.
    ///
    /// Interleave will run the benchmarks in an interleaved fashion.
    /// Otherwise the benchmarks will be run sequentially.
    /// Interleaving will help to better compare the benchmarks.
    pub fn set_interleave(&mut self, interleave: bool) {
        self.options.interleave = interleave;
    }

    /// Sets the filter, which is used to filter the benchmarks by name.
    /// The filter is fetched from the command line arguments.
    ///
    /// It can also match an input name.
    ///
    /// By default this is parsed from the command line arguments.
    pub fn set_filter(&mut self, filter: Option<String>) {
        self.options.filter = filter;
    }

    /// Register a benchmark with the given name and function.
    pub fn register_with_input<I, F, S: Into<String>>(
        &mut self,
        bench_name: S,
        input_name: S,
        input: &'a I,
        fun: F,
    ) where
        F: Fn(&'a I) + 'static,
    {
        let name = bench_name.into();
        let input_name = input_name.into();

        let bench = NamedBench::new(name, Box::new(fun));
        self.register_named_with_input(
            bench,
            NamedInput {
                name: Cow::Owned(input_name),
                data: input,
            },
        );
    }
    /// Register a benchmark with the given name and function.
    pub(crate) fn register_named_with_input<I>(
        &mut self,
        bench: NamedBench<'a, I>,
        input: NamedInput<'a, I>,
    ) {
        if let Some(filter) = &self.options.filter {
            if !bench.name.contains(filter) && !input.name.contains(filter) {
                return;
            }
        }

        let bundle = InputWithBenchmark::new(
            input,
            self.input_size_in_bytes,
            bench,
            self.options.enable_perf,
        );

        self.benches.push(Box::new(bundle));
    }
    /// Register a benchmark with the given name and function.
    pub fn register<I, F, S: Into<String>>(&mut self, name: S, fun: F)
    where
        F: Fn(&'a ()) + 'static,
    {
        let name = name.into();
        let bench = NamedBench::new(name, Box::new(fun));
        self.register_named_with_input(bench, EMPTY_INPUT);
    }

    /// Trash CPU cache between bench runs. Defaults to false.
    pub fn set_cache_trasher(&mut self, enable: bool) {
        self.options.cache_trasher = enable;
    }

    /// Run the benchmarks and report the results.
    pub fn run(&mut self) {
        if self.benches.is_empty() {
            return;
        }

        if let Some(name) = &self.name {
            println!("{}", name.black().on_red().invert().bold());
        }

        // TODO: group by should be configurable
        group_by_mut(
            &mut self.benches,
            |b| b.get_input_name(),
            |group| {
                let input_name = group[0].get_input_name();
                if !input_name.is_empty() {
                    println!("{}", input_name.black().on_yellow().invert().italic());
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
                    Self::warm_up_group_and_set_iter(group, self.num_iter, self.options.verbose);

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
                // We report at the end, so the alignment is correct (could be calculated up front)
                report_group(&self.name, group, self.alloc.is_some());
            },
        );

        self.clear_results();
    }

    /// Clear the stored results of the benchmarks.
    pub fn clear_results(&mut self) {
        for bench in &mut self.benches {
            bench.clear_results();
        }
    }

    fn run_sequential(benches: &mut [Box<dyn Bench<'a> + 'a>], alloc: &Option<Alloc>) {
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

    fn run_interleaved(
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
            //std::thread::yield_now();

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

    fn warm_up_group_and_set_iter(
        benches: &mut [Box<dyn Bench<'a> + 'a>],
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
            minmax(benches.iter().map(|b| b.sample_num_iter())).unwrap();

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

/// Note: The data will be sorted.
///
/// Returns slices of the input data grouped by passed closure.
pub fn group_by_mut<T, K: Ord + ?Sized, F>(
    mut data: &mut [T],
    compare_by: impl Fn(&T) -> &K,
    mut callback: F,
) where
    F: FnMut(&mut [T]),
{
    while !data.is_empty() {
        let last_element = data.last().unwrap();
        let count = data
            .iter()
            .rev()
            .take_while(|&x| compare_by(x) == compare_by(last_element))
            .count();

        let (rest, group) = data.split_at_mut(data.len() - count);
        data = rest;
        callback(group);
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
        Self::new(1024 * 1024 * 16) // 16MB
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