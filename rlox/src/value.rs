#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueType {
    Bool,
    Number,
    Nil,
}

#[derive(Clone, Copy, Debug, enum_as_inner::EnumAsInner)]
pub enum Value {
    Bool(bool),
    Number(f64),
    Nil,
}

impl PartialEq<Value> for Value {
    fn eq(&self, other: &Value) -> bool {
        if self.get_type() != other.get_type() {
            return false;
        }
        match self {
            Self::Bool(a) => a == other.as_bool().unwrap(),
            Self::Number(x) => x == other.as_number().unwrap(),
            Self::Nil => true,
        }
    }
}
impl Value {
    pub fn get_type(&self) -> ValueType {
        match self {
            Self::Bool(_) => ValueType::Bool,
            Self::Number(_) => ValueType::Number,
            Self::Nil => ValueType::Nil,
        }
    }
}

// TODO: Maybe the printing can be done with Display trait itself?, evaluate later.

pub trait ValuePrinter {
    fn print(&self);
}

impl ValuePrinter for Value {
    fn print(&self) {
        use Value::*;
        match self {
            Bool(x) => print!("{}", x),
            Number(x) => print!("{}", x),
            Nil => print!("nil"),
        };
    }
}
