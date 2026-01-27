use crate::error::Result;
use crate::interpreter::Transform;
use crate::value::{Array, Value};

pub struct Join;

impl Transform for Join {
    fn apply(&self, value: Value) -> Result<Value> {
        match value {
            Value::Array(arr) => Ok(join_array(arr)),
            other => Ok(other),
        }
    }
}

fn join_array(arr: Array) -> Value {
    let first_is_array = arr
        .elements
        .first()
        .is_some_and(|v| matches!(v, Value::Array(_)));

    if first_is_array {
        let total_len: usize = arr
            .elements
            .iter()
            .map(|e| {
                if let Value::Array(inner) = e {
                    inner.len()
                } else {
                    0
                }
            })
            .sum();
        let mut flattened: Vec<Value> = Vec::with_capacity(total_len);
        let mut inner_level = arr.level;

        for elem in arr.elements {
            if let Value::Array(inner) = elem {
                inner_level = inner.level;
                flattened.extend(inner.elements);
            }
        }

        Value::Array(Array::from((flattened, inner_level)))
    } else {
        let parts: Vec<String> = arr
            .elements
            .into_iter()
            .map(|v| match v {
                Value::Text(s) => s,
                Value::Number(n) => n.to_string(),
                Value::Array(inner) => match join_array(inner) {
                    Value::Text(s) => s,
                    Value::Number(n) => n.to_string(),
                    _ => String::new(),
                },
            })
            .collect();
        Value::Text(parts.join(" "))
    }
}

pub struct JoinDelim {
    delimiter: String,
}

impl JoinDelim {
    pub fn new(delimiter: String) -> Self {
        Self { delimiter }
    }
}

impl Transform for JoinDelim {
    fn apply(&self, value: Value) -> Result<Value> {
        match value {
            Value::Array(arr) => {
                let parts: Vec<String> = arr
                    .elements
                    .into_iter()
                    .map(|v| match v {
                        Value::Text(s) => s,
                        Value::Number(n) => n.to_string(),
                        Value::Array(inner) => inner.to_string(),
                    })
                    .collect();
                Ok(Value::Text(parts.join(&self.delimiter)))
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

    fn line_array(lines: &[&str]) -> Value {
        Value::Array(Array::from((
            lines.iter().map(|s| text(s)).collect(),
            Level::Line,
        )))
    }

    #[test]
    fn join_strings_with_space() {
        let input = word_array(&["hello", "world"]);
        let result = Join.apply(input).unwrap();
        assert_eq!(result, text("hello world"));
    }

    #[test]
    fn join_single_string() {
        let input = word_array(&["hello"]);
        let result = Join.apply(input).unwrap();
        assert_eq!(result, text("hello"));
    }

    #[test]
    fn join_empty_array_of_strings() {
        let input = Value::Array(Array::from((vec![], Level::Word)));
        let result = Join.apply(input).unwrap();
        assert_eq!(result, text(""));
    }

    #[test]
    fn join_flattens_array_of_arrays() {
        let inner1 = word_array(&["hello", "world"]);
        let inner2 = word_array(&["foo", "bar"]);
        let outer = Value::Array(Array::from((vec![inner1, inner2], Level::Line)));

        let result = Join.apply(outer).unwrap();
        match result {
            Value::Array(arr) => {
                assert_eq!(arr.len(), 4);
                assert_eq!(arr.elements[0], text("hello"));
                assert_eq!(arr.elements[1], text("world"));
                assert_eq!(arr.elements[2], text("foo"));
                assert_eq!(arr.elements[3], text("bar"));
            }
            _ => panic!("expected array"),
        }
    }

    #[test]
    fn join_flattens_single_inner_array() {
        let inner = word_array(&["a", "b", "c"]);
        let outer = Value::Array(Array::from((vec![inner], Level::Line)));

        let result = Join.apply(outer).unwrap();
        match result {
            Value::Array(arr) => {
                assert_eq!(arr.len(), 3);
                assert_eq!(arr.elements[0], text("a"));
            }
            _ => panic!("expected array"),
        }
    }

    #[test]
    fn join_flattens_empty_inner_arrays() {
        let inner1 = Value::Array(Array::from((vec![], Level::Word)));
        let inner2 = word_array(&["a"]);
        let inner3 = Value::Array(Array::from((vec![], Level::Word)));
        let outer = Value::Array(Array::from((vec![inner1, inner2, inner3], Level::Line)));

        let result = Join.apply(outer).unwrap();
        match result {
            Value::Array(arr) => {
                assert_eq!(arr.len(), 1);
                assert_eq!(arr.elements[0], text("a"));
            }
            _ => panic!("expected array"),
        }
    }

    #[test]
    fn join_preserves_inner_level() {
        let inner1 = Value::Array(Array::from((vec![text("a")], Level::Char)));
        let inner2 = Value::Array(Array::from((vec![text("b")], Level::Char)));
        let outer = Value::Array(Array::from((vec![inner1, inner2], Level::Word)));

        let result = Join.apply(outer).unwrap();
        match result {
            Value::Array(arr) => {
                assert_eq!(arr.level, Level::Char);
                assert_eq!(arr.len(), 2);
            }
            _ => panic!("expected array"),
        }
    }

    #[test]
    fn join_numbers_with_space() {
        let input = Value::Array(Array::from((
            vec![Value::Number(1.0), Value::Number(2.0), Value::Number(3.0)],
            Level::Word,
        )));
        let result = Join.apply(input).unwrap();
        assert_eq!(result, text("1 2 3"));
    }

    #[test]
    fn join_non_array_is_identity() {
        let input = text("hello");
        let result = Join.apply(input).unwrap();
        assert_eq!(result, text("hello"));
    }

    #[test]
    fn join_delim_comma() {
        let input = line_array(&["a", "b", "c"]);
        let result = JoinDelim::new(",".to_string()).apply(input).unwrap();
        assert_eq!(result, text("a,b,c"));
    }

    #[test]
    fn join_delim_multi_char() {
        let input = line_array(&["a", "b", "c"]);
        let result = JoinDelim::new(", ".to_string()).apply(input).unwrap();
        assert_eq!(result, text("a, b, c"));
    }

    #[test]
    fn join_delim_empty() {
        let input = line_array(&["a", "b", "c"]);
        let result = JoinDelim::new("".to_string()).apply(input).unwrap();
        assert_eq!(result, text("abc"));
    }

    #[test]
    fn join_delim_single_element() {
        let input = line_array(&["hello"]);
        let result = JoinDelim::new(",".to_string()).apply(input).unwrap();
        assert_eq!(result, text("hello"));
    }

    #[test]
    fn join_delim_empty_array() {
        let input = Value::Array(Array::from((vec![], Level::Line)));
        let result = JoinDelim::new(",".to_string()).apply(input).unwrap();
        assert_eq!(result, text(""));
    }

    #[test]
    fn join_delim_with_numbers() {
        let input = Value::Array(Array::from((
            vec![Value::Number(1.0), Value::Number(2.0), Value::Number(3.0)],
            Level::Line,
        )));
        let result = JoinDelim::new(",".to_string()).apply(input).unwrap();
        assert_eq!(result, text("1,2,3"));
    }

    #[test]
    fn join_delim_non_array_is_identity() {
        let input = text("hello");
        let result = JoinDelim::new(",".to_string()).apply(input).unwrap();
        assert_eq!(result, text("hello"));
    }
}
