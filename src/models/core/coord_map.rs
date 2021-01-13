use std::collections::HashMap;

// maps two integers to a generic
#[derive(Clone)]
pub struct CoordMap<T> {
    map: HashMap<i32, HashMap<i32, T>> // x by z
}

impl<T> CoordMap<T> {
    pub fn new() -> CoordMap<T> {
        CoordMap { map: HashMap::new() }
    }

    pub fn get(&self, x: i32, z: i32) -> Option<&T> {
        let item = self.map.get(&x);
        if item.is_none() {
            return None
        }

        item.unwrap().get(&z)
    }

    pub fn get_mut(&mut self, x: i32, z: i32) -> Option<&mut T> {
        let item = self.map.get_mut(&x);
        if item.is_none() {
            return None
        }

        item.unwrap().get_mut(&z)
    }

    pub fn contains(&self, x: i32, z: i32) -> bool {
        let item = self.map.get(&x);
        if item.is_none() {
            return false
        }

        item.unwrap().contains_key(&z)
    }

    pub fn insert(&mut self, x: i32, z: i32, value: T) {
        if !self.map.contains_key(&x) {
            self.map.insert(x, HashMap::new());
        }
        self.map.get_mut(&x).unwrap().insert(z, value);        
    }
}