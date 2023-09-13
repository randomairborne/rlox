use std::rc::Rc;
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Bool(bool),
    Number(f64),
    Str(Rc<str>),
    Nil,
}

impl Value {
    pub fn is_falsey(&self) -> bool {
        match self {
            Self::Bool(val) => !val,
            Self::Nil => true,
            _ => true,
        }
    }
    pub fn is_bool(&self) -> bool {
        matches!(self, Value::Bool(_))
    }
    pub fn is_number(&self) -> bool {
        matches!(self, Value::Number(_))
    }
    pub fn is_str(&self) -> bool {
        matches!(self, Value::Str(_))
    }
    pub fn is_nil(&self) -> bool {
        matches!(self, Value::Nil)
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Bool(val) => write!(f, "{val}"),
            Value::Number(val) => write!(f, "{val}"),
            Value::Str(val) => write!(f, "{val}"),
            Value::Nil => write!(f, "nil"),
        }
    }
}
