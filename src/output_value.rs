use crate::report::format::{format_duration, format_with_underscores};
use core::mem::{needs_drop, size_of};
use std::collections::HashMap;

/// Every bench returns an OutputValue, which can be formatted to a string.
///
/// This can be useful in many cases as provide additional dimensions to track for a bench.
///
/// The OutputValue is printed in the output table with the title [OutputValue::column_title].
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
    /// The name of the column title. The default is "Output".
    fn column_title() -> &'static str {
        "Output"
    }

    /// Whether the output should be buffered and dropped after the measurement was recorded.
    /// Defaults to deferring drop for types that actually need it and are not zero-sized.
    #[inline]
    fn defer_drop() -> bool
    where
        Self: Sized,
    {
        needs_drop::<Self>() && size_of::<Self>() > 0
    }
}

impl OutputValue for () {
    fn format(&self) -> Option<String> {
        None
    }
}
impl OutputValue for Option<u64> {
    fn format(&self) -> Option<String> {
        self.map(format_with_underscores)
    }
}
impl OutputValue for u64 {
    fn format(&self) -> Option<String> {
        Some(format_with_underscores(*self))
    }
}
impl OutputValue for usize {
    fn format(&self) -> Option<String> {
        Some(format_with_underscores(*self as u64))
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

impl<T> OutputValue for Vec<T> {
    fn format(&self) -> Option<String> {
        Some(format_with_underscores(self.len() as u64))
    }
    fn column_title() -> &'static str {
        "Vec(len)"
    }
}
impl<K, V> OutputValue for HashMap<K, V> {
    fn format(&self) -> Option<String> {
        Some(format_with_underscores(self.len() as u64))
    }
    fn column_title() -> &'static str {
        "Map(len)"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct NeedsDrop(u8);
    impl Drop for NeedsDrop {
        fn drop(&mut self) {}
    }
    impl OutputValue for NeedsDrop {
        fn format(&self) -> Option<String> {
            Some(self.0.to_string())
        }
    }

    #[test]
    fn format_u64_test() {
        let value = 123456789u64;
        assert_eq!(value.format(), Some("123_456_789".to_string()));
    }

    #[test]
    fn should_buffer_outputs_respects_drop_semantics() {
        assert!(NeedsDrop::defer_drop());
        assert!(!bool::defer_drop());

        struct ZeroSizedNeedsDrop;
        impl Drop for ZeroSizedNeedsDrop {
            fn drop(&mut self) {}
        }
        impl OutputValue for ZeroSizedNeedsDrop {
            fn format(&self) -> Option<String> {
                None
            }
        }

        // Zero-sized types should not allocate buffering slots
        assert!(!ZeroSizedNeedsDrop::defer_drop());
    }
}
