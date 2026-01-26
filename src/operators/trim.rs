use crate::ast::Selection;
use crate::error::Result;
use crate::interpreter::Transform;
use crate::value::Value;

use super::select::apply_to_selected;

pub struct Trim;

impl Transform for Trim {
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
            Value::Text(s) => Ok(Value::Text(s.trim().to_string())),
            Value::Number(n) => Ok(Value::Number(n)),
        }
    }
}

pub struct TrimSelected {
    selection: Selection,
}

impl TrimSelected {
    pub fn new(selection: Selection) -> Self {
        Self { selection }
    }
}

impl Transform for TrimSelected {
    fn apply(&self, value: Value) -> Result<Value> {
        match value {
            Value::Array(arr) => apply_to_selected(arr, &self.selection, |v| Trim.apply(v)),
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
    fn trim_text() {
        let result = Trim.apply(text("  hello  ")).unwrap();
        assert_eq!(result, text("hello"));
    }

    #[test]
    fn trim_text_with_tabs_and_newlines() {
        let result = Trim.apply(text("\thello\n")).unwrap();
        assert_eq!(result, text("hello"));
    }

    #[test]
    fn trim_array() {
        let input = line_array(&["  hello  ", "\tworld\n"]);
        let result = Trim.apply(input).unwrap();

        match result {
            Value::Array(arr) => {
                assert_eq!(arr.elements[0], text("hello"));
                assert_eq!(arr.elements[1], text("world"));
            }
            _ => panic!("expected array"),
        }
    }

    #[test]
    fn trim_preserves_numbers() {
        let result = Trim.apply(Value::Number(42.0)).unwrap();
        assert_eq!(result, Value::Number(42.0));
    }

    #[test]
    fn trim_selected_single_index() {
        let input = Value::Array(Array::from((
            vec![text("  hello  "), text("  world  "), text("  foo  ")],
            Level::Line,
        )));
        let sel = Selection {
            items: vec![SelectItem::Index(0)],
        };
        let result = TrimSelected::new(sel).apply(input).unwrap();
        match result {
            Value::Array(arr) => {
                assert_eq!(arr.elements[0], text("hello"));
                assert_eq!(arr.elements[1], text("  world  "));
                assert_eq!(arr.elements[2], text("  foo  "));
            }
            _ => panic!("expected array"),
        }
    }

    #[test]
    fn trim_selected_slice() {
        let input = Value::Array(Array::from((
            vec![text("  hello  "), text("  world  "), text("  foo  ")],
            Level::Line,
        )));
        let sel = Selection {
            items: vec![SelectItem::Slice(Slice {
                start: Some(1),
                end: None,
                step: None,
            })],
        };
        let result = TrimSelected::new(sel).apply(input).unwrap();
        match result {
            Value::Array(arr) => {
                assert_eq!(arr.elements[0], text("  hello  "));
                assert_eq!(arr.elements[1], text("world"));
                assert_eq!(arr.elements[2], text("foo"));
            }
            _ => panic!("expected array"),
        }
    }

    #[test]
    fn trim_selected_multi_index() {
        let input = Value::Array(Array::from((
            vec![
                text("  hello  "),
                text("  world  "),
                text("  foo  "),
                text("  bar  "),
            ],
            Level::Line,
        )));
        let sel = Selection {
            items: vec![SelectItem::Index(0), SelectItem::Index(2)],
        };
        let result = TrimSelected::new(sel).apply(input).unwrap();
        match result {
            Value::Array(arr) => {
                assert_eq!(arr.elements[0], text("hello"));
                assert_eq!(arr.elements[1], text("  world  "));
                assert_eq!(arr.elements[2], text("foo"));
                assert_eq!(arr.elements[3], text("  bar  "));
            }
            _ => panic!("expected array"),
        }
    }

    #[test]
    fn trim_selected_negative_index() {
        let input = Value::Array(Array::from((
            vec![text("  hello  "), text("  world  "), text("  foo  ")],
            Level::Line,
        )));
        let sel = Selection {
            items: vec![SelectItem::Index(-1)],
        };
        let result = TrimSelected::new(sel).apply(input).unwrap();
        match result {
            Value::Array(arr) => {
                assert_eq!(arr.elements[0], text("  hello  "));
                assert_eq!(arr.elements[1], text("  world  "));
                assert_eq!(arr.elements[2], text("foo"));
            }
            _ => panic!("expected array"),
        }
    }

    #[test]
    fn trim_selected_non_array_is_identity() {
        let input = text("  hello  ");
        let sel = Selection {
            items: vec![SelectItem::Index(0)],
        };
        let result = TrimSelected::new(sel).apply(input).unwrap();
        assert_eq!(result, text("  hello  "));
    }
}
