use crate::group::Group;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// NOTE(elsuizo:2021-11-14): recordar que es una tuple-struct
pub struct GroupTable(Mutex<HashMap<Arc<String>, Arc<Group>>>);

impl GroupTable {
    pub fn new() -> Self {
        Self(Mutex::new(HashMap::new()))
    }

    pub fn get(&self, name: &String) -> Option<Arc<Group>> {
        self.0.lock().unwrap().get().cloned()
    }

    pub fn get_or_create(&self, name: Arc<String>) -> Arc<Group> {
        self.0
            .lock()
            .unwrap()
            .entry()
            .or_insert_with(|| Arc::new(Group::new(name)))
            .clone()
    }
}
