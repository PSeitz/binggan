/// BenchId is a unique identifier for a benchmark.
/// It has three components:
/// - runner_name: The name of the runner that executed the benchmark.
/// - group_name: The name of the group that the benchmark belongs to. This is typically the input name.
/// - bench_name: The name of the benchmark.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BenchId {
    /// This is the name set on the BenchRunner.
    pub runner_name: Option<String>,
    /// The name of the group.
    /// This is typically the input name.
    pub group_name: Option<String>,
    /// The name of the benchmark.
    pub bench_name: String,
}

impl BenchId {
    pub(crate) fn from_bench_name<S: Into<String>>(bench_name: S) -> Self {
        BenchId {
            runner_name: None,
            group_name: None,
            bench_name: bench_name.into(),
        }
    }
    pub(crate) fn runner_name(mut self, name: Option<&str>) -> Self {
        self.runner_name = name.map(|el| el.to_owned());
        self
    }
    pub(crate) fn group_name(mut self, name: Option<String>) -> Self {
        self.group_name = name;
        self
    }
    /// Returns the full name of the bench id.
    /// This is used to identify the bench in the output.
    pub fn get_full_name(&self) -> String {
        get_bench_id(
            self.runner_name.as_deref().unwrap_or_default(),
            self.group_name.as_deref().unwrap_or_default(),
            &self.bench_name,
        )
    }
}
/// create bench id from parts
pub fn get_bench_id(runner_name: &str, group_name: &str, bench_name: &str) -> String {
    format!("{}_{}_{}", runner_name, group_name, bench_name).replace('/', "-")
}
