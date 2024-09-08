use std::sync::{Arc, Once};

/// The bench runners name is like a header and should only be printed if there are tests to be
/// run. Since this information is available at the time of creation, it will be handled when
/// executing the benches instead.
struct PrintOnce {
    name: String,
    print_once: Once, // Instance-specific Once
}

impl PrintOnce {
    pub(crate) fn new(name: String) -> Arc<Self> {
        Arc::new(PrintOnce {
            name,
            print_once: Once::new(), // Each instance has its own Once
        })
    }

    pub(crate) fn print_name(&self) {
        self.print_once.call_once(|| {
            println!("Singleton name: {}", self.name);
        });
    }
}

pub struct BenchId {
    pub runner_name: Option<String>,
    pub group_name: Option<String>,
    pub bench_name: String,
}

impl BenchId {
    pub fn get_full_name(&self) {
        get_bench_id(
            self.runner_name.as_deref().unwrap_or_default(),
            self.group_name.as_deref().unwrap_or_default(),
            &self.bench_name,
        );
    }
}
/// create bench id from parts
pub fn get_bench_id(runner_name: &str, group_name: &str, bench_name: &str) -> String {
    format!("{}_{}_{}", runner_name, group_name, bench_name).replace('/', "-")
}
