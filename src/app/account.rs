use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use hex;

#[derive(Debug, Serialize, Deserialize, Hash)]
pub struct Account {
    pub username: String,
    pub password: String,
}

impl Account {
    pub fn new(username: String, password: String) -> Self {
        {
            Self {
                username: username,
                password: password,
            }
        }
    }
    
    pub fn get_hash(&self) -> String {
        let mut hasher = Sha256::new();
        
        let serialized = serde_json::to_string(&self).unwrap();
        hasher.update(serialized);

        hex::encode(hasher.finalize())
    }
}
