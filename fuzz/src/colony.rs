#![no_main]

use libfuzzer_sys::arbitrary::Arbitrary;
use libfuzzer_sys::{arbitrary, fuzz_target};
use packed_colony::Colony;
use std::collections::HashMap;

type T = u8;

#[derive(Arbitrary, Debug)]
enum Operation {
    Insert(T),
    Remove(u16),
}

fuzz_target!(|operations: Vec<Operation>| {
    let mut colony = Colony::default();
    let mut values = HashMap::new();

    for operation in operations {
        match operation {
            Operation::Insert(value) => {
                let index = colony.insert(value);
                let old = values.insert(index, value);
                assert!(old.is_none());
            }
            Operation::Remove(index) => {
                if let Some(value) = values.remove(&(index as usize)) {
                    let colony_value = colony.get(index as usize);
                    assert_eq!(*colony_value, value);
                    colony.remove(index as usize);
                }
            }
        }
    }

    for (index, value) in values {
        assert_eq!(value, *colony.get(index));
    }
});
