use regex::Regex;

use crate::error::Result;
use crate::interpreter::Transform;
use crate::value::{Array, Value};

pub struct Filter {
    pattern: Regex,
    negate: bool,
}

impl Filter {
    pub fn new(pattern: Regex, negate: bool) -> Self {
        Self { pattern, negate }
    }
}

impl Transform for Filter {
    fn apply(&self, value: Value) -> Result<Value> {
        match value {
            Value::Array(arr) => {
                let filtered: Vec<Value> = arr
                    .elements
                    .into_iter()
                    .filter(|elem| {
                        let text = match elem {
                            Value::Text(s) => s.clone(),
                            Value::Number(n) => n.to_string(),
                            Value::Array(inner) => inner.to_string(),
                        };
                        let matches = self.pattern.is_match(&text);
                        if self.negate { !matches } else { matches }
                    })
                    .collect();
                Ok(Value::Array(Array::from((filtered, arr.level))))
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

    #[test]
    fn filter_keep_matching() {
        let input = Value::Array(Array::from((
            vec![text("apple"), text("banana"), text("apricot")],
            Level::Line,
        )));
        let filter = Filter::new(Regex::new("^a").unwrap(), false);
        let result = filter.apply(input).unwrap();

        match result {
            Value::Array(arr) => {
                assert_eq!(arr.len(), 2);
                assert_eq!(arr.elements[0], text("apple"));
                assert_eq!(arr.elements[1], text("apricot"));
            }
            _ => panic!("expected array"),
        }
    }

    #[test]
    fn filter_remove_matching() {
        let input = Value::Array(Array::from((
            vec![text("apple"), text("banana"), text("apricot")],
            Level::Line,
        )));
        let filter = Filter::new(Regex::new("^a").unwrap(), true);
        let result = filter.apply(input).unwrap();

        match result {
            Value::Array(arr) => {
                assert_eq!(arr.len(), 1);
                assert_eq!(arr.elements[0], text("banana"));
            }
            _ => panic!("expected array"),
        }
    }

    #[test]
    fn filter_no_matches() {
        let input = Value::Array(Array::from((
            vec![text("apple"), text("banana"), text("cherry")],
            Level::Line,
        )));
        let filter = Filter::new(Regex::new("^z").unwrap(), false);
        let result = filter.apply(input).unwrap();

        match result {
            Value::Array(arr) => {
                assert!(arr.is_empty());
            }
            _ => panic!("expected array"),
        }
    }

    #[test]
    fn filter_all_match() {
        let input = Value::Array(Array::from((
            vec![text("apple"), text("apricot"), text("avocado")],
            Level::Line,
        )));
        let filter = Filter::new(Regex::new("^a").unwrap(), false);
        let result = filter.apply(input).unwrap();

        match result {
            Value::Array(arr) => {
                assert_eq!(arr.len(), 3);
            }
            _ => panic!("expected array"),
        }
    }

    #[test]
    fn filter_with_numbers() {
        let input = Value::Array(Array::from((
            vec![
                Value::Number(10.0),
                Value::Number(20.0),
                Value::Number(100.0),
            ],
            Level::Line,
        )));
        let filter = Filter::new(Regex::new("^1").unwrap(), false);
        let result = filter.apply(input).unwrap();

        match result {
            Value::Array(arr) => {
                assert_eq!(arr.len(), 2);
                assert_eq!(arr.elements[0], Value::Number(10.0));
                assert_eq!(arr.elements[1], Value::Number(100.0));
            }
            _ => panic!("expected array"),
        }
    }

    #[test]
    fn filter_preserves_level() {
        let input = Value::Array(Array::from((
            vec![text("apple"), text("banana")],
            Level::Word,
        )));
        let filter = Filter::new(Regex::new("a").unwrap(), false);
        let result = filter.apply(input).unwrap();

        match result {
            Value::Array(arr) => {
                assert_eq!(arr.level, Level::Word);
            }
            _ => panic!("expected array"),
        }
    }

    #[test]
    fn filter_non_array_is_identity() {
        let input = text("hello");
        let filter = Filter::new(Regex::new("e").unwrap(), false);
        let result = filter.apply(input).unwrap();
        assert_eq!(result, text("hello"));
    }

    #[test]
    fn filter_empty_array() {
        let input = Value::Array(Array::from((vec![], Level::Line)));
        let filter = Filter::new(Regex::new("a").unwrap(), false);
        let result = filter.apply(input).unwrap();

        match result {
            Value::Array(arr) => {
                assert!(arr.is_empty());
            }
            _ => panic!("expected array"),
        }
    }

    #[test]
    fn filter_regex_contains() {
        let input = Value::Array(Array::from((
            vec![text("ERROR: fail"), text("INFO: ok"), text("ERROR: crash")],
            Level::Line,
        )));
        let filter = Filter::new(Regex::new("ERROR").unwrap(), false);
        let result = filter.apply(input).unwrap();

        match result {
            Value::Array(arr) => {
                assert_eq!(arr.len(), 2);
                assert_eq!(arr.elements[0], text("ERROR: fail"));
                assert_eq!(arr.elements[1], text("ERROR: crash"));
            }
            _ => panic!("expected array"),
        }
    }
}
