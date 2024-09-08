use std::{
    ops::Deref,
    sync::{Arc, Once},
};

use yansi::Paint;

/// The bench runners name is like a header and should only be printed if there are tests to be
/// run. Since this information is available at the time of creation, it will be handled when
/// executing the benches instead.
#[derive(Clone)]
pub struct PrintOnce {
    inner: Arc<PrintOnceInner>,
}

impl Deref for PrintOnce {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.inner.name
    }
}
struct PrintOnceInner {
    name: String,
    print_once: Once,
}

impl PrintOnce {
    pub fn new(name: String) -> Self {
        PrintOnce {
            inner: Arc::new(PrintOnceInner {
                name,
                print_once: Once::new(),
            }),
        }
    }

    pub fn print_name(&self) {
        self.inner.print_once.call_once(|| {
            println!("{}", self.get_name().black().on_red().invert().bold());
        });
    }
    pub fn get_name(&self) -> &str {
        &self.inner.name
    }
}

/// BenchId is a unique identifier for a benchmark.
/// It has three components:
/// - runner_name: The name of the runner that executed the benchmark.
/// - group_name: The name of the group that the benchmark belongs to. This is typically the input name.
/// - bench_name: The name of the benchmark.
#[derive(Clone)]
pub struct BenchId {
    runner_name: Option<String>,
    /// This is typically the input name
    group_name: Option<String>,
    pub(crate) bench_name: String,
}

impl BenchId {
    pub fn from_bench_name<S: Into<String>>(bench_name: S) -> Self {
        BenchId {
            runner_name: None,
            group_name: None,
            bench_name: bench_name.into(),
        }
    }
    pub fn runner_name(mut self, name: Option<&str>) -> Self {
        self.runner_name = name.map(|el| el.to_owned());
        self
    }
    pub fn group_name(mut self, name: Option<String>) -> Self {
        self.group_name = name;
        self
    }
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
