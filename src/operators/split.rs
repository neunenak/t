use crate::error::Result;
use crate::interpreter::Transform;
use crate::value::{Array, Level, Value};

pub struct Split;

impl Transform for Split {
    fn apply(&self, value: Value) -> Result<Value> {
        match value {
            Value::Array(mut arr) => {
                arr.elements = arr
                    .elements
                    .into_iter()
                    .map(|v| self.apply(v))
                    .collect::<Result<Vec<_>>>()?;
                Ok(Value::Array(arr))
            }
            Value::Text(s) => Ok(split_text(&s, Level::Line)),
            Value::Number(n) => Ok(Value::Number(n)),
        }
    }
}

fn split_text(s: &str, level: Level) -> Value {
    let new_level = level.split_into();
    let elements: Vec<Value> = match level {
        Level::File => s
            .lines()
            .map(|line| Value::Text(line.to_string()))
            .collect(),
        Level::Line => s
            .split_whitespace()
            .map(|word| Value::Text(word.to_string()))
            .collect(),
        Level::Word => s.chars().map(|c| Value::Text(c.to_string())).collect(),
        Level::Char => vec![Value::Text(s.to_string())],
    };
    Value::Array(Array::from((elements, new_level)))
}

pub struct SplitDelim {
    delimiter: String,
}

impl SplitDelim {
    pub fn new(delimiter: String) -> Self {
        Self { delimiter }
    }
}

impl Transform for SplitDelim {
    fn apply(&self, value: Value) -> Result<Value> {
        match value {
            Value::Array(mut arr) => {
                arr.elements = arr
                    .elements
                    .into_iter()
                    .map(|v| self.apply(v))
                    .collect::<Result<Vec<_>>>()?;
                Ok(Value::Array(arr))
            }
            Value::Text(s) => {
                let parts: Vec<Value> = s
                    .split(&self.delimiter)
                    .map(|part| Value::Text(part.to_string()))
                    .collect();
                Ok(Value::Array(Array::from((parts, Level::Word))))
            }
            Value::Number(n) => Ok(Value::Number(n)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text(s: &str) -> Value {
        Value::Text(s.to_string())
    }

    fn line_array(lines: &[&str]) -> Value {
        Value::Array(Array::from((
            lines.iter().map(|s| text(s)).collect(),
            Level::Line,
        )))
    }

    #[test]
    fn split_line_into_words() {
        let result = Split.apply(text("hello world")).unwrap();
        match result {
            Value::Array(arr) => {
                assert_eq!(arr.level, Level::Word);
                assert_eq!(arr.len(), 2);
                assert_eq!(arr.elements[0], text("hello"));
                assert_eq!(arr.elements[1], text("world"));
            }
            _ => panic!("expected array"),
        }
    }

    #[test]
    fn split_array_of_lines() {
        let input = line_array(&["hello world", "foo bar baz"]);
        let result = Split.apply(input).unwrap();

        match result {
            Value::Array(arr) => {
                assert_eq!(arr.len(), 2);
                match &arr.elements[0] {
                    Value::Array(inner) => {
                        assert_eq!(inner.len(), 2);
                        assert_eq!(inner.elements[0], text("hello"));
                    }
                    _ => panic!("expected inner array"),
                }
                match &arr.elements[1] {
                    Value::Array(inner) => {
                        assert_eq!(inner.len(), 3);
                    }
                    _ => panic!("expected inner array"),
                }
            }
            _ => panic!("expected array"),
        }
    }

    #[test]
    fn split_delim_comma() {
        let input = text("a,b,c");
        let result = SplitDelim::new(",".to_string()).apply(input).unwrap();
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
    fn split_delim_multi_char() {
        let input = text("a::b::c");
        let result = SplitDelim::new("::".to_string()).apply(input).unwrap();
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
    fn split_delim_no_match() {
        let input = text("hello world");
        let result = SplitDelim::new(",".to_string()).apply(input).unwrap();
        match result {
            Value::Array(arr) => {
                assert_eq!(arr.len(), 1);
                assert_eq!(arr.elements[0], text("hello world"));
            }
            _ => panic!("expected array"),
        }
    }

    #[test]
    fn split_delim_empty_parts() {
        let input = text("a,,b");
        let result = SplitDelim::new(",".to_string()).apply(input).unwrap();
        match result {
            Value::Array(arr) => {
                assert_eq!(arr.len(), 3);
                assert_eq!(arr.elements[0], text("a"));
                assert_eq!(arr.elements[1], text(""));
                assert_eq!(arr.elements[2], text("b"));
            }
            _ => panic!("expected array"),
        }
    }

    #[test]
    fn split_delim_array_of_strings() {
        let input = line_array(&["a,b", "c,d,e"]);
        let result = SplitDelim::new(",".to_string()).apply(input).unwrap();
        match result {
            Value::Array(arr) => {
                assert_eq!(arr.len(), 2);
                match &arr.elements[0] {
                    Value::Array(inner) => {
                        assert_eq!(inner.len(), 2);
                        assert_eq!(inner.elements[0], text("a"));
                        assert_eq!(inner.elements[1], text("b"));
                    }
                    _ => panic!("expected inner array"),
                }
                match &arr.elements[1] {
                    Value::Array(inner) => {
                        assert_eq!(inner.len(), 3);
                    }
                    _ => panic!("expected inner array"),
                }
            }
            _ => panic!("expected array"),
        }
    }

    #[test]
    fn split_delim_preserves_numbers() {
        let input = Value::Number(42.0);
        let result = SplitDelim::new(",".to_string()).apply(input).unwrap();
        assert_eq!(result, Value::Number(42.0));
    }
}
