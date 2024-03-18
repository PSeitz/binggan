pub fn format_duration(duration: u64) -> String {
    const NANOS_PER_SEC: u64 = 1_000_000_000;
    const NANOS_PER_MILLI: u64 = 1_000_000;

    let total_nanos = duration; // Get total nanoseconds

    if total_nanos < 100 {
        format!("{}ns", total_nanos)
    } else if total_nanos <= NANOS_PER_SEC {
        let millis = duration as f64 / NANOS_PER_MILLI as f64;
        format!("{:.4}ms", millis)
    } else {
        let seconds = duration as f64 / NANOS_PER_SEC as f64;
        format!("{}s", seconds)
    }
}

pub fn format_with_underscores(number: u64) -> String {
    let num_str = number.to_string();
    let mut result = String::new();
    let chars: Vec<_> = num_str.chars().rev().collect();
    for (i, char) in chars.iter().enumerate() {
        if i % 3 == 0 && i != 0 {
            result.push('_');
        }
        result.push(*char);
    }
    result.chars().rev().collect()
}

/// bytes size for 1 kilobyte
pub const KB: u64 = 1_000;

static UNITS: &str = "KMGTPE";
static LN_KB: f64 = 6.931471806; // ln 1024

/// Byte size representation
#[derive(Copy, Clone, PartialEq, PartialOrd, Eq, Ord, Hash, Default)]
pub struct ByteSize(pub u64);

pub fn bytes_to_string(bytes: u64) -> String {
    let unit = KB;
    let unit_base = LN_KB;
    let unit_prefix = UNITS.as_bytes();
    let unit_suffix = "B";

    if bytes < unit {
        format!("{} B", bytes)
    } else {
        let size = bytes as f64;
        let exp = match (size.ln() / unit_base) as usize {
            e if e == 0 => 1,
            e => e,
        };

        format!(
            "{:.1} {}{}",
            (size / unit.pow(exp as u32) as f64),
            unit_prefix[exp - 1] as char,
            unit_suffix
        )
    }
}
