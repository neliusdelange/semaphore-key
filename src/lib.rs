#![crate_name = "semaphore_key"]

use std::{collections::HashMap, sync::Arc};
use once_cell::sync::Lazy;
use tokio::sync::{RwLock, Semaphore};

static SEMAPHORE_MAP: Lazy<RwLock<HashMap<String, Arc<Semaphore>>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

/// An empty SemaphoreKey struct to allow an implementation to be created. Use the implementation.
pub struct SemaphoreKey {}

impl SemaphoreKey {

    /// Returns or creates a semaphore wrapped in an Arc by the provided key
    ///
    /// # Arguments
    ///
    /// * `key` - The key to get an existing or create a new semaphore by
    /// 
    /// * `allowed_concurrent_threads` - Used when creating a new semaphore (if an existing one is not found by key), 
    /// to specify how many concurrent threads are allowed access.
    pub async fn get_semaphore_by_key(key: &String, allowed_concurrent_threads: usize) -> Arc<Semaphore> {
        let option_semaphore = SemaphoreKey::get_semaphore_if_exists_read_guard(key).await;

        let semaphore = if let Some(semaphore) = option_semaphore {
            semaphore
        } else {
            SemaphoreKey::create_new_semaphore_by_key(key, allowed_concurrent_threads).await
        };

        return semaphore;
    }

    /// Removes a semaphore from the internal map if it exists, and returns it wrapped in an Arc
    ///
    /// # Arguments
    ///
    /// * `key` - The key to get an existing semaphore by
    /// 
    pub async fn remove_semaphore_by_key(key: &String) -> Option<Arc<Semaphore>> {
        let mut write_guard = SEMAPHORE_MAP.write().await;
        let option_arc_semaphore = write_guard.remove(key);

        option_arc_semaphore
    }

    async fn get_semaphore_if_exists_read_guard(key: &String) -> Option<Arc<Semaphore>> {
        let mut result: Option<Arc<Semaphore>> = None;

        let read_guard = SEMAPHORE_MAP.read().await;
        let option_arc_semaphore = read_guard.get(key);

        if let Some(arc_semaphore) = option_arc_semaphore {
            let new_arc_semaphore = arc_semaphore.clone();
            result = Some(new_arc_semaphore);
        }

        return result;
    }

    async fn create_new_semaphore_by_key(key: &String, allowed_concurrent_threads: usize) -> Arc<Semaphore> {
        //do another check in write local before creating a new semaphore
        let mut write_guard = SEMAPHORE_MAP.write().await;
        let option_arc_semaphore = write_guard.get(key);

        if let Some(arc_semaphore) = option_arc_semaphore {
            return arc_semaphore.clone();
        }

        let new_arc_semaphore = Arc::new(Semaphore::new(allowed_concurrent_threads));

        write_guard.insert(key.clone(), new_arc_semaphore.clone()); //insert a reference into hashmap

        return new_arc_semaphore;
    }
}