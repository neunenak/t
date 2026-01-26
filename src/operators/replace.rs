use regex::Regex;

use crate::ast::Selection;
use crate::error::Result;
use crate::interpreter::Transform;
use crate::value::Value;

use super::select::apply_to_selected;

pub struct Replace {
    pattern: Regex,
    replacement: String,
    selection: Option<Selection>,
}

impl Replace {
    pub fn new(pattern: Regex, replacement: String, selection: Option<Selection>) -> Self {
        Self {
            pattern,
            replacement,
            selection,
        }
    }

    fn replace_value(&self, value: Value) -> Result<Value> {
        match value {
            Value::Array(mut arr) => {
                arr.elements = arr
                    .elements
                    .into_iter()
                    .map(|v| self.replace_value(v))
                    .collect::<Result<Vec<_>>>()?;
                Ok(Value::Array(arr))
            }
            Value::Text(s) => Ok(Value::Text(
                self.pattern.replace_all(&s, &self.replacement).into_owned(),
            )),
            Value::Number(n) => Ok(Value::Number(n)),
        }
    }
}

impl Transform for Replace {
    fn apply(&self, value: Value) -> Result<Value> {
        match &self.selection {
            Some(sel) => match value {
                Value::Array(arr) => apply_to_selected(arr, sel, |v| self.replace_value(v)),
                other => Ok(other),
            },
            None => self.replace_value(value),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{SelectItem, Selection, Slice};
    use crate::value::{Array, Level};

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
    fn replace_basic() {
        let input = line_array(&["foo bar", "foo baz"]);
        let replace = Replace::new(Regex::new("foo").unwrap(), "qux".to_string(), None);
        let result = replace.apply(input).unwrap();
        match result {
            Value::Array(arr) => {
                assert_eq!(arr.elements[0], text("qux bar"));
                assert_eq!(arr.elements[1], text("qux baz"));
            }
            _ => panic!("expected array"),
        }
    }

    #[test]
    fn replace_all_occurrences() {
        let input = text("foo foo foo");
        let replace = Replace::new(Regex::new("foo").unwrap(), "bar".to_string(), None);
        let result = replace.apply(input).unwrap();
        assert_eq!(result, text("bar bar bar"));
    }

    #[test]
    fn replace_empty_replacement() {
        let input = text("ERROR: something");
        let replace = Replace::new(Regex::new("ERROR: ").unwrap(), "".to_string(), None);
        let result = replace.apply(input).unwrap();
        assert_eq!(result, text("something"));
    }

    #[test]
    fn replace_with_selection() {
        let input = line_array(&["foo", "foo", "foo"]);
        let replace = Replace::new(
            Regex::new("foo").unwrap(),
            "bar".to_string(),
            Some(Selection {
                items: vec![SelectItem::Index(0)],
            }),
        );
        let result = replace.apply(input).unwrap();
        match result {
            Value::Array(arr) => {
                assert_eq!(arr.elements[0], text("bar"));
                assert_eq!(arr.elements[1], text("foo"));
                assert_eq!(arr.elements[2], text("foo"));
            }
            _ => panic!("expected array"),
        }
    }

    #[test]
    fn replace_with_slice_selection() {
        let input = line_array(&["foo", "foo", "foo"]);
        let replace = Replace::new(
            Regex::new("foo").unwrap(),
            "bar".to_string(),
            Some(Selection {
                items: vec![SelectItem::Slice(Slice {
                    start: Some(1),
                    end: None,
                    step: None,
                })],
            }),
        );
        let result = replace.apply(input).unwrap();
        match result {
            Value::Array(arr) => {
                assert_eq!(arr.elements[0], text("foo"));
                assert_eq!(arr.elements[1], text("bar"));
                assert_eq!(arr.elements[2], text("bar"));
            }
            _ => panic!("expected array"),
        }
    }

    #[test]
    fn replace_regex_capture_groups() {
        let input = text("hello world");
        let replace = Replace::new(
            Regex::new("(\\w+) (\\w+)").unwrap(),
            "$2 $1".to_string(),
            None,
        );
        let result = replace.apply(input).unwrap();
        assert_eq!(result, text("world hello"));
    }

    #[test]
    fn replace_non_array_with_selection_is_identity() {
        let input = text("foo");
        let replace = Replace::new(
            Regex::new("foo").unwrap(),
            "bar".to_string(),
            Some(Selection {
                items: vec![SelectItem::Index(0)],
            }),
        );
        let result = replace.apply(input).unwrap();
        assert_eq!(result, text("foo"));
    }
}
