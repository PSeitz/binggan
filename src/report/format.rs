/// Formats a duration given in nanoseconds into a human-readable string.
///
/// # Parameters
/// - `duration`: The duration in nanoseconds.
///
/// # Returns
/// A string representation of the duration, which can be in nanoseconds (ns), milliseconds (ms), or seconds (s).
///
/// - If the duration is less than 10,000 nanoseconds, it will be represented as nanoseconds.
/// - If the duration is between 10,000 nanoseconds and 1 second, it will be represented as milliseconds.
/// - If the duration is greater than 1 second, it will be represented as seconds.
pub fn format_duration(duration: u64) -> String {
    const NANOS_PER_SEC: u64 = 1_000_000_000;
    const NANOS_PER_MILLI: u64 = 1_000_000;

    let total_nanos = duration; // Get total nanoseconds

    if total_nanos < 10_000 {
        format!("{}ns", total_nanos)
    } else if total_nanos <= NANOS_PER_SEC {
        let millis = duration as f64 / NANOS_PER_MILLI as f64;
        format!("{:.4}ms", millis)
    } else {
        let seconds = duration as f64 / NANOS_PER_SEC as f64;
        format!("{}s", seconds)
    }
}

/// Formats a number by adding underscores to separate thousands for better readability.
///
/// # Parameters
/// - `number`: The number to format.
///
/// # Returns
/// A string representation of the number, where underscores are inserted every three digits for readability.
/// For example, `1000000` becomes `1_000_000`.
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

/// Converts a byte size to a human-readable string representation.
///
/// # Parameters
/// - `bytes`: The number of bytes to convert.
///
/// # Returns
/// A string representing the size with an appropriate unit (e.g., B, KB, MB, GB, etc.).
/// For example, `1024` bytes becomes `1.0 KB`, and `1_048_576` bytes becomes `1.0 MB`.
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
            0 => 1,
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

/// Formats a duration or throughput depending on whether the input size is provided.
pub fn format_duration_or_throughput(
    duration_ns: u64,
    input_size_in_bytes: Option<usize>,
) -> String {
    if let Some(input_size_in_bytes) = input_size_in_bytes {
        let mut duration_ns: f64 = duration_ns as f64;
        let unit = unit_per_second(input_size_in_bytes, &mut duration_ns);
        format!("{:>6} {}", format_float(duration_ns), unit)
    } else {
        format_duration(duration_ns).to_string()
    }
}

/// Formats a floating-point number (`f64`) into a shorter, human-readable string
/// with varying precision depending on the value of the number.
///
/// # Parameters
/// - `n`: The floating-point number to format.
///
/// # Returns
/// A string representation of the number with different decimal precision based on its value:
/// - If `n` is less than 10, it will be formatted with 4 decimal places.
/// - If `n` is between 10 and 100, it will be formatted with 3 decimal places.
/// - If `n` is between 100 and 1000, it will be formatted with 2 decimal places.
/// - If `n` is between 1000 and 10000, it will be formatted with 1 decimal place.
/// - If `n` is greater than or equal to 10000, it will be formatted with no decimal places.
///
/// # Examples
/// ```
/// use binggan::report::format::format_float;
/// let value = 9.876543;
/// assert_eq!(format_float(value), "9.8765");
///
/// let value = 987.6543;
/// assert_eq!(format_float(value), "987.65");
///
/// let value = 12345.67;
/// assert_eq!(format_float(value), "12346");
/// ```
pub fn format_float(n: f64) -> String {
    if n < 10.0 {
        format!("{:.4}", n)
    } else if n < 100.0 {
        format!("{:.3}", n)
    } else if n < 1000.0 {
        format!("{:.2}", n)
    } else if n < 10000.0 {
        format!("{:.1}", n)
    } else {
        format!("{:.0}", n)
    }
}

/// Returns the unit and alters the passed parameter to match the unit
pub fn unit_per_second(bytes: usize, nanoseconds: &mut f64) -> &'static str {
    let bytes_per_second = bytes as f64 * (1e9 / *nanoseconds);
    let (denominator, unit) = if bytes_per_second < 1000.0 {
        (1.0, "  B/s")
    } else if bytes_per_second < 1000.0 * 1000.0 {
        (1000.0, "KB/s")
    } else if bytes_per_second < 1000.0 * 1000.0 * 1000.0 {
        (1000.0 * 1000.0, "MB/s")
    } else {
        (1000.0 * 1000.0 * 1000.0, "GB/s")
    };

    let bytes_per_second = bytes as f64 * (1e9 / *nanoseconds);
    *nanoseconds = bytes_per_second / denominator;

    unit
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_throughput_test() {
        let bytes = 1000;
        let mut nanoseconds = 1e9;
        assert_eq!(unit_per_second(bytes, &mut nanoseconds), "KB/s");
        assert_eq!(
            format_duration_or_throughput(1e9 as u64, Some(1000000)),
            "1.0000 MB/s"
        );
    }
}
