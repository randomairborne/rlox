#[derive(Clone, Debug, PartialEq, Copy)]
pub enum Value {
    Bool(bool),
    Number(f64),
    Object(Box),
    Nil,
}
compile_error!("https://craftinginterpreters.com/strings.html#values-and-objects");
impl Value {
    pub fn is_falsey(self) -> bool {
        match self {
            Self::Bool(val) => !val,
            Self::Nil => true,
            _ => true,
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Bool(val) => write!(f, "{val}"),
            Value::Number(val) => write!(f, "{val}"),
            Value::Nil => write!(f, "nil"),
        }
    }
}
