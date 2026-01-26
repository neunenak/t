use crate::error::Result;
use crate::interpreter::Transform;
use crate::value::Value;

pub struct DeleteEmpty;

impl Transform for DeleteEmpty {
    fn apply(&self, value: Value) -> Result<Value> {
        match value {
            Value::Array(mut arr) => {
                arr.elements.retain(|v| !v.is_empty());
                Ok(Value::Array(arr))
            }
            other => Ok(other),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::{Array, Level};

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
    fn delete_empty_removes_empty_strings() {
        let input = Value::Array(Array::from((
            vec![text("a"), text(""), text("b"), text(""), text("c")],
            Level::Line,
        )));
        let result = DeleteEmpty.apply(input).unwrap();

        match result {
            Value::Array(arr) => {
                assert_eq!(arr.len(), 3);
                assert_eq!(arr.elements[0], text("a"));
                assert_eq!(arr.elements[1], text("b"));
                assert_eq!(arr.elements[2], text("c"));
            }
            _ => panic!("expected array"),
        }
    }

    #[test]
    fn delete_empty_removes_empty_arrays() {
        let inner1 = word_array(&["a", "b"]);
        let inner2 = Value::Array(Array::from((vec![], Level::Word)));
        let inner3 = word_array(&["c"]);
        let input = Value::Array(Array::from((vec![inner1, inner2, inner3], Level::Line)));

        let result = DeleteEmpty.apply(input).unwrap();

        match result {
            Value::Array(arr) => {
                assert_eq!(arr.len(), 2);
            }
            _ => panic!("expected array"),
        }
    }

    #[test]
    fn delete_empty_keeps_numbers() {
        let input = Value::Array(Array::from((
            vec![Value::Number(0.0), text(""), Value::Number(1.0)],
            Level::Line,
        )));
        let result = DeleteEmpty.apply(input).unwrap();

        match result {
            Value::Array(arr) => {
                assert_eq!(arr.len(), 2);
                assert_eq!(arr.elements[0], Value::Number(0.0));
                assert_eq!(arr.elements[1], Value::Number(1.0));
            }
            _ => panic!("expected array"),
        }
    }

    #[test]
    fn delete_empty_on_all_empty() {
        let input = Value::Array(Array::from((vec![text(""), text("")], Level::Line)));
        let result = DeleteEmpty.apply(input).unwrap();

        match result {
            Value::Array(arr) => {
                assert!(arr.is_empty());
            }
            _ => panic!("expected array"),
        }
    }

    #[test]
    fn delete_empty_non_array_is_identity() {
        let input = text("hello");
        let result = DeleteEmpty.apply(input).unwrap();
        assert_eq!(result, text("hello"));
    }
}
