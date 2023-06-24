use std::rc::Rc;

use crate::{prelude::Value, value::LoxObject};

const TABLE_MAX_LOAD: f32 = 0.75;

pub struct Entry<'a> {
    key: Option<Rc<LoxObject<'a>>>,
    value: Value<'a>,
}

impl<'a> Entry<'a> {
    pub fn all_none() -> Self {
        Entry {
            key: None,
            value: Value::Nil,
        }
    }
}
pub struct HashTable<'a> {
    /// Total filled entries in table
    count: usize,
    entries: Vec<Entry<'a>>,
}

impl<'a> HashTable<'a> {
    pub fn new() -> Self {
        return HashTable {
            count: 0,
            entries: vec![],
        };
    }
    /// Returns `true` if a new key is added
    /// `false` is existing key is updated
    pub fn set(&mut self, key: Rc<LoxObject<'a>>, value: Value<'a>) -> bool {
        // TODO(perf): Control the vector capacity grow instead of looking at len
        if self.count as f32 + 1.0 > self.entries.len() as f32 * TABLE_MAX_LOAD {
            let old_capacity = self.entries.len();
            let new_capacity = old_capacity * 2;
            self.adjust_capacity(new_capacity);
        }
        let entry: &mut Entry = Self::find_entry(&mut self.entries, &key);
        let is_new_key = entry.key.is_none();
        if is_new_key && entry.value.is_nil() {
            self.count += 1;
        }

        entry.key = Some(key);
        entry.value = value;
        return is_new_key;
    }

    pub fn get(&mut self, key: &Rc<LoxObject<'a>>) -> Option<&mut Value<'a>> {
        if self.count == 0 {
            return None;
        }
        let entry = Self::find_entry(&mut self.entries, key);
        if let Some(_) = &entry.key {
            return Some(&mut entry.value);
        }
        None
    }

    pub fn delete(&mut self, key: &Rc<LoxObject<'a>>) -> bool {
        if self.count == 0 {
            return false;
        }
        let entry = Self::find_entry(&mut self.entries, key);
        if entry.key.is_none() {
            return false;
        }

        // Place a tombstone in the entry
        entry.key = None;
        entry.value = Value::Bool(true);
        true
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    fn find_entry<'b>(entries: &'b mut [Entry<'a>], key: &Rc<LoxObject>) -> &'b mut Entry<'a> {
        let mut index = key.as_string().unwrap().1 % entries.len() as u32;

        let mut tombstone_index = None;
        loop {
            let entry = entries.get(index as usize).unwrap();
            match &entry.key {
                None => {
                    if entry.value.is_nil() {
                        if let Some(i) = tombstone_index {
                            unsafe {
                                return entries.get_unchecked_mut(i as usize);
                            };
                        } else {
                            unsafe {
                                return entries.get_unchecked_mut(index as usize);
                            };
                        }
                    } else {
                        if tombstone_index.is_none() {
                            tombstone_index = Some(index);
                        }
                    }
                }
                Some(x) => {
                    unsafe {
                        return entries.get_unchecked_mut(index as usize);
                    };
                }
            }
            match &entry.key {
                Some(x) if Rc::ptr_eq(x, key) => unsafe {
                    return entries.get_unchecked_mut(index as usize);
                },
                None => unsafe {
                    return entries.get_unchecked_mut(index as usize);
                },
                _ => {}
            }

            index += 1;
            index %= entries.len() as u32;
        }
    }

    fn adjust_capacity(&mut self, new_capacity: usize) {
        let mut entries = Vec::with_capacity(new_capacity);
        entries.extend((0..new_capacity).map(|_| Entry::all_none()));

        std::mem::swap(&mut entries, &mut self.entries);
        self.count = 0;
        let old_entries = entries;

        for entry in old_entries.into_iter() {
            if let Some(x) = entry.key {
                let dest = Self::find_entry(&mut self.entries, &x);
                dest.key = Some(x);
                dest.value = entry.value;
                self.count += 1;
            }
        }
    }

    fn table_add_all(from: &mut Self, to: &mut Self) {
        for i in 0..from.entries.len() {
            let entry = &from.entries[i];
            if let Some(x) = entry.key.as_ref() {
                to.set(x.clone(), entry.value.clone());
            }
        }
    }
}