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
/// Implementations can opt into storing and comparing output values across runs by overriding
/// [OutputValue::serialize], [OutputValue::deserialize] and [OutputValue::format_delta].
pub trait OutputValue {
    /// The formatted output value.
    /// If the value is None, it will not be printed.
    ///
    fn format(&self) -> Option<String>;
    /// The name of the column title. The default is "Output".
    fn column_title() -> &'static str {
        "Output"
    }

    /// Serialize the output value for comparison with the next run.
    ///
    /// Returning `None` keeps the old behavior: the formatted output is displayed, but no output
    /// delta is stored or reported.
    fn serialize(&self) -> Option<String> {
        None
    }

    /// Deserialize a previously stored output value.
    fn deserialize(_serialized: &str) -> Option<Self>
    where
        Self: Sized,
    {
        None
    }

    /// Format the delta between this output value and another value.
    fn format_delta(&self, _old: &Self) -> Option<String>
    where
        Self: Sized,
    {
        None
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

fn serialize_json<T: miniserde::Serialize>(value: &T) -> Option<String> {
    Some(miniserde::json::to_string(value))
}

fn deserialize_json<T: miniserde::Deserialize>(serialized: &str) -> Option<T> {
    miniserde::json::from_str(serialized).ok()
}

fn format_percentage_delta(current: f64, old: f64) -> Option<String> {
    if old == 0.0 || current == 0.0 || old == current || !old.is_finite() || !current.is_finite() {
        return None;
    }
    let diff = (current / old - 1.0) * 100.0;
    let diff_str = if diff >= 0.0 {
        format!("(+{:.2}%)", diff)
    } else {
        format!("({:.2}%)", diff)
    };
    Some(diff_str)
}

fn format_u64_delta(current: u64, old: u64) -> Option<String> {
    format_percentage_delta(current as f64, old as f64)
}

fn format_changed_delta<T: PartialEq>(current: &T, old: &T) -> Option<String> {
    if current == old {
        None
    } else {
        Some("(changed)".to_string())
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

    fn serialize(&self) -> Option<String> {
        self.as_ref().and_then(serialize_json)
    }

    fn deserialize(serialized: &str) -> Option<Self> {
        deserialize_json(serialized).map(Some)
    }

    fn format_delta(&self, old: &Self) -> Option<String> {
        format_u64_delta((*self)?, (*old)?)
    }
}
impl OutputValue for u64 {
    fn format(&self) -> Option<String> {
        Some(format_with_underscores(*self))
    }

    fn serialize(&self) -> Option<String> {
        serialize_json(self)
    }

    fn deserialize(serialized: &str) -> Option<Self> {
        deserialize_json(serialized)
    }

    fn format_delta(&self, old: &Self) -> Option<String> {
        format_u64_delta(*self, *old)
    }
}
impl OutputValue for usize {
    fn format(&self) -> Option<String> {
        Some(format_with_underscores(*self as u64))
    }

    fn serialize(&self) -> Option<String> {
        serialize_json(&(*self as u64))
    }

    fn deserialize(serialized: &str) -> Option<Self> {
        let value: u64 = deserialize_json(serialized)?;
        value.try_into().ok()
    }

    fn format_delta(&self, old: &Self) -> Option<String> {
        format_u64_delta(*self as u64, *old as u64)
    }
}
impl OutputValue for String {
    fn format(&self) -> Option<String> {
        Some(self.clone())
    }

    fn serialize(&self) -> Option<String> {
        serialize_json(self)
    }

    fn deserialize(serialized: &str) -> Option<Self> {
        deserialize_json(serialized)
    }

    fn format_delta(&self, old: &Self) -> Option<String> {
        format_changed_delta(self, old)
    }
}
impl OutputValue for f64 {
    fn format(&self) -> Option<String> {
        Some(self.to_string())
    }

    fn serialize(&self) -> Option<String> {
        serialize_json(self)
    }

    fn deserialize(serialized: &str) -> Option<Self> {
        deserialize_json(serialized)
    }

    fn format_delta(&self, old: &Self) -> Option<String> {
        format_percentage_delta(*self, *old)
    }
}
impl OutputValue for i64 {
    fn format(&self) -> Option<String> {
        Some(self.to_string())
    }

    fn serialize(&self) -> Option<String> {
        serialize_json(self)
    }

    fn deserialize(serialized: &str) -> Option<Self> {
        deserialize_json(serialized)
    }

    fn format_delta(&self, old: &Self) -> Option<String> {
        format_percentage_delta(*self as f64, *old as f64)
    }
}
impl OutputValue for bool {
    fn format(&self) -> Option<String> {
        Some(self.to_string())
    }

    fn serialize(&self) -> Option<String> {
        serialize_json(self)
    }

    fn deserialize(serialized: &str) -> Option<Self> {
        deserialize_json(serialized)
    }

    fn format_delta(&self, old: &Self) -> Option<String> {
        format_changed_delta(self, old)
    }
}
impl OutputValue for std::time::Duration {
    fn format(&self) -> Option<String> {
        Some(format_duration(self.as_nanos() as u64))
    }

    fn serialize(&self) -> Option<String> {
        serialize_json(&(self.as_nanos() as u64))
    }

    fn deserialize(serialized: &str) -> Option<Self> {
        let nanos: u64 = deserialize_json(serialized)?;
        Some(Self::from_nanos(nanos))
    }

    fn format_delta(&self, old: &Self) -> Option<String> {
        format_u64_delta(self.as_nanos() as u64, old.as_nanos() as u64)
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

    fn format_delta(&self, old: &Self) -> Option<String> {
        format_u64_delta(self.len() as u64, old.len() as u64)
    }
}
impl<K, V> OutputValue for HashMap<K, V> {
    fn format(&self) -> Option<String> {
        Some(format_with_underscores(self.len() as u64))
    }
    fn column_title() -> &'static str {
        "Map(len)"
    }

    fn format_delta(&self, old: &Self) -> Option<String> {
        format_u64_delta(self.len() as u64, old.len() as u64)
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
    fn output_values_can_serialize_and_delta() {
        let value = 150u64;
        assert_eq!(value.serialize(), Some("150".to_string()));
        assert_eq!(value.format_delta(&100), Some("(+50.00%)".to_string()));

        let value = vec![1, 2, 3];
        assert_eq!(value.serialize(), None);
        assert_eq!(
            value.format_delta(&vec![1, 2]),
            Some("(+50.00%)".to_string())
        );
    }

    #[test]
    fn output_value_delta_for_changed_non_numeric_values() {
        assert_eq!(
            "new".to_string().format_delta(&"old".to_string()),
            Some("(changed)".to_string())
        );
        assert_eq!(true.format_delta(&false), Some("(changed)".to_string()));
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
