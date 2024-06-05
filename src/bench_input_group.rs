use std::{alloc::GlobalAlloc, borrow::Cow, mem};

use crate::{
    bench::NamedBench, bench_runner::BenchRunner, parse_args, BenchGroup, Config, NamedInput,
};
use peakmem_alloc::*;

pub(crate) type Alloc = &'static dyn PeakMemAllocTrait;

/// `InputGroup` is a collection of benchmarks that are run with the same inputs.
///
/// It is self-contained and can be run independently.
///
/// The ownership of the inputs is transferred
/// to the `InputGroup`. If this is not possible, use [BenchRunner](crate::BenchRunner) instead.
pub struct InputGroup<I: 'static = ()> {
    inputs: Vec<OwnedNamedInput<I>>,
    benches_per_input: Vec<Vec<NamedBench<'static, I>>>,
    runner: BenchRunner,
}

impl Default for InputGroup<()> {
    fn default() -> Self {
        Self::new()
    }
}

impl InputGroup<()> {
    /// Create a new InputGroup with no inputs.
    pub fn new() -> Self {
        Self::new_with_inputs(vec![("", ())])
    }
}

/// Input
pub struct OwnedNamedInput<I> {
    pub(crate) name: String,
    pub(crate) data: I,
    pub(crate) input_size_in_bytes: Option<usize>,
}

impl<I: 'static> InputGroup<I> {
    /// Sets name of the group and returns the group.
    pub fn name<S: AsRef<str>>(mut self, name: S) -> Self {
        self.runner.set_name(name);
        self
    }
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
    /// Set the peak mem allocator to be used for the benchmarks.
    /// This will report the peak memory consumption of the benchmarks.
    pub fn set_alloc<A: GlobalAlloc + 'static>(&mut self, alloc: &'static PeakMemAlloc<A>) {
        self.runner.set_alloc(alloc);
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
        &mut self.runner.options
    }

    /// Register a benchmark with the given name and function.
    pub fn register<F, S: Into<String>>(&mut self, name: S, fun: F)
    where
        F: Fn(&I) + 'static + Clone,
    {
        let name = name.into();

        for (ord, _) in self.inputs.iter().enumerate() {
            let named_bench: NamedBench<'static, I> =
                NamedBench::new(name.to_string(), Box::new(fun.clone()));

            self.benches_per_input[ord].push(named_bench);
        }
    }

    /// Run the benchmarks and report the results.
    pub fn run(&mut self) {
        for (ord, benches) in self.benches_per_input.iter_mut().enumerate() {
            let input = &self.inputs[ord];
            let mut group = BenchGroup::new(self.runner.clone());
            group.set_name(&input.name);
            benches.reverse();
            while let Some(bench) = benches.pop() {
                let named_input: NamedInput<'_, I> = NamedInput {
                    name: Cow::Borrowed(&input.name),
                    data: &input.data,
                };
                // The input lives in the InputGroup, so we can transmute the lifetime to 'static
                // (probably).
                let named_input: NamedInput<'static, I> =
                    unsafe { transmute_lifetime(named_input) };

                if let Some(input_size) = input.input_size_in_bytes {
                    group.set_input_size(input_size);
                }
                group.register_named_with_input(bench, named_input);
            }
            group.run();
        }
    }
}

unsafe fn transmute_lifetime<I>(input: NamedInput<'_, I>) -> NamedInput<'static, I> {
    mem::transmute(input)
}
