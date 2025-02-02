mod split;

use hashbrown::HashMap;
use hashbrown::hash_map::Drain;
use crate::awk_str::{RcAwkStr};
use crate::vm::RuntimeScalar;

pub use split::{split_on_string, split_on_regex};
use crate::typing::GlobalArrayId;
use crate::util::unwrap;

#[derive(Hash, Clone, Eq, PartialEq)]
pub struct MapKey {
    key: RcAwkStr,
}

impl MapKey {
    pub fn new(key: RcAwkStr) -> Self {
        Self { key }
    }
}

struct AwkMap {
    map: HashMap<MapKey, RuntimeScalar>,
}

impl AwkMap {
    fn access(&self, key: &MapKey) -> Option<&RuntimeScalar> {
        self.map.get(key)
    }
    fn assign(&mut self, key: &MapKey, val: RuntimeScalar) -> Option<RuntimeScalar> {
        self.map.insert(key.clone(), val)
    }
    fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }
    fn in_array(&mut self, key: &MapKey) -> bool {
        self.map.contains_key(key)
    }

    fn drain(&mut self) -> Drain<'_, MapKey, RuntimeScalar> {
        self.map.drain()
    }
}

pub struct Arrays {
    arrays: Vec<AwkMap>,
}

impl Arrays {
    pub fn new(count: usize) -> Self {
        let mut arrays = Vec::with_capacity(count);
        for _ in 0..count {
            arrays.push(AwkMap::new())
        }
        Self { arrays }
    }

    pub fn clear(&mut self, arr: GlobalArrayId) -> Drain<'_, MapKey, RuntimeScalar> {
        let array = self.arrays.get_mut(arr.id).expect("array to exist based on id");
        array.drain()
    }

    pub fn access(&mut self, arr: GlobalArrayId, key: RcAwkStr) -> Option<&RuntimeScalar> {
        let array = self.arrays.get_mut(arr.id).expect("array to exist based on id");
        array.access(&MapKey::new(key))
    }

    pub fn assign(
        &mut self,
        arr: GlobalArrayId,
        indices: RcAwkStr,
        value: RuntimeScalar,
    ) -> Option<RuntimeScalar> {
        let array = unwrap(self.arrays.get_mut(arr.id));
        array.assign(&MapKey::new(indices), value)
    }

    pub fn in_array(&mut self, arr: GlobalArrayId, indices: RcAwkStr) -> bool {
        let array = unwrap(self.arrays.get_mut(arr.id));
        array.in_array(&MapKey::new(indices))
    }
}
