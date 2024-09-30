use std::{fmt::Debug, mem::swap, slice::Iter};

use crate::{
    object::{HeapSize, ObjString},
    value::RuntimeValue,
};

pub const MAX_TABLE_LOAD: f32 = 0.75;

#[derive(Debug)]
pub struct Table<T: Clone + Debug + HeapSize = RuntimeValue> {
    count: usize,
    entries: Vec<Option<TableEntry<T>>>,
}

impl<T: Clone + Debug + HeapSize> Default for Table<T> {
    fn default() -> Self {
        Self {
            count: 0,
            entries: vec![None; 8],
        }
    }
}

impl<T: Clone + Debug + HeapSize> HeapSize for Table<T> {
    fn size(&self) -> usize {
        size_of::<usize>()
            + self
                .entries
                .iter()
                .map(|x| match x {
                    Some(e) => e.size(),
                    None => size_of_val(x),
                })
                .sum::<usize>()
    }
}

impl<T: Clone + Debug + HeapSize> Table<T> {
    pub fn get(&self, key: &ObjString) -> Option<&T> {
        if self.count == 0 {
            return None;
        }

        let index = find_entry_index(&self.entries, key);
        match &self.entries[index] {
            Some(e) => match &e.key {
                Some(_) => match &e.value {
                    Some(v) => Some(v),
                    None => None,
                },
                None => None,
            },
            None => None,
        }
    }

    pub fn get_mut(&mut self, key: &ObjString) -> Option<&mut T> {
        if self.count == 0 {
            return None;
        }

        let index = find_entry_index(&self.entries, key);
        match self.entries[index].as_mut() {
            Some(e) => match &e.key {
                Some(_) => match &mut e.value {
                    Some(v) => Some(v),
                    None => None,
                },
                None => None,
            },
            None => None,
        }
    }

    pub fn insert(&mut self, key: ObjString, value: T) -> bool {
        if (self.count + 1) as f32 > self.entries.len() as f32 * MAX_TABLE_LOAD {
            self.adjust_capacity();
        }

        let index = find_entry_index(&self.entries, &key);
        let mut is_new_key = false;
        match &mut self.entries[index] {
            Some(e) => {
                if e.key.is_none() {
                    is_new_key = true;
                }
                e.key = Some(key);
                e.value = Some(value);
            }
            None => {
                is_new_key = true;
                self.entries[index] = Some(TableEntry {
                    key: Some(key),
                    value: Some(value),
                });
                self.count += 1;
            }
        }

        is_new_key
    }

    pub fn remove(&mut self, key: &ObjString) -> bool {
        if self.count == 0 {
            return false;
        }

        let index = find_entry_index(&self.entries, key);
        match &mut self.entries[index] {
            Some(e) => {
                if e.key.is_none() {
                    return false;
                }
                e.key = None;
                e.value = None;
                true
            }
            None => false,
        }
    }

    pub fn find_string(&self, chars: &str, hash: u32) -> Option<&ObjString> {
        if self.count == 0 {
            return None;
        }

        let mut index = hash as usize & (self.entries.len() - 1);
        loop {
            match &self.entries[index] {
                Some(e) => {
                    let Some(ref key) = e.key else {
                        continue;
                    };
                    if key.chars.len() == chars.len() && key.hash == hash && key.chars == chars {
                        return Some(key);
                    }
                }
                None => return None,
            }
            index = (index + 1) & (self.entries.len() - 1);
        }
    }

