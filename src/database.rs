use std::collections::HashMap;
use rand::distributions::{Alphanumeric, DistString};
use sha2::{Digest, Sha256};

fn random_string() -> String {
    Alphanumeric.sample_string(&mut rand::thread_rng(), 32)
}

fn sha2_hash(data: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data.as_bytes());
    hex::encode::<[u8; 32]>(hasher.finalize().into())
}

#[derive(Debug)]
pub struct Database {
    tables: HashMap<String, HashMap<String, u64>>,
}

impl Database {
    pub fn new() -> Self {
        Database { tables: HashMap::new() }
    }

    pub fn create_table(&mut self, table_name: &str) {
        self.tables.insert(table_name.to_owned(), HashMap::new());
    }

    pub fn insert_data(&mut self, table_name: &str, _expiration: u64) -> String {
        if let Some(table) = self.tables.get_mut(table_name) {
            let generated_hash = sha2_hash(&random_string());
            table.insert(generated_hash.to_string(), 0);
            return generated_hash;
        }
        String::new()
    }

    pub fn check_value_exists(&self, table_name: &str, key_val: &str) -> bool {
        if let Some(table) = self.tables.get(table_name) {
            return table.contains_key(key_val);
        } false
    }
}
