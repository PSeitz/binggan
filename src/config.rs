use rustop::opts;

/// Configure the benchmarking options.
#[derive(Debug, Clone)]
pub struct Config {
    /// Interleave benchmarks
    pub interleave: bool,
    /// The filter for the benchmarks
    /// This is read from the command line by default.
    /// Supports tantivy query grammar like `bench_name:my_bench AND group_name:my_group`
    pub filter: Option<String>,
    /// Verbose output of binggan. Prints the number of iterations.
    pub verbose: bool,
    /// Manually set the number of iterations the benchmarks registered afterwards are called.
    ///
    /// This disables the automatic detection of the number of iterations.
    pub num_iter_bench: Option<usize>,
    /// Manually set the number of iterations the benchmark group is run.
    ///
    pub num_iter_group: Option<usize>,
    /// Adjust duration by subtracting time the thread was not scheduled (Linux only).
    /// Intended for single-threaded, single-benchmark runs.
    /// Assumes a single thread is doing work during the measurement.
    pub adjust_for_single_threaded_cpu_scheduling: bool,
}

impl Default for Config {
    fn default() -> Self {
        // Check ENV for verbose
        let verbose = std::env::var("BINGGAN_VERBOSE").is_ok();
        let filter = std::env::var("BINGGAN_FILTER").ok();
        Config {
            interleave: true,
            filter,
            verbose,
            num_iter_bench: None,
            num_iter_group: None,
            adjust_for_single_threaded_cpu_scheduling: false,
        }
    }
}

impl Config {
    /// Parses the command line arguments to get the options.
    pub fn new() -> Self {
        parse_args()
    }

    /// Manully set the number of iterations the benchmarks registered afterwards are called.
    ///
    /// This disables the automatic detection of the number of iterations.
    ///
    /// # Note
    /// Use this to get more stable and comparable benchmark results, as the number of
    /// iterations has a big impact on measurement and the iteration detection may
    /// not always get the same num iterations between runs. There are ways implemented
    /// to mitigate that but they are limited.
    pub fn set_num_iter_for_bench(&mut self, num_iter: usize) -> &mut Self {
        self.num_iter_bench = Some(num_iter);
        self
    }

    /// Returns the number of iterations for the group.
    ///
    /// If the `NUM_ITER_GROUP` environment variable is set, it takes precedence.
    pub fn get_num_iter_for_group(&self) -> usize {
        num_iter_from_env("NUM_ITER_GROUP")
            .or(self.num_iter_group)
            .unwrap_or(32)
    }

    /// Manully set the number of iterations the benchmark group is run.
    ///
    /// The benchmarks in a group are interleaved for more stable results.
    /// For long running benchmarks that may not be desirable.
    pub fn set_num_iter_for_group(&mut self, num_iter: usize) -> &mut Self {
        self.num_iter_group = Some(num_iter);
        self
    }

    /// Set the options to the given value.
    /// This will overwrite all current options.
    ///
    /// See the Options struct for more information.
    pub fn set_config(&mut self, options: Config) -> &mut Self {
        *self = options;
        self
    }

    /// Interleave will run the benchmarks in an interleaved fashion.
    /// Otherwise the benchmarks will be run sequentially.
    ///
    /// Interleaving may help to get more stable comparisons between benchmarks.
    pub fn set_interleave(&mut self, interleave: bool) -> &mut Self {
        self.interleave = interleave;
        self
    }

    /// Adjust duration by subtracting time the thread was not scheduled (Linux only).
    /// Intended for single-threaded, single-benchmark runs.
    /// Assumes a single thread is doing work during the measurement.
    pub fn set_adjust_for_single_threaded_cpu_scheduling(&mut self, enabled: bool) -> &mut Self {
        self.adjust_for_single_threaded_cpu_scheduling = enabled;
        self
    }
}

/// Parses a benchmark iteration override from an environment variable.
pub(crate) fn num_iter_from_env(var_name: &str) -> Option<usize> {
    std::env::var(var_name)
        .ok()
        .and_then(|val| val.parse::<usize>().ok())
}

pub(crate) fn parse_args() -> Config {
    let res = opts! {
        synopsis "";
        opt bench:bool, desc:"bench flag passed by rustc";
        opt interleave:bool=true, desc:"The benchmarks run interleaved by default, i.e. one iteration of each bench after another
                         This may lead to better results, it may also lead to worse results.
                         It very much depends on the benches and the environment you would like to simulate. ";
        opt exact:bool, desc:"Filter benchmarks by exact name rather than by pattern.";
        param filter:Option<String>, desc:"run only bench matching filter. Supports AND/OR and fields like runner_name, group_name, bench_name."; // an optional positional parameter
    }
    .parse();
    if let Ok((args, _rest)) = res {
        let default_config = Config::default();
        Config {
            interleave: args.interleave,
            filter: args.filter.or(default_config.filter),
            ..default_config
        }
    } else if let Err(rustop::Error::Help(help)) = res {
        println!("{}", help);
        std::process::exit(0);
    } else if let Err(e) = res {
        println!("{}", e);
        Config::default()
    } else {
        unreachable!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{LazyLock, Mutex};

    static ENV_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

    struct EnvVarGuard {
        key: &'static str,
        original_value: Option<String>,
    }

    impl EnvVarGuard {
        fn set(key: &'static str, value: &str) -> Self {
            let original_value = std::env::var(key).ok();
            // SAFETY: The tests serialize environment mutations with ENV_LOCK and restore the
            // previous state before releasing the lock.
            unsafe { std::env::set_var(key, value) };
            Self {
                key,
                original_value,
            }
        }

        fn unset(key: &'static str) -> Self {
            let original_value = std::env::var(key).ok();
            // SAFETY: The tests serialize environment mutations with ENV_LOCK and restore the
            // previous state before releasing the lock.
            unsafe { std::env::remove_var(key) };
            Self {
                key,
                original_value,
            }
        }
    }

    impl Drop for EnvVarGuard {
        fn drop(&mut self) {
            // SAFETY: The tests serialize environment mutations with ENV_LOCK and restore the
            // previous state before releasing the lock.
            unsafe {
                if let Some(value) = &self.original_value {
                    std::env::set_var(self.key, value);
                } else {
                    std::env::remove_var(self.key);
                }
            }
        }
    }

    #[test]
    fn num_iter_group_env_overrides_config() {
        let _env_lock = ENV_LOCK.lock().unwrap();
        let _env_guard = EnvVarGuard::set("NUM_ITER_GROUP", "17");

        let mut config = Config::default();
        config.set_num_iter_for_group(9);

        assert_eq!(config.get_num_iter_for_group(), 17);
    }

    #[test]
    fn invalid_num_iter_group_env_falls_back_to_config() {
        let _env_lock = ENV_LOCK.lock().unwrap();
        let _env_guard = EnvVarGuard::set("NUM_ITER_GROUP", "invalid");

        let mut config = Config::default();
        config.set_num_iter_for_group(9);

        assert_eq!(config.get_num_iter_for_group(), 9);
    }

    #[test]
    fn num_iter_group_defaults_to_32() {
        let _env_lock = ENV_LOCK.lock().unwrap();
        let _env_guard = EnvVarGuard::unset("NUM_ITER_GROUP");

        assert_eq!(Config::default().get_num_iter_for_group(), 32);
    }
}
