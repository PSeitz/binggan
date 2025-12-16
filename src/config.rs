use rustop::opts;

/// Configure the benchmarking options.
#[derive(Debug, Clone)]
pub struct Config {
    /// Interleave benchmarks
    pub interleave: bool,
    /// The filter for the benchmarks
    /// This is read from the command line by default.
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
}

impl Default for Config {
    fn default() -> Self {
        // Check ENV for verbose
        let verbose = std::env::var("BINGGAN_VERBOSE").is_ok();
        Config {
            interleave: true,
            filter: None,
            verbose,
            num_iter_bench: None,
            num_iter_group: None,
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
    pub fn get_num_iter_for_group(&self) -> usize {
        self.num_iter_group.unwrap_or(32)
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
            filter: args.filter,
            ..Default::default()
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
