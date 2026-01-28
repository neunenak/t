use crate::error::Result;
use crate::interpreter::Transform;
use crate::value::{Array, Level, Value};

/// Split mode determines how `s` splits text at the line level.
#[derive(Debug, Clone, Default)]
pub enum SplitMode {
    /// Split on whitespace (default)
    #[default]
    Whitespace,
    /// Split on a specific delimiter
    Delimiter(String),
    /// Split as CSV fields
    Csv,
}

pub struct Split {
    mode: SplitMode,
}

impl Split {
    pub fn new(mode: SplitMode) -> Self {
        Self { mode }
    }
}

impl Default for Split {
    fn default() -> Self {
        Self {
            mode: SplitMode::Whitespace,
        }
    }
}

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
            Value::Text(s) => Ok(split_text(&s, Level::Line, &self.mode)),
            Value::Number(n) => Ok(Value::Number(n)),
        }
    }
}

fn split_text(s: &str, level: Level, mode: &SplitMode) -> Value {
    let new_level = level.split_into();
    let elements: Vec<Value> = match level {
        Level::File => s
            .lines()
            .map(|line| Value::Text(line.to_string()))
            .collect(),
        Level::Line => split_line(s, mode),
        Level::Word => s.chars().map(|c| Value::Text(c.to_string())).collect(),
        Level::Char => vec![Value::Text(s.to_string())],
    };
    Value::Array(Array::from((elements, new_level)))
}

fn split_line(s: &str, mode: &SplitMode) -> Vec<Value> {
    match mode {
        SplitMode::Whitespace => s
            .split_whitespace()
            .map(|word| Value::Text(word.to_string()))
            .collect(),
        SplitMode::Delimiter(delim) => s
            .split(delim.as_str())
            .map(|part| Value::Text(part.to_string()))
            .collect(),
        SplitMode::Csv => {
            let mut reader = csv::ReaderBuilder::new()
                .has_headers(false)
                .from_reader(s.as_bytes());
            let mut record = csv::StringRecord::new();
            if reader.read_record(&mut record).unwrap_or(false) {
                record.iter().map(|f| Value::Text(f.to_string())).collect()
            } else {
                vec![]
            }
        }
    }
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
        let result = Split::default().apply(text("hello world")).unwrap();
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
        let result = Split::default().apply(input).unwrap();

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
    fn split_with_delimiter() {
        let input = text("a,b,c");
        let result = Split::new(SplitMode::Delimiter(",".to_string()))
            .apply(input)
            .unwrap();
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
    fn split_csv_simple() {
        let input = text("a,b,c");
        let result = Split::new(SplitMode::Csv).apply(input).unwrap();
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
    fn split_csv_quoted_fields() {
        let input = text(r#"a,"b,c",d"#);
        let result = Split::new(SplitMode::Csv).apply(input).unwrap();
        match result {
            Value::Array(arr) => {
                assert_eq!(arr.len(), 3);
                assert_eq!(arr.elements[0], text("a"));
                assert_eq!(arr.elements[1], text("b,c"));
                assert_eq!(arr.elements[2], text("d"));
            }
            _ => panic!("expected array"),
        }
    }

    #[test]
    fn split_csv_quoted_with_quotes() {
        let input = text(r#"a,"b""c",d"#);
        let result = Split::new(SplitMode::Csv).apply(input).unwrap();
        match result {
            Value::Array(arr) => {
                assert_eq!(arr.len(), 3);
                assert_eq!(arr.elements[0], text("a"));
                assert_eq!(arr.elements[1], text(r#"b"c"#));
                assert_eq!(arr.elements[2], text("d"));
            }
            _ => panic!("expected array"),
        }
    }

    #[test]
    fn split_csv_empty() {
        let input = text("");
        let result = Split::new(SplitMode::Csv).apply(input).unwrap();
        match result {
            Value::Array(arr) => {
                assert_eq!(arr.len(), 0);
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
