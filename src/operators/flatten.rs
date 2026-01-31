use crate::error::Result;
use crate::interpreter::Transform;
use crate::value::{Array, Value};

/// Flattens nested arrays by one level.
///
/// `[["a", "b"], ["c"]]` → `["a", "b", "c"]`
///
/// Only flattens one level deep. Non-array elements are kept as-is.
pub struct Flatten;

impl Transform for Flatten {
    fn apply(&self, value: Value) -> Result<Value> {
        match value {
            Value::Array(arr) => {
                let level = arr.level;
                let mut flattened = Vec::new();
                for elem in arr.elements {
                    match elem {
                        Value::Array(inner) => {
                            flattened.extend(inner.elements);
                        }
                        other => flattened.push(other),
                    }
                }
                Ok(Value::Array(Array::from((flattened, level))))
            }
            other => Ok(other),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::Level;

    fn text(s: &str) -> Value {
        Value::Text(s.to_string())
    }

    fn word_array(words: &[&str]) -> Value {
        Value::Array(Array::from((
            words.iter().map(|s| text(s)).collect(),
            Level::Word,
        )))
    }

    #[test]
    fn flatten_nested_arrays() {
        let inner1 = word_array(&["a", "b"]);
        let inner2 = word_array(&["c"]);
        let outer = Value::Array(Array::from((vec![inner1, inner2], Level::Line)));

        let result = Flatten.apply(outer).unwrap();
        match result {
            Value::Array(arr) => {
                assert_eq!(arr.level, Level::Line);
                assert_eq!(arr.len(), 3);
                assert_eq!(arr.elements[0], text("a"));
                assert_eq!(arr.elements[1], text("b"));
                assert_eq!(arr.elements[2], text("c"));
            }
            _ => panic!("expected array"),
        }
    }

    #[test]
    fn flatten_single_level_only() {
        // [["a", ["b", "c"]]] -> ["a", ["b", "c"]]
        let deep = word_array(&["b", "c"]);
        let inner = Value::Array(Array::from((vec![text("a"), deep], Level::Word)));
        let outer = Value::Array(Array::from((vec![inner], Level::Line)));

        let result = Flatten.apply(outer).unwrap();
        match result {
            Value::Array(arr) => {
                assert_eq!(arr.len(), 2);
                assert_eq!(arr.elements[0], text("a"));
                match &arr.elements[1] {
                    Value::Array(inner) => {
                        assert_eq!(inner.len(), 2);
                    }
                    _ => panic!("expected nested array"),
                }
            }
            _ => panic!("expected array"),
        }
    }

    #[test]
    fn flatten_mixed_elements() {
        // [["a"], "b", ["c", "d"]] -> ["a", "b", "c", "d"]
        let inner1 = word_array(&["a"]);
        let inner2 = word_array(&["c", "d"]);
        let outer = Value::Array(Array::from((vec![inner1, text("b"), inner2], Level::Line)));

        let result = Flatten.apply(outer).unwrap();
        match result {
            Value::Array(arr) => {
                assert_eq!(arr.len(), 4);
                assert_eq!(arr.elements[0], text("a"));
                assert_eq!(arr.elements[1], text("b"));
                assert_eq!(arr.elements[2], text("c"));
                assert_eq!(arr.elements[3], text("d"));
            }
            _ => panic!("expected array"),
        }
    }

    #[test]
    fn flatten_empty_array() {
        let input = Value::Array(Array::from((vec![], Level::Line)));
        let result = Flatten.apply(input).unwrap();
        match result {
            Value::Array(arr) => {
                assert_eq!(arr.len(), 0);
            }
            _ => panic!("expected array"),
        }
    }

    #[test]
    fn flatten_flat_array_is_noop() {
        // ["a", "b", "c"] -> ["a", "b", "c"]
        let input = word_array(&["a", "b", "c"]);
        let result = Flatten.apply(input).unwrap();
        match result {
            Value::Array(arr) => {
                assert_eq!(arr.len(), 3);
                assert_eq!(arr.elements[0], text("a"));
            }
            _ => panic!("expected array"),
        }
    }

    #[test]
    fn flatten_non_array_is_identity() {
        let input = text("hello");
        let result = Flatten.apply(input).unwrap();
        assert_eq!(result, text("hello"));
    }

    #[test]
    fn flatten_number_is_identity() {
        let input = Value::Number(42.0);
        let result = Flatten.apply(input).unwrap();
        assert_eq!(result, Value::Number(42.0));
    }
}
