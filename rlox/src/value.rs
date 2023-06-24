use enum_kinds;
use std::rc::Rc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueType {
    Bool,
    Number,
    Nil,
    Object,
}

#[derive(Debug, Clone, enum_as_inner::EnumAsInner, enum_kinds::EnumKind)]
#[enum_kind(LoxObjectKind)]
pub enum LoxObject<'a> {
    /// A Lox String can either be interned. Where it'll be shared across all
    /// Or it can be an intermediate result like in (a+b+c) result of a+b is not important
    String {
        value: String,
        hash: u32,
    },
    Random(&'a u32),
}

impl<'a> LoxObject<'a> {
    pub fn new_string(value: String) -> LoxObject<'a> {
        let hash = hash_string(&value);
        LoxObject::String { value, hash }
    }
}

impl<'a> LoxObject<'a> {
    pub fn kind(&self) -> LoxObjectKind {
        LoxObjectKind::from(self)
    }
}

#[derive(Debug, Clone, enum_as_inner::EnumAsInner)]
pub enum Value<'a> {
    Bool(bool),
    Number(f64),
    Nil,
    Object(Rc<LoxObject<'a>>),
}

impl<'a> PartialEq<Value<'a>> for Value<'a> {
    fn eq(&self, other: &Value) -> bool {
        if self.get_type() != other.get_type() {
            return false;
        }
        match self {
            Self::Bool(a) => a == other.as_bool().unwrap(),
            Self::Number(x) => x == other.as_number().unwrap(),
            Self::Nil => true,
            Self::Object(x) => {
                x.as_string().unwrap() == other.as_object().unwrap().as_string().unwrap()
            }
        }
    }
}
impl<'a> Value<'a> {
    pub fn get_type(&self) -> ValueType {
        match self {
            Self::Bool(_) => ValueType::Bool,
            Self::Number(_) => ValueType::Number,
            Self::Nil => ValueType::Nil,
            Self::Object(_) => ValueType::Object,
        }
    }
}

// TODO: Maybe the printing can be done with Display trait itself?, evaluate later.

pub trait ValuePrinter {
    fn print(&self);
}

impl<'a> ValuePrinter for Value<'a> {
    fn print(&self) {
        use Value::*;
        let obj = match self {
            Bool(x) => {
                print!("{}", x);
                return;
            }
            Number(x) => {
                print!("{}", x);
                return;
            }
            Nil => {
                print!("nil");
                return;
            }
            Object(x) => x,
        };

        match obj.as_ref() {
            LoxObject::String { value, hash: _ } => print!("{}", value),
            _ => unreachable!(),
        }
    }
}

pub fn hash_string(val: &str) -> u32 {
    let mut hash = 2166136261u32;
    for i in 0..val.len() {
        hash ^= val.bytes().nth(i).unwrap() as u32;
        hash = hash.wrapping_mul(16777619);
    }
    hash
}
