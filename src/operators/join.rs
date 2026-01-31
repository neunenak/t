use crate::error::Result;
use crate::interpreter::Transform;
use crate::value::{Array, Value};

/// Join mode determines how `j` joins strings.
#[derive(Debug, Clone, Default)]
pub enum JoinMode {
    /// Join using semantic level (default) - inverse of split
    #[default]
    Semantic,
    /// Join with a specific delimiter
    Delimiter(String),
    /// Join as CSV fields
    Csv,
}

pub struct Join {
    mode: JoinMode,
}

impl Join {
    pub fn new(mode: JoinMode) -> Self {
        Self { mode }
    }
}

impl Default for Join {
    fn default() -> Self {
        Self {
            mode: JoinMode::Semantic,
        }
    }
}

impl Transform for Join {
    fn apply(&self, value: Value) -> Result<Value> {
        match value {
            Value::Array(mut arr) => {
                // Apply join to each element of the array
                arr.elements = arr
                    .elements
                    .into_iter()
                    .map(|elem| join_element(elem, &self.mode))
                    .collect();
                Ok(Value::Array(arr))
            }
            other => Ok(other),
        }
    }
}

/// Join a single element. Arrays are joined into text; non-arrays pass through unchanged.
fn join_element(value: Value, mode: &JoinMode) -> Value {
    match value {
        Value::Array(arr) => join_array(arr, mode),
        other => other,
    }
}

