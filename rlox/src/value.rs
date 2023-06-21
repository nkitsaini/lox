pub type Value = f64;

// TODO: Maybe the printing can be done with Display trait itself?, evaluate later.

pub trait ValuePrinter {
    fn print(&self);
}

impl ValuePrinter for Value {
    fn print(&self) {
        print!("{}", self)
    }
}
