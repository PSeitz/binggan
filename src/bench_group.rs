use std::borrow::Cow;

use crate::{
    bench::{Bench, InputWithBenchmark, NamedBench},
    bench_runner::{BenchRunner, EMPTY_INPUT},
    NamedInput,
};

/// `BenchGroup` is a group of benchmarks run together.
///
pub struct BenchGroup<'a> {
    name: Option<String>,
    pub(crate) benches: Vec<Box<dyn Bench<'a> + 'a>>,
    /// The size of the input.
    /// Enables throughput reporting.
    input_size_in_bytes: Option<usize>,
    pub(crate) runner: BenchRunner,
}

impl<'a> BenchGroup<'a> {
    /// Create a new BenchGroup with no benchmarks.
    pub fn new(runner: BenchRunner) -> Self {
        Self {
            name: None,
            benches: Vec::new(),
            input_size_in_bytes: None,
            runner,
        }
    }

    /// Create a new BenchGroup with no benchmarks.
    pub fn with_name<S: Into<String>>(runner: BenchRunner, name: S) -> Self {
        Self {
            name: Some(name.into()),
            benches: Vec::new(),
            input_size_in_bytes: None,
            runner,
        }
    }

    /// Sets name of the group and returns the group.
    pub fn name<S: Into<String>>(mut self, name: S) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Enables throughput reporting. The throughput will be valid for all inputs that are
    /// registered afterwards.
    pub fn set_input_size(&mut self, input_size: usize) {
        self.input_size_in_bytes = Some(input_size);
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
    pub fn register<I, F, S: Into<String>>(&mut self, name: S, fun: F)
    where
        F: Fn(&'a ()) + 'static,
    {
        let name = name.into();
        let bench = NamedBench::new(name, Box::new(fun));

        self.register_named_with_input(bench, EMPTY_INPUT);
    }

    /// Register a benchmark with the given name and function.
    pub(crate) fn register_named_with_input<I>(
        &mut self,
        bench: NamedBench<'a, I>,
        input: NamedInput<'a, I>,
    ) {
        if let Some(filter) = &self.runner.options.filter {
            if !bench.name.contains(filter) && !input.name.contains(filter) {
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
    pub fn set_name<S: Into<String>>(&mut self, name: S) {
        self.name = Some(name.into());
    }

    /// Run the benchmarks and report the results.
    pub fn run(&mut self) {
        self.runner
            .run_group(self.name.as_deref(), &mut self.benches);
    }
}
