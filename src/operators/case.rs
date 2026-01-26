use crate::ast::Selection;
use crate::error::Result;
use crate::interpreter::Transform;
use crate::value::Value;

use super::select::apply_to_selected;

pub struct Lowercase;

impl Transform for Lowercase {
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
            Value::Text(s) => Ok(Value::Text(s.to_lowercase())),
            Value::Number(n) => Ok(Value::Number(n)),
        }
    }
}

pub struct LowercaseSelected {
    selection: Selection,
}

impl LowercaseSelected {
    pub fn new(selection: Selection) -> Self {
        Self { selection }
    }
}

impl Transform for LowercaseSelected {
    fn apply(&self, value: Value) -> Result<Value> {
        match value {
            Value::Array(arr) => apply_to_selected(arr, &self.selection, |v| Lowercase.apply(v)),
            other => Ok(other),
        }
    }
}

pub struct Uppercase;

impl Transform for Uppercase {
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
            Value::Text(s) => Ok(Value::Text(s.to_uppercase())),
            Value::Number(n) => Ok(Value::Number(n)),
        }
    }
}

pub struct UppercaseSelected {
    selection: Selection,
}

impl UppercaseSelected {
    pub fn new(selection: Selection) -> Self {
        Self { selection }
    }
}

impl Transform for UppercaseSelected {
    fn apply(&self, value: Value) -> Result<Value> {
        match value {
            Value::Array(arr) => apply_to_selected(arr, &self.selection, |v| Uppercase.apply(v)),
            other => Ok(other),
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
    fn lowercase_text() {
        let result = Lowercase.apply(text("HELLO World")).unwrap();
        assert_eq!(result, text("hello world"));
    }

    #[test]
    fn lowercase_array() {
        let input = line_array(&["HELLO", "WORLD"]);
        let result = Lowercase.apply(input).unwrap();

        match result {
            Value::Array(arr) => {
                assert_eq!(arr.elements[0], text("hello"));
                assert_eq!(arr.elements[1], text("world"));
            }
            _ => panic!("expected array"),
        }
    }

    #[test]
    fn uppercase_text() {
        let result = Uppercase.apply(text("hello World")).unwrap();
        assert_eq!(result, text("HELLO WORLD"));
    }

    #[test]
    fn lowercase_selected_single_index() {
        let input = line_array(&["HELLO", "WORLD", "FOO"]);
        let sel = Selection {
            items: vec![SelectItem::Index(0)],
        };
        let result = LowercaseSelected::new(sel).apply(input).unwrap();
        match result {
            Value::Array(arr) => {
                assert_eq!(arr.elements[0], text("hello"));
                assert_eq!(arr.elements[1], text("WORLD"));
                assert_eq!(arr.elements[2], text("FOO"));
            }
            _ => panic!("expected array"),
        }
    }

    #[test]
    fn lowercase_selected_slice() {
        let input = line_array(&["HELLO", "WORLD", "FOO"]);
        let sel = Selection {
            items: vec![SelectItem::Slice(Slice {
                start: None,
                end: Some(2),
                step: None,
            })],
        };
        let result = LowercaseSelected::new(sel).apply(input).unwrap();
        match result {
            Value::Array(arr) => {
                assert_eq!(arr.elements[0], text("hello"));
                assert_eq!(arr.elements[1], text("world"));
                assert_eq!(arr.elements[2], text("FOO"));
            }
            _ => panic!("expected array"),
        }
    }

    #[test]
    fn lowercase_selected_multi_index() {
        let input = line_array(&["HELLO", "WORLD", "FOO", "BAR"]);
        let sel = Selection {
            items: vec![SelectItem::Index(0), SelectItem::Index(2)],
        };
        let result = LowercaseSelected::new(sel).apply(input).unwrap();
        match result {
            Value::Array(arr) => {
                assert_eq!(arr.elements[0], text("hello"));
                assert_eq!(arr.elements[1], text("WORLD"));
                assert_eq!(arr.elements[2], text("foo"));
                assert_eq!(arr.elements[3], text("BAR"));
            }
            _ => panic!("expected array"),
        }
    }

    #[test]
    fn lowercase_selected_negative_index() {
        let input = line_array(&["HELLO", "WORLD", "FOO"]);
        let sel = Selection {
            items: vec![SelectItem::Index(-1)],
        };
        let result = LowercaseSelected::new(sel).apply(input).unwrap();
        match result {
            Value::Array(arr) => {
                assert_eq!(arr.elements[0], text("HELLO"));
                assert_eq!(arr.elements[1], text("WORLD"));
                assert_eq!(arr.elements[2], text("foo"));
            }
            _ => panic!("expected array"),
        }
    }

    #[test]
    fn lowercase_selected_non_array_is_identity() {
        let input = text("HELLO");
        let sel = Selection {
            items: vec![SelectItem::Index(0)],
        };
        let result = LowercaseSelected::new(sel).apply(input).unwrap();
        assert_eq!(result, text("HELLO"));
    }

    #[test]
    fn uppercase_selected_single_index() {
        let input = line_array(&["hello", "world", "foo"]);
        let sel = Selection {
            items: vec![SelectItem::Index(0)],
        };
        let result = UppercaseSelected::new(sel).apply(input).unwrap();
        match result {
            Value::Array(arr) => {
                assert_eq!(arr.elements[0], text("HELLO"));
                assert_eq!(arr.elements[1], text("world"));
                assert_eq!(arr.elements[2], text("foo"));
            }
            _ => panic!("expected array"),
        }
    }

    #[test]
    fn uppercase_selected_slice() {
        let input = line_array(&["hello", "world", "foo"]);
        let sel = Selection {
            items: vec![SelectItem::Slice(Slice {
                start: Some(1),
                end: None,
                step: None,
            })],
        };
        let result = UppercaseSelected::new(sel).apply(input).unwrap();
        match result {
            Value::Array(arr) => {
                assert_eq!(arr.elements[0], text("hello"));
                assert_eq!(arr.elements[1], text("WORLD"));
                assert_eq!(arr.elements[2], text("FOO"));
            }
            _ => panic!("expected array"),
        }
    }
}
