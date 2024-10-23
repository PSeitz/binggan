use crate::{
    bench::{Bench, InputWithBenchmark, NamedBench},
    bench_id::BenchId,
    bench_runner::BenchRunner,
    output_value::OutputValue,
};

/// `BenchGroup` is a group of benchmarks wich are executed together.
///
pub struct BenchGroup<'a, 'runner> {
    group_name: Option<String>,
    pub(crate) benches: Vec<Box<dyn Bench<'a> + 'a>>,
    /// The size of the input.
    /// Enables throughput reporting.
    input_size_in_bytes: Option<usize>,
    pub(crate) runner: &'runner mut BenchRunner,
    pub(crate) output_value_column_title: &'static str,
}

impl<'a, 'runner> BenchGroup<'a, 'runner> {
    /// Create a new BenchGroup with no benchmarks.
    pub fn new(runner: &'runner mut BenchRunner) -> Self {
        Self {
            group_name: None,
            benches: Vec::new(),
            input_size_in_bytes: None,
            runner,
            output_value_column_title: "Output",
        }
    }

    /// Sets name of the group and returns the group.
    pub fn name<S: Into<String>>(mut self, name: S) -> Self {
        self.group_name = Some(name.into());
        self
    }

    /// Set the name of the group.
    /// The name is printed before the benchmarks are run.
    /// It is also used to distinguish when writing the results to disk.
    pub fn set_name<S: AsRef<str>>(&mut self, name: S) {
        self.group_name = Some(name.as_ref().into());
    }

    /// Enables throughput reporting. The `input_size` will be used for all benchmarks that are
    /// registered afterwards.
    pub fn set_input_size(&mut self, input_size: usize) {
        self.input_size_in_bytes = Some(input_size);
    }

    /// Register a benchmark with the given name, function and input.
    ///
    /// The return value of the function will be reported as the `OutputValue`
    pub fn register_with_input<I, F, S: Into<String>, O: OutputValue + 'static>(
        &mut self,
        bench_name: S,
        input: &'a I,
        fun: F,
    ) where
        F: Fn(&'a I) -> O + 'a,
    {
        let bench = NamedBench::new(
            self.get_bench_id(bench_name.into()),
            Box::new(fun),
            self.runner.config().get_num_iter_for_group(),
        );
        self.register_named_with_input(bench, input);
    }

    /// Register a benchmark with the given name and function.
    ///
    /// The return value of the function will be reported as the `OutputValue`.
    pub fn register<F, S: Into<String>, O: OutputValue + 'static>(&mut self, bench_name: S, fun: F)
    where
        F: Fn(&'a ()) -> O + 'static,
    {
        let bench_name = bench_name.into();
        let bench = NamedBench::new(
            self.get_bench_id(bench_name),
            Box::new(fun),
            self.runner.config().get_num_iter_for_group(),
        );

        self.register_named_with_input(bench, &());
    }

    fn get_bench_id(&self, bench_name: String) -> BenchId {
        BenchId::from_bench_name(bench_name)
            .runner_name(self.runner.name.as_deref())
            .group_name(self.group_name.clone())
    }

    /// Register a benchmark with the given name and function.
    pub(crate) fn register_named_with_input<I, O: OutputValue + 'static>(
        &mut self,
        bench: NamedBench<'a, I, O>,
        input: &'a I,
    ) {
        self.output_value_column_title = O::column_title();
        if let Some(filter) = &self.runner.config.filter {
            let bench_id = bench.bench_id.get_full_name();

            if !bench_id.contains(filter) {
                return;
            }
        }

        let bundle = InputWithBenchmark::new(
            input,
            self.input_size_in_bytes,
            bench,
            self.runner.config.num_iter_bench,
        );

        self.benches.push(Box::new(bundle));
    }

    /// Run the benchmarks and report the results.
    pub fn run(&mut self) {
        self.runner.run_group(
            self.group_name.as_deref(),
            &mut self.benches,
            self.output_value_column_title,
        )
    }
}