    pub fn iter(&self) -> Iter<'_, Option<TableEntry<T>>> {
        self.entries.iter()
    }

    pub fn values(&self) -> Vec<&T> {
        self.entries
            .iter()
            .filter_map(|x| match x.as_ref() {
                Some(e) => e.value.as_ref(),
                None => None,
            })
            .collect()
    }

    fn adjust_capacity(&mut self) {
        let mut entries = vec![None; self.entries.len() * 2];
        swap(&mut self.entries, &mut entries);
        self.count = 0;
        for entry in entries {
            match entry {
                Some(e) => {
                    let Some(ref key) = e.key else {
                        continue;
                    };
                    let index = find_entry_index(&self.entries, key);
                    self.entries[index] = Some(e);
                    self.count += 1;
                }
                None => continue,
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct TableEntry<T: Clone + Debug + HeapSize> {
    pub key: Option<ObjString>,
    pub value: Option<T>,
}

impl<T: Clone + Debug + HeapSize> HeapSize for TableEntry<T> {
    fn size(&self) -> usize {
        let value_size = match &self.value {
            Some(v) => v.size(),
            None => size_of::<Option<T>>(),
        };
        value_size
            + match &self.key {
                Some(k) => k.size(),
                None => size_of::<Option<ObjString>>(),
            }
    }
}

fn find_entry_index<T: Clone + Debug + HeapSize>(
    entries: &Vec<Option<TableEntry<T>>>,
    key: &ObjString,
) -> usize {
    let mut index = (key.hash as usize) & (entries.len() - 1);
    let mut tombstone = None;
    loop {
        match &entries[index] {
            Some(entry) => match &entry.key {
                Some(k) => {
                    if k == key {
                        return index;
                    }
                }
                None => tombstone = Some(index),
            },
            None => match tombstone {
                Some(t) => return t,
                None => return index,
            },
        }

        index = (index + 1) & (entries.capacity() - 1);
    }
}

#[cfg(test)]
mod test {
    use crate::value::RuntimeValue;

    use super::*;

    #[test]
    fn it_return_none_when_empty() {
        let table = Table::<RuntimeValue>::default();
        let key = "test".into();
        assert!(table.get(&key).is_none());
    }

    #[test]
    fn it_inserts_a_value() {
        let mut table = Table::default();
        let key = "test".into();
        let value = RuntimeValue::Nil;
        assert!(table.insert(key, value));
        assert_eq!(table.count, 1);
        let key = "test".into();
        let value = table.get(&key).expect("Failed to get inserted value");
        assert_eq!(value, &RuntimeValue::Nil);
    }

    #[test]
    fn it_overwrites_a_value() {
        let mut table = Table::default();
        let key = "test".into();
        let value = RuntimeValue::Nil;
        assert!(table.insert(key, value));
        let key = "test".into();
        let value = table.get(&key).expect("Failed to get inserted value");
        assert_eq!(value, &RuntimeValue::Nil);
        let value = RuntimeValue::Bool(true);
        assert!(!table.insert(key, value));
        let key = "test".into();
        let value = table.get(&key).expect("Failed to get inserted value");
        assert_eq!(value, &RuntimeValue::Bool(true));
    }

    #[test]
    fn it_retrieves_the_correct_value() {
        let mut table = Table::default();
        let key = "nil".into();
        let value = RuntimeValue::Nil;
        assert!(table.insert(key, value));
        let key = "boolean".into();
        let value = RuntimeValue::Bool(false);
        assert!(table.insert(key, value));
        let key = "number".into();
        let value = RuntimeValue::Number(0.1);
        assert!(table.insert(key, value));
        let key = "nil".into();
        let value = table.get(&key).expect("Failed to get inserted value");
        assert_eq!(value, &RuntimeValue::Nil);
        let key = "boolean".into();
        let value = table.get(&key).expect("Failed to get inserted value");
        assert_eq!(value, &RuntimeValue::Bool(false));
        let key = "number".into();
        let value = table.get(&key).expect("Failed to get inserted value");
        assert_eq!(value, &RuntimeValue::Number(0.1));
    }

    #[test]
    fn it_grows_in_size() {
        let mut table = Table::default();
        for i in 0..128 {
            let key = format!("{i}").into();
            let value = RuntimeValue::Number(i as f64);
            assert!(table.insert(key, value));
        }
        for i in 0..128 {
            let key = format!("{i}").into();
            let value = table.get(&key).expect("Failed to get inserted value");
            assert_eq!(value, &RuntimeValue::Number(i as f64));
        }
        assert_eq!(table.count, 128);
    }

    #[test]
    fn it_removes_a_value() {
        let mut table = Table::default();
        for i in 0..128 {
            let key = format!("{i}").into();
            let value = RuntimeValue::Number(i as f64);
            assert!(table.insert(key, value));
        }
        for i in 0..128 {
            let key = format!("{i}").into();
            let value = table.get(&key).expect("Failed to get inserted value");
            assert_eq!(value, &RuntimeValue::Number(i as f64));
        }
        assert!(table.remove(&("32".into())));

        for i in 0..128 {
            let key = format!("{i}").into();
            if i == 32 {
                assert!(table.get(&key).is_none());
                let value = RuntimeValue::Number(i as f64);
                assert!(table.insert(key.clone(), value));
            }
            let value = table.get(&key).expect("Failed to get inserted value");
            assert_eq!(value, &RuntimeValue::Number(i as f64));
        }
        assert_eq!(table.count, 128);
    }

    #[test]
    fn it_removes_non_existent_value() {
        let mut table = Table::default();
        assert!(!table.remove(&("test".into())));
        assert!(table.insert("test".into(), RuntimeValue::Nil));
        assert!(table.remove(&("test".into())));
        assert!(!table.remove(&("test".into())));
    }

    #[test]
    fn it_finds_a_string_key() {
        let mut table = Table::default();
        assert!(table.insert("test".into(), RuntimeValue::Nil));
        let string = ObjString::from("test");
        let key = table
            .find_string(&string.chars, string.hash)
            .expect("Failed to find string");
        assert_eq!(key, &("test".into()));
    }
}
