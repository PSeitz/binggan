use crate::report::format::{format_duration, format_with_underscores};

/// Every bench returns an OutputValue, which can be formatted to a string.
///
/// This can be useful in many cases as it allows to return additional information from a bench to
/// provide additional dimensions to track and compare.
///
/// The OutputValue is typically a single value, but can be a tuple or struct if multiple values
/// are needed.
/// The OutputValue is printed in the output table.
///
/// # Example
/// In a compression benchmark this could be the output size.
/// In a tree this could be the number of nodes. Any metric that is interesting to compare.
///
/// # Limitations
/// OutputValue is currently not part of the delta detection between runs.
pub trait OutputValue {
    /// The formatted output value.
    /// If the value is None, it will not be printed.
    ///
    fn format(&self) -> Option<String>;
}

impl OutputValue for () {
    fn format(&self) -> Option<String> {
        None
    }
}
impl OutputValue for u64 {
    fn format(&self) -> Option<String> {
        Some(format_with_underscores(*self))
    }
}
impl OutputValue for String {
    fn format(&self) -> Option<String> {
        Some(self.clone())
    }
}
impl OutputValue for f64 {
    fn format(&self) -> Option<String> {
        Some(self.to_string())
    }
}
impl OutputValue for i64 {
    fn format(&self) -> Option<String> {
        Some(self.to_string())
    }
}
impl OutputValue for bool {
    fn format(&self) -> Option<String> {
        Some(self.to_string())
    }
}
impl OutputValue for std::time::Duration {
    fn format(&self) -> Option<String> {
        Some(format_duration(self.as_nanos() as u64))
    }
}
impl OutputValue for std::time::Instant {
    fn format(&self) -> Option<String> {
        Some(format_duration(self.elapsed().as_nanos() as u64))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_u64_test() {
        let value = 123456789u64;
        assert_eq!(value.format(), Some("123_456_789".to_string()));
    }
}
