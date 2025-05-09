use std::cmp::Ordering;
use std::env;

use crate::output_value::OutputValue;
use crate::plugins::{EventListener, PluginEvents, PluginManager};
use crate::report::PlainReporter;
use crate::{
    bench::{Bench, InputWithBenchmark, NamedBench},
    bench_id::BenchId,
    black_box, parse_args,
    report::report_group,
    BenchGroup, Config,
};

/// The main struct to run benchmarks.
///
pub struct BenchRunner {
    pub(crate) config: Config,
    /// The size of the input.
    /// Enables throughput reporting.
    input_size_in_bytes: Option<usize>,

    /// Name of the test
    pub(crate) name: Option<String>,

    plugins: PluginManager,
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

    /// Returns the plugin manager, which can be used to add and remove plugins.
    /// See [crate::plugins::PluginManager] for more information.
    pub fn get_plugin_manager(&mut self) -> &mut PluginManager {
        &mut self.plugins
    }

    /// Add a new plugin and returns the PluginManager for chaining.
    pub fn add_plugin<L: EventListener + 'static>(&mut self, listener: L) -> &mut PluginManager {
        self.get_plugin_manager().add_plugin(listener)
    }

    /// Creates a new `BenchRunner` with custom options set.
    pub(crate) fn new_with_options(options: Config) -> Self {
        use yansi::Condition;
        yansi::whenever(Condition::TTY_AND_COLOR);

        let mut plugins = PluginManager::new();
        plugins.add_plugin_if_absent(PlainReporter::new());

        BenchRunner {
            config: options,
            input_size_in_bytes: None,
            name: None,
            plugins,
        }
    }

    /// Creates a new `BenchGroup`
    /// The group is a collection of benchmarks that are run together.
    pub fn new_group(&mut self) -> BenchGroup<'_, '_> {
        BenchGroup::new(self)
    }

    /// Set the name of the current test runner. This is like a header for all tests in in this
    /// runner.
    /// It is also used to distinguish when writing the results to disk.
    pub fn set_name<S: AsRef<str>>(&mut self, name: S) {
        self.name = Some(name.as_ref().to_string());
    }

    /// Enables throughput reporting. The throughput will be valid for all inputs that are
    /// registered afterwards.
    pub fn set_input_size(&mut self, input_size: usize) {
        self.input_size_in_bytes = Some(input_size);
    }

    /// Run a single function. This will directly execute and report the function and therefore does
    /// not support interleaved execution.
    ///
    /// The return value of the function will be reported as the [OutputValue::column_title].
    pub fn bench_function<F, S: Into<String>, O: OutputValue>(&mut self, name: S, f: F) -> &mut Self
    where
        F: Fn(&()) -> O + 'static,
    {
        let bench_id = BenchId::from_bench_name(name).runner_name(self.name.as_deref());
        let named_bench = NamedBench::new(
            bench_id,
            Box::new(f),
            self.config().get_num_iter_for_group(),
        );
        let bundle = InputWithBenchmark::new(
            EMPTY_INPUT,
            self.input_size_in_bytes,
            named_bench,
            self.config().num_iter_bench,
        );

        self.run_group(None, &mut [Box::new(bundle)], O::column_title());
        self
    }

    /// Configure the benchmark.
    ///
    /// See the [Config] struct for more information.
    pub fn config(&mut self) -> &mut Config {
        &mut self.config
    }

    /// Run the benchmarks and report the results.
    pub fn run_group<'a>(
        &mut self,
        group_name: Option<&str>,
        group: &mut [Box<dyn Bench<'a> + 'a>],
        output_value_column_title: &'static str,
    ) {
        if group.is_empty() {
            return;
        }

        self.plugins.emit(PluginEvents::GroupStart {
            runner_name: self.name.as_deref(),
            group_name,
            output_value_column_title,
        });

        const MAX_GROUP_SIZE: usize = 5;
        if self.config.verbose && group.len() > MAX_GROUP_SIZE {
            println!(
                "Group is quite big, splitting into chunks of {} elements",
                MAX_GROUP_SIZE
            );
        }

        let num_group_iter = self.config.get_num_iter_for_group();
        self.get_plugin_manager().emit(PluginEvents::GroupNumIters {
            num_iter: num_group_iter,
        });
        // If the group is quite big, we don't want to create too big chunks, which causes
        // slow tests, therefore a chunk is at most 5 elements large.
        for group in group.chunks_mut(MAX_GROUP_SIZE) {
            Self::detect_and_set_num_iter(group, self.config.verbose, &mut self.plugins);

            if self.config.interleave {
                Self::run_interleaved(group, num_group_iter, &mut self.plugins);
            } else {
                Self::run_sequential(group, num_group_iter, &mut self.plugins);
            }
        }

        report_group(
            self.name.as_deref(),
            group_name,
            group,
            output_value_column_title,
            &mut self.plugins,
        );

        // TODO: clearing should be optional, to check the results yourself, e.g. in CI
        //for bench in group {
        //bench.clear_results();
        //}
    }

    fn run_sequential<'a>(
        benches: &mut [Box<dyn Bench<'a> + 'a>],
        num_group_iter: usize,
        plugins: &mut PluginManager,
    ) {
        for bench in benches {
            for iteration in 0..num_group_iter {
                alloca::with_alloca(
                    iteration, // we increase the byte offset by 1 for each iteration
                    |_memory: &mut [core::mem::MaybeUninit<u8>]| {
                        bench.exec_bench(plugins);
                        black_box(());
                    },
                );
            }
        }
    }

    fn run_interleaved<'a>(
        benches: &mut [Box<dyn Bench<'a> + 'a>],
        num_group_iter: usize,
        plugins: &mut PluginManager,
    ) {
        let mut bench_indices: Vec<usize> = (0..benches.len()).collect();
        for iteration in 0..num_group_iter {
            // We interleave the benches to address benchmarking randomness
            //
            // This has the drawback, that one bench will affect another one.
            shuffle(&mut bench_indices, iteration as u64);

            for bench_idx in bench_indices.iter() {
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
                            bench.exec_bench(plugins);
                            black_box(());
                        },
                    );
                }
                #[cfg(not(any(target_family = "unix", target_family = "windows")))]
                {
                    black_box(bench.exec_bench(plugins));
                }
            }
        }
    }

    /// Detect how often each bench should be run if it is not set manually.
    fn detect_and_set_num_iter<'b>(
        benches: &mut [Box<dyn Bench<'b> + 'b>],
        verbose: bool,
        plugins: &mut PluginManager,
    ) {
        if let Some(num_iter) = env::var("NUM_ITER_BENCH")
            .ok()
            .and_then(|val| val.parse::<usize>().ok())
        {
            for input_and_bench in benches {
                input_and_bench.set_num_iter(num_iter, plugins);
            }
            plugins.emit(PluginEvents::GroupBenchNumIters { num_iter });

            return;
        }

        // Filter benches that already have num_iter set
        let mut benches = {
            let filtered: Vec<_> = benches
                .iter_mut()
                .filter(|b| b.get_num_iter().is_none())
                .collect::<Vec<_>>();
            if filtered.is_empty() {
                plugins.emit(PluginEvents::GroupBenchNumIters {
                    num_iter: benches[0].get_num_iter().unwrap(),
                });
                return;
            }
            filtered
        };

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
        // max_num_iter to 10x of min. This is done to avoid having too long running benchmarks
        let max_num_iter = max_num_iter.min(min_num_iter * 10);
        // We round up, so that we may get the same number of iterations between runs
        let max_num_iter = round_up(max_num_iter as u64) as usize;
        plugins.emit(PluginEvents::GroupBenchNumIters {
            num_iter: max_num_iter,
        });
        if verbose {
            println!("Set common iterations of {} for group", max_num_iter);
        }

        for input_and_bench in benches {
            input_and_bench.set_num_iter(max_num_iter, plugins);
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

    num.div_ceil(divisor) * divisor
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
