use crate::{prelude::Value, value::LoxObject};

pub struct Entry<'a> {
    key: LoxObject<'a>,
    value: Value<'a>,
}
pub struct HashTable<'a> {
    entries: Vec<Entry<'a>>,
}

impl HashTable {
    pub fn clear(&mut self) {
        self.entries.clear();
    }
}
