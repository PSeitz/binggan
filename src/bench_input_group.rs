use std::{alloc::GlobalAlloc, borrow::Cow, collections::HashMap};

use crate::{
    bench::NamedBench,
    bench_runner::{group_by_mut, BenchRunner},
    parse_args, BenchGroup, NamedInput, Options,
};
use peakmem_alloc::*;
use yansi::Paint;

pub(crate) type Alloc = &'static dyn PeakMemAllocTrait;

/// `InputGroup` is a collection of benchmarks that are run with the same inputs.
///
/// It is self-contained and can be run independently.
///
/// The ownership of the inputs is transferred
/// to the `InputGroup`. If this is not possible, use [BenchRunner](crate::BenchRunner) instead.
pub struct InputGroup<I = ()> {
    inputs: Vec<OwnedNamedInput<I>>,
    bench_group: BenchGroup<'static>,
    pub(crate) name: Option<String>,
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
    pub fn name<S: Into<String>>(mut self, name: S) -> Self {
        self.name = Some(name.into());
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
        options: Options,
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
        let mut runner = BenchRunner::new();
        runner.set_options(options);

        InputGroup {
            inputs,
            name: None,
            bench_group: BenchGroup::new(runner),
        }
    }
    /// Set the peak mem allocator to be used for the benchmarks.
    /// This will report the peak memory consumption of the benchmarks.
    pub fn set_alloc<A: GlobalAlloc + 'static>(&mut self, alloc: &'static PeakMemAlloc<A>) {
        self.bench_group.runner.set_alloc(alloc);
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
        self.bench_group.runner.options.enable_perf = true;
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
    pub fn set_name<S: Into<String>>(&mut self, name: S) {
        self.name = Some(name.into());
    }

    /// Set the options to the given value.
    /// This will overwrite all current options.
    ///
    /// See the Options struct for more information.
    pub fn set_options(&mut self, options: Options) {
        self.bench_group.runner.set_options(options);
    }

    /// Manully set the number of iterations each benchmark is called.
    ///
    /// This disables the automatic detection of the number of iterations.
    pub fn set_num_iter(&mut self, num_iter: usize) {
        self.bench_group.runner.set_num_iter(num_iter);
    }

    /// Trash CPU cache between bench runs. Defaults to false.
    pub fn set_cache_trasher(&mut self, enable: bool) {
        self.bench_group.runner.set_cache_trasher(enable);
    }

    /// Sets the interleave option to the given value.
    pub fn set_interleave(&mut self, interleave: bool) {
        self.bench_group.runner.set_interleave(interleave);
    }

    /// Sets the filter, which is used to filter the benchmarks by name.
    /// The filter is fetched from the command line arguments.
    ///
    /// It can also match an input name.
    pub fn set_filter(&mut self, filter: Option<String>) {
        self.bench_group.runner.set_filter(filter);
    }

    /// Register a benchmark with the given name and function.
    pub fn register<F, S: Into<String>>(&mut self, name: S, fun: F)
    where
        F: Fn(&I) + 'static + Clone,
    {
        let name = name.into();

        for input in &self.inputs {
            let named_bench: NamedBench<'static, I> =
                NamedBench::new(name.to_string(), Box::new(fun.clone()));
            let named_input: NamedInput<'_, I> = NamedInput {
                name: Cow::Borrowed(&input.name),
                data: &input.data,
            };
            // The input lives in the InputGroup, so we can transmute the lifetime to 'static
            // (probably).
            let named_input: NamedInput<'static, I> = unsafe { transmute_lifetime(named_input) };
            if let Some(input_size) = input.input_size_in_bytes {
                self.bench_group.runner.set_input_size(input_size);
            }
            self.bench_group
                .register_named_with_input(named_bench, named_input);
        }
    }

    /// Run the benchmarks and report the results.
    pub fn run(&mut self) {
        if let Some(name) = &self.name {
            println!("{}", name.black().on_red().invert().bold());
        }
        let input_name_to_ordinal: HashMap<String, usize> = self
            .inputs
            .iter()
            .enumerate()
            .map(|(i, input)| (input.name.clone(), i))
            .collect();
        self.bench_group
            .benches
            .sort_by_key(|bench| std::cmp::Reverse(input_name_to_ordinal[bench.get_input_name()]));
        group_by_mut(
            self.bench_group.benches.as_mut_slice(),
            |b| b.get_input_name(),
            |group| {
                let input_name = group[0].get_input_name().to_owned();
                //if !input_name.is_empty() {
                //println!("{}", input_name.black().on_yellow().invert().italic());
                //}
                self.bench_group.runner.run_group(Some(&input_name), group);
            },
        );
    }
}

unsafe fn transmute_lifetime<I>(input: NamedInput<'_, I>) -> NamedInput<'static, I> {
    std::mem::transmute(input)
}