/// Join an array into a single text value using the appropriate delimiter.
fn join_array(arr: Array, mode: &JoinMode) -> Value {
    let delimiter = match mode {
        JoinMode::Semantic => arr.level.join_delimiter(),
        JoinMode::Delimiter(delim) => delim.as_str(),
        JoinMode::Csv => ",", // CSV handled specially below
    };

    let parts: Vec<String> = arr
        .elements
        .into_iter()
        .map(|v| match v {
            Value::Text(s) => s,
            Value::Number(n) => n.to_string(),
            Value::Array(inner) => match join_array(inner, mode) {
                Value::Text(s) => s,
                Value::Number(n) => n.to_string(),
                _ => String::new(),
            },
        })
        .collect();

    let joined = match mode {
        JoinMode::Csv => {
            if parts.is_empty() {
                String::new()
            } else {
                let mut writer = csv::Writer::from_writer(vec![]);
                writer.write_record(&parts).ok();
                writer.flush().ok();
                let data = writer.into_inner().unwrap_or_default();
                let s = String::from_utf8(data).unwrap_or_default();
                s.trim_end_matches('\n').to_string()
            }
        }
        _ => parts.join(delimiter),
    };
    Value::Text(joined)
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

    fn char_array(chars: &[&str]) -> Value {
        Value::Array(Array::from((
            chars.iter().map(|s| text(s)).collect(),
            Level::Char,
        )))
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

    // Join applies to each element - strings pass through unchanged

    #[test]
    fn join_strings_is_noop() {
        // j on array of strings is a no-op (strings aren't arrays)
        let input = word_array(&["hello", "world"]);
        let result = Join::default().apply(input).unwrap();
        match result {
            Value::Array(arr) => {
                assert_eq!(arr.len(), 2);
                assert_eq!(arr.elements[0], text("hello"));
                assert_eq!(arr.elements[1], text("world"));
            }
            _ => panic!("expected array"),
        }
    }

    #[test]
    fn join_empty_array() {
        let input = Value::Array(Array::from((vec![], Level::Word)));
        let result = Join::default().apply(input).unwrap();
        match result {
            Value::Array(arr) => {
                assert_eq!(arr.len(), 0);
            }
            _ => panic!("expected array"),
        }
    }

    #[test]
    fn join_numbers_is_noop() {
        let input = Value::Array(Array::from((
            vec![Value::Number(1.0), Value::Number(2.0), Value::Number(3.0)],
            Level::Word,
        )));
        let result = Join::default().apply(input).unwrap();
        match result {
            Value::Array(arr) => {
                assert_eq!(arr.len(), 3);
                assert_eq!(arr.elements[0], Value::Number(1.0));
            }
            _ => panic!("expected array"),
        }
    }

    // Array of arrays: join inner arrays, preserve outer structure

    #[test]
    fn join_array_of_word_arrays() {
        // sj roundtrip: [["hello", "world"], ["foo", "bar"]] -> ["hello world", "foo bar"]
        let inner1 = word_array(&["hello", "world"]);
        let inner2 = word_array(&["foo", "bar"]);
        let outer = Value::Array(Array::from((vec![inner1, inner2], Level::Line)));

        let result = Join::default().apply(outer).unwrap();
        match result {
            Value::Array(arr) => {
                assert_eq!(arr.level, Level::Line);
                assert_eq!(arr.len(), 2);
                assert_eq!(arr.elements[0], text("hello world"));
                assert_eq!(arr.elements[1], text("foo bar"));
            }
            _ => panic!("expected array"),
        }
    }

    #[test]
    fn join_array_of_char_arrays() {
        // Chars join with empty string
        let inner1 = char_array(&["h", "i"]);
        let inner2 = char_array(&["b", "y", "e"]);
        let outer = Value::Array(Array::from((vec![inner1, inner2], Level::Word)));

        let result = Join::default().apply(outer).unwrap();
        match result {
            Value::Array(arr) => {
                assert_eq!(arr.level, Level::Word);
                assert_eq!(arr.len(), 2);
                assert_eq!(arr.elements[0], text("hi"));
                assert_eq!(arr.elements[1], text("bye"));
            }
            _ => panic!("expected array"),
        }
    }

    #[test]
    fn join_single_inner_array() {
        let inner = word_array(&["a", "b", "c"]);
        let outer = Value::Array(Array::from((vec![inner], Level::Line)));

        let result = Join::default().apply(outer).unwrap();
        match result {
            Value::Array(arr) => {
                assert_eq!(arr.level, Level::Line);
                assert_eq!(arr.len(), 1);
                assert_eq!(arr.elements[0], text("a b c"));
            }
            _ => panic!("expected array"),
        }
    }

    #[test]
    fn join_empty_inner_arrays() {
        let inner1 = Value::Array(Array::from((vec![], Level::Word)));
        let inner2 = word_array(&["a"]);
        let inner3 = Value::Array(Array::from((vec![], Level::Word)));
        let outer = Value::Array(Array::from((vec![inner1, inner2, inner3], Level::Line)));

        let result = Join::default().apply(outer).unwrap();
        match result {
            Value::Array(arr) => {
                assert_eq!(arr.level, Level::Line);
                assert_eq!(arr.len(), 3);
                assert_eq!(arr.elements[0], text(""));
                assert_eq!(arr.elements[1], text("a"));
                assert_eq!(arr.elements[2], text(""));
            }
            _ => panic!("expected array"),
        }
    }

    #[test]
    fn join_non_array_is_identity() {
        let input = text("hello");
        let result = Join::default().apply(input).unwrap();
        assert_eq!(result, text("hello"));
    }

    #[test]
    fn join_with_delimiter_on_inner_arrays() {
        // Delimiter mode also applies to each element
        let inner1 = word_array(&["a", "b", "c"]);
        let inner2 = word_array(&["d", "e"]);
        let outer = Value::Array(Array::from((vec![inner1, inner2], Level::Line)));
        let result = Join::new(JoinMode::Delimiter(",".to_string()))
            .apply(outer)
            .unwrap();
        match result {
            Value::Array(arr) => {
                assert_eq!(arr.len(), 2);
                assert_eq!(arr.elements[0], text("a,b,c"));
                assert_eq!(arr.elements[1], text("d,e"));
            }
            _ => panic!("expected array"),
        }
    }

    #[test]
    fn join_csv_on_inner_arrays() {
        let inner1 = word_array(&["a", "b,c", "d"]);
        let outer = Value::Array(Array::from((vec![inner1], Level::Line)));
        let result = Join::new(JoinMode::Csv).apply(outer).unwrap();
        match result {
            Value::Array(arr) => {
                assert_eq!(arr.len(), 1);
                assert_eq!(arr.elements[0], text(r#"a,"b,c",d"#));
            }
            _ => panic!("expected array"),
        }
    }

    // JoinDelim tests

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
