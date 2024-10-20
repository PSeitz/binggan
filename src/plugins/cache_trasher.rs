use super::{EventListener, PluginEvents};

/// Performs a dummy reads from memory to spoil given amount of CPU cache
///
/// Uses cache aligned data arrays to perform minimum amount of reads possible to spoil the cache
#[derive(Clone)]
pub struct CacheTrasher {
    cache_lines: Vec<CacheLine>,
    seed: u64, // Seed for pseudo-random number generation
}
impl Default for CacheTrasher {
    fn default() -> Self {
        Self::new(1024 * 1024 * 32) // 32MB
    }
}

impl CacheTrasher {
    /// Creates a new instance of `CacheTrasher`.
    ///
    /// The `bytes` parameter is the amount of memory to read to spoil the cache.
    #[allow(unused_qualifications)]
    pub fn new(bytes: usize) -> Self {
        let n = bytes / std::mem::size_of::<CacheLine>();
        let cache_lines = vec![CacheLine::default(); n];
        Self {
            cache_lines,
            seed: 0,
        }
    }

    /// Linear Congruential Generator (LCG) for pseudo-random numbers
    fn lcg_rand(&mut self) -> usize {
        const A: u64 = 1664525;
        const C: u64 = 1013904223;

        // Update the seed
        self.seed = A.wrapping_mul(self.seed).wrapping_add(C);
        (self.seed % (self.cache_lines.len() as u64)) as usize
    }

    fn issue_read(&mut self) {
        let num_reads = self.cache_lines.len();
        for _ in 0..num_reads {
            // Use the LCG to generate a random index
            let idx = self.lcg_rand();
            // Because CacheLine is aligned on 64 bytes it is enough to read single element from the array
            // to spoil the whole cache line
            unsafe { std::ptr::read_volatile(&self.cache_lines[idx].0[0]) }; // Access a random cache line
        }
    }
}

#[repr(C)]
#[repr(align(64))]
#[derive(Default, Clone, Copy)]
struct CacheLine([u16; 32]);

impl EventListener for CacheTrasher {
    fn as_any(&mut self) -> &mut dyn std::any::Any {
        self
    }
    fn name(&self) -> &'static str {
        "cache_trasher"
    }
    fn on_event(&mut self, event: PluginEvents) {
        if let PluginEvents::BenchStart { bench_id: _ } = event {
            self.issue_read();
        }
    }
}
