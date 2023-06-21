use std::collections::HashMap;
use rand::distributions::{Alphanumeric, DistString};
use sha2::{Digest, Sha256};
use tokio::time::{Instant, Duration};

fn random_string() -> String {
    Alphanumeric.sample_string(&mut rand::thread_rng(), 32)
}

fn sha2_hash(data: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data.as_bytes());
    hex::encode::<[u8; 32]>(hasher.finalize().into())
}

#[derive(Debug, Clone)]
pub struct Database {
    tables: HashMap<String, HashMap<String, Instant>>,
}

impl Database {
    pub fn new() -> Self {
        Database { tables: HashMap::new() }
    }

    pub fn create_table(&mut self, table_name: &str) {
        self.tables.insert(table_name.to_owned(), HashMap::new());
    }

    pub fn check_table_exists(&self, table_name: &str) -> bool {
        if let Some(_) = self.tables.get(table_name) {
            return true;
        } false
    }

    pub fn drop_table(&mut self, table_name: &str) {
        self.tables.remove(table_name);
    }

    pub fn generate_value(&mut self, table_name: &str, expiration: u64) -> String {
        if let Some(table) = self.tables.get_mut(table_name) {
            let generated_hash = sha2_hash(&random_string());
            let expires = Instant::now()+Duration::from_secs(expiration);
            table.insert(generated_hash.to_string(), expires);
            return generated_hash;
        }
        String::new()
    }

    pub fn check_value_exists(&self, table_name: &str, key_val: &str) -> bool {
        if let Some(table) = self.tables.get(table_name) {
            if let Some(values_time) = table.get(key_val){
                return values_time > &Instant::now();
            } else {
                return false;
            }
        } false
    }

    pub fn remove_value(&mut self, table_name: &str, key_val: &str){
        if let Some(table) = self.tables.get_mut(table_name) {
            table.remove(key_val);
        };
    }

    pub fn filter_expired(&mut self) {
        let time_now = Instant::now();
        for table in self.tables.values_mut() {
            table.retain(|_, &mut value_time| value_time >= time_now);
        }
    }
}
