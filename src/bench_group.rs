use crate::{
    bench::{Bench, BenchResult, InputWithBenchmark, NamedBench},
    bench_id::{BenchId, PrintOnce},
    bench_runner::BenchRunner,
    output_value::OutputValue,
};

/// `BenchGroup` is a group of benchmarks wich are executed together.
///
pub struct BenchGroup<'a> {
    bench_runner_name: Option<PrintOnce>,
    group_name: Option<String>,
    pub(crate) benches: Vec<Box<dyn Bench<'a> + 'a>>,
    /// The size of the input.
    /// Enables throughput reporting.
    input_size_in_bytes: Option<usize>,
    pub(crate) runner: BenchRunner,
    pub(crate) coutput_value_column_title: &'static str,
}

impl<'a> BenchGroup<'a> {
    /// Create a new BenchGroup with no benchmarks.
    pub fn new(runner: BenchRunner) -> Self {
        Self {
            group_name: None,
            bench_runner_name: runner.name.to_owned(),
            benches: Vec::new(),
            input_size_in_bytes: None,
            runner,
            coutput_value_column_title: "Output",
        }
    }

    /// Create a new BenchGroup with no benchmarks.
    pub fn with_name<S: Into<String>>(runner: BenchRunner, name: S) -> Self {
        Self {
            group_name: Some(name.into()),
            bench_runner_name: runner.name.to_owned(),
            benches: Vec::new(),
            input_size_in_bytes: None,
            runner,
            coutput_value_column_title: "Output",
        }
    }

    /// Sets name of the group and returns the group.
    pub fn name<S: Into<String>>(mut self, name: S) -> Self {
        self.group_name = Some(name.into());
        self
    }

    /// Enables throughput reporting. The `input_size` will be used for all benchmarks that are
    /// registered afterwards.
    pub fn set_input_size(&mut self, input_size: usize) {
        self.input_size_in_bytes = Some(input_size);
    }

    /// Register a benchmark with the given name, function and input.
    ///
    /// The return value of the function will be reported as the `OutputValue` if it is `Some`.
    pub fn register_with_input<I, F, S: Into<String>, O: OutputValue + 'static>(
        &mut self,
        bench_name: S,
        input: &'a I,
        fun: F,
    ) where
        F: Fn(&'a I) -> Option<O> + 'static,
    {
        let bench = NamedBench::new(self.get_bench_id(bench_name.into()), Box::new(fun));
        self.register_named_with_input(bench, input);
    }

    /// Register a benchmark with the given name and function.
    ///
    /// The return value of the function will be reported as the `OutputValue` if it is `Some`.
    pub fn register<I, F, S: Into<String>, O: OutputValue + 'static>(
        &mut self,
        bench_name: S,
        fun: F,
    ) where
        F: Fn(&'a ()) -> Option<O> + 'static,
    {
        let bench_name = bench_name.into();
        let bench = NamedBench::new(self.get_bench_id(bench_name), Box::new(fun));

        self.register_named_with_input(bench, &());
    }

    fn get_bench_id(&self, bench_name: String) -> BenchId {
        BenchId::from_bench_name(bench_name)
            .runner_name(self.bench_runner_name.as_deref())
            .group_name(self.group_name.clone())
    }

    /// Register a benchmark with the given name and function.
    pub(crate) fn register_named_with_input<I, O: OutputValue + 'static>(
        &mut self,
        bench: NamedBench<'a, I, O>,
        input: &'a I,
    ) {
        if let Some(filter) = &self.runner.options.filter {
            let bench_id = bench.bench_id.get_full_name();

            if !bench_id.contains(filter) {
                return;
            }
        }

        let bundle = InputWithBenchmark::new(
            input,
            self.input_size_in_bytes,
            bench,
            self.runner.options.enable_perf,
        );

        self.benches.push(Box::new(bundle));
    }

    /// Set the name of the group.
    /// The name is printed before the benchmarks are run.
    /// It is also used to distinguish when writing the results to disk.
    pub fn set_name<S: AsRef<str>>(&mut self, name: S) {
        self.group_name = Some(name.as_ref().into());
    }

    /// Run the benchmarks and report the results.
    pub fn run(&mut self) -> Vec<BenchResult> {
        self.runner.run_group(
            self.group_name.as_deref(),
            &mut self.benches,
            self.coutput_value_column_title,
        )
    }
}
