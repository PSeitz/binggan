use std::{alloc::GlobalAlloc, mem};

use crate::output_value::OutputValue;
use crate::{
    bench::NamedBench, bench_id::BenchId, bench_runner::BenchRunner, parse_args, report::Reporter,
    BenchGroup, Config,
};
use peakmem_alloc::*;

pub(crate) type Alloc = &'static dyn PeakMemAllocTrait;

/// `InputGroup` is a collection of benchmarks that are run with the same inputs.
///
/// It is self-contained and can be run independently.
///
/// The ownership of the inputs is transferred to the `InputGroup`.
/// If this is not possible, use [BenchRunner](crate::BenchRunner) instead.
pub struct InputGroup<I: 'static = (), O = ()> {
    inputs: Vec<OwnedNamedInput<I>>,
    benches_per_input: Vec<Vec<NamedBench<'static, I, O>>>,
    runner: BenchRunner,
}

impl Default for InputGroup<()> {
    fn default() -> Self {
        Self::new()
    }
}

impl InputGroup<(), ()> {
    /// Create a new InputGroup with no inputs.
    pub fn new() -> Self {
        Self::new_with_inputs(vec![("", ())])
    }
}

/// Bundles data with some name and its input_size_in_bytes.
pub struct OwnedNamedInput<I> {
    pub(crate) name: String,
    pub(crate) data: I,
    pub(crate) input_size_in_bytes: Option<usize>,
}

impl<I: 'static, O: OutputValue + 'static> InputGroup<I, O> {
    /// The inputs are a vector of tuples, where the first element is the name of the input and the
    /// second element is the input itself.
    pub fn new_with_inputs<S: Into<String>>(inputs: Vec<(S, I)>) -> Self {
        Self::new_with_inputs_and_options(inputs, parse_args())
    }
    /// The inputs are a vector of tuples, where the first element is the name of the input and the
    /// second element is the input itself.
    pub(crate) fn new_with_inputs_and_options<S: Into<String>>(
        inputs: Vec<(S, I)>,
        options: Config,
    ) -> Self {
        use yansi::Condition;
        yansi::whenever(Condition::TTY_AND_COLOR);

        let inputs: Vec<OwnedNamedInput<I>> = inputs
            .into_iter()
            .map(|(name, input)| OwnedNamedInput {
                name: name.into(),
                data: input,
                input_size_in_bytes: None,
            })
            .collect();
        let runner = BenchRunner::new_with_options(options);
        let mut benches_per_input = Vec::new();
        // Can't use resize because of clone
        for _input in &inputs {
            benches_per_input.push(Vec::new());
        }

        InputGroup {
            inputs,
            runner,
            benches_per_input,
        }
    }

    /// Enables throughput reporting.
    /// The passed closure should return the size of the input in bytes.
    pub fn throughput<F>(&mut self, f: F)
    where
        F: Fn(&I) -> usize + 'static,
    {
        for input in &mut self.inputs {
            input.input_size_in_bytes = Some(f(&input.data));
        }
    }

    /// Register a benchmark with the given name and function.
    ///
    /// The return value of the function will be reported as the `OutputValue` if it is `Some`.
    pub fn register<F, S: Into<String>>(&mut self, name: S, fun: F)
    where
        F: Fn(&I) -> Option<O> + 'static + Clone,
    {
        let name = name.into();

        let num_iter_for_group = self.config().get_num_iter_for_group();
        for (ord, input) in self.inputs.iter().enumerate() {
            let bench_id = BenchId::from_bench_name(name.clone())
                .runner_name(self.runner.name.as_deref())
                .group_name(Some(input.name.clone()));
            let named_bench: NamedBench<'static, I, O> =
                NamedBench::new(bench_id, Box::new(fun.clone()), num_iter_for_group);

            self.benches_per_input[ord].push(named_bench);
        }
    }

    /// Run the benchmarks and report the results.
    pub fn run(&mut self) {
        for (ord, benches) in self.benches_per_input.iter_mut().enumerate() {
            let input = &self.inputs[ord];
            let mut group = BenchGroup::new(self.runner.clone());
            group.set_name(&input.name);
            // reverse so we can use pop and keep the order
            benches.reverse();
            while let Some(bench) = benches.pop() {
                // The input lives in the InputGroup, so we can transmute the lifetime to 'static
                // (probably).
                let extended_input = unsafe { transmute_lifetime(&input.data) };

                if let Some(input_size) = input.input_size_in_bytes {
                    group.set_input_size(input_size);
                }
                group.register_named_with_input(bench, extended_input);
            }
            group.run();
        }
    }

    // Expose runner methods
    /// Set the peak mem allocator to be used for the benchmarks.
    /// This will report the peak memory consumption of the benchmarks.
    pub fn set_alloc<A: GlobalAlloc + 'static>(&mut self, alloc: &'static PeakMemAlloc<A>) {
        self.runner.set_alloc(alloc);
    }

    /// Set the name of the group.
    /// The name is printed before the benchmarks are run.
    /// It is also used to distinguish when writing the results to disk.
    pub fn set_name<S: AsRef<str>>(&mut self, name: S) {
        self.runner.set_name(name);
    }

    /// Configure the benchmarking options.
    ///
    /// See the [Config] struct for more information.
    pub fn config(&mut self) -> &mut Config {
        &mut self.runner.config
    }

    /// Set the reporter to be used for the benchmarks. See [Reporter] for more information.
    pub fn set_reporter<R: Reporter + 'static>(&mut self, reporter: R) {
        self.runner.set_reporter(reporter);
    }
}

unsafe fn transmute_lifetime<I>(input: &I) -> &'static I {
    mem::transmute(input)
}
