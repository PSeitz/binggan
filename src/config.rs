use rustop::opts;

/// Configure the benchmarking options.
#[derive(Debug, Clone)]
pub struct Config {
    /// Interleave benchmarks
    pub interleave: bool,
    /// Filter should match exact
    pub exact: bool,
    /// The filter for the benchmarks
    /// This is read from the command line by default.
    pub filter: Option<String>,
    /// Enable/disable perf integration
    pub enable_perf: bool,
    /// Trash CPU cache between bench runs.
    pub cache_trasher: bool,
    /// Verbose output of binggan. Prints the number of iterations.
    pub verbose: bool,
    /// Manully set the number of iterations each benchmark is called.
    ///
    /// This disables the automatic detection of the number of iterations.
    pub num_iter: Option<usize>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            interleave: true,
            exact: false,
            filter: None,
            enable_perf: false,
            cache_trasher: false,
            verbose: false,
            num_iter: None,
        }
    }
}

impl Config {
    /// Parses the command line arguments to get the options.
    pub fn new() -> Self {
        parse_args()
    }

    /// Manully set the number of iterations each benchmark is called.
    ///
    /// This disables the automatic detection of the number of iterations.
    ///
    /// # Note
    /// Use this to get more stable and comparable benchmark results, as the number of
    /// iterations has a big impact on measurement and the iteration detection may
    /// not always get the same num iterations between runs. There are ways implemented
    /// to mitigate that but they are limited.
    pub fn set_num_iter(&mut self, num_iter: usize) -> &mut Self {
        self.num_iter = Some(num_iter);
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
    ///
    /// # Note:
    /// This is only available on Linux. On other OSs this uses `dummy_profiler`, which does nothing.
    pub fn enable_perf(&mut self) -> &mut Self {
        self.enable_perf = true;
        self
    }

    /// Trash CPU cache between bench runs. Defaults to false.
    pub fn set_cache_trasher(&mut self, enable: bool) -> &mut Self {
        self.cache_trasher = enable;
        self
    }
}

pub(crate) fn parse_args() -> Config {
    let res = opts! {
        synopsis "";
        opt bench:bool, desc:"bench flag passed by rustc";
        opt interleave:bool=true, desc:"The benchmarks run interleaved by default, i.e. one iteration of each bench after another
                         This may lead to better results, it may also lead to worse results.
                         It very much depends on the benches and the environment you would like to simulate. ";
        opt exact:bool, desc:"Filter benchmarks by exact name rather than by pattern.";
        param filter:Option<String>, desc:"run only bench containing name."; // an optional positional parameter
    }
    .parse();
    if let Ok((args, _rest)) = res {
        Config {
            interleave: args.interleave,
            exact: args.exact,
            filter: args.filter,
            ..Default::default()
        }
    } else if let Err(rustop::Error::Help(help)) = res {
        println!("{}", help);
        std::process::exit(0);
    } else if let Err(e) = res {
        println!("{}", e);
        std::process::exit(1);
    } else {
        unreachable!();
    }
}
