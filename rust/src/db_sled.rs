use crate::db::{DBTx, DB};
use sled::{Db, Tree};
use std::sync::Arc;

pub struct SledDB {
    db: Arc<Db>,
}

impl SledDB {
    pub fn new(path: &str) -> Result<Self, String> {
        let db = sled::open(path).map_err(|e| e.to_string())?;
        Ok(SledDB { db: Arc::new(db) })
    }
}

pub struct SledDBTx {
    tree: Tree,
}

impl SledDBTx {
    pub fn new(tree: Tree) -> Self {
        SledDBTx { tree }
    }
}

impl DBTx for SledDBTx {
    fn obj_put(&self, key: &str, val: Vec<u8>) -> Result<(), String> {
        self.tree.insert(key, val).map_err(|e| e.to_string())?;
        Ok(())
    }

    fn obj_get(&self, key: &str) -> Result<Option<Vec<u8>>, String> {
        let result = self.tree.get(key).map_err(|e| e.to_string())?;
        Ok(result.map(|ivec| ivec.to_vec()))
    }

    fn obj_del(&self, key: &str) -> Result<Option<Vec<u8>>, String> {
        let result = self.tree.remove(key).map_err(|e| e.to_string())?;
        Ok(result.map(|ivec| ivec.to_vec()))
    }

    fn commit(&self) -> Result<(), String> {
        self.tree.flush().map_err(|e| e.to_string())?;
        Ok(())
    }

    fn cancel(&self) -> Result<(), String> {
        Ok(()) // No-op for sled
    }

    fn tail_push(&self, prefix: &str, val: Vec<u8>) -> Result<String, String> {
        let key = format!("{}:tail", prefix);
        self.tree
            .insert(key.clone(), val)
            .map_err(|e| e.to_string())?;
        Ok(key)
    }

    fn tail_pop(&self, prefix: &str) -> Result<Option<(String, Vec<u8>)>, String> {
        let key = format!("{}:tail", prefix);
        if let Some(value) = self.tree.remove(key.clone()).map_err(|e| e.to_string())? {
            Ok(Some((key, value.to_vec())))
        } else {
            Ok(None)
        }
    }

    fn head_push(&self, prefix: &str, val: Vec<u8>) -> Result<String, String> {
        let key = format!("{}:head", prefix);
        self.tree
            .insert(key.clone(), val)
            .map_err(|e| e.to_string())?;
        Ok(key)
    }

    fn head_pop(&self, prefix: &str) -> Result<Option<(String, Vec<u8>)>, String> {
        let key = format!("{}:head", prefix);
        if let Some(value) = self.tree.remove(key.clone()).map_err(|e| e.to_string())? {
            Ok(Some((key, value.to_vec())))
        } else {
            Ok(None)
        }
    }

    fn seq_get(&self, prefix: &str) -> Result<Box<dyn Iterator<Item = String> + Send>, String> {
        let iter = self.tree.scan_prefix(prefix).map(|res| match res {
            Ok((key, _)) => {
                println!("Scanned key: {}", String::from_utf8_lossy(&key));
                Ok(String::from_utf8_lossy(&key).to_string())
            }
            Err(e) => {
                eprintln!("Error scanning key: {}", e);
                Err(e)
            }
        });
        Ok(Box::new(iter.filter_map(Result::ok)))
    }
}

impl DB for SledDB {
    fn dbtx_create(&self) -> Result<Arc<dyn DBTx>, String> {
        let tree = self.db.open_tree("default").map_err(|e| e.to_string())?;
        let sled_db_tx = Arc::new(SledDBTx::new(tree));
        Ok(sled_db_tx)
    }

    fn flush(&self) -> Result<(), String> {
        self.db.flush().map_err(|e| e.to_string())?;
        Ok(())
    }
}
