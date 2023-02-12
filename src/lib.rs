#![crate_name = "semaphore_key"]

use log::{trace};
use once_cell::sync::Lazy;
use std::thread;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{RwLock, Semaphore};
use tokio::task;

static SEMAPHORE_MAP: Lazy<RwLock<HashMap<String, Arc<Semaphore>>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

/// The SemaphoreKey struct holds an implementation to be used to for the Semaphore by key functionality.
pub struct SemaphoreKey {}

impl SemaphoreKey {
    /// Gets or creates a semaphore wrapped in an Arc by the provided key
    ///
    /// # Arguments
    ///
    /// * `key` - The key to get an existing or create a new semaphore by
    ///
    /// * `allowed_concurrent_threads` - Used when creating a new semaphore (if an existing one is not found by key),
    /// to specify how many concurrent threads are allowed access.
    pub async fn get_or_create_semaphore(key: &String, allowed_concurrent_threads: usize) -> Arc<Semaphore> {

        trace!("Thread:{:?} request semaphore for key={} allowed threads={}", thread::current().id(), key, allowed_concurrent_threads);

        let option_semaphore = SemaphoreKey::get_semaphore_if_exists_read_guard(key).await;

        let semaphore = if let Some(semaphore) = option_semaphore {
            trace!("Thread:{:?} semaphore exists for key={}", thread::current().id(), key);
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
    pub async fn remove_if_exists(key: &String) -> Option<Arc<Semaphore>> {

        trace!("Thread:{:?} remove semaphore for key={}", thread::current().id(), key);

        let option_arc_semaphore: Option<Arc<Semaphore>>;

        //create new scope for the write_guard
        {
            let mut write_guard = SEMAPHORE_MAP.write().await;
            option_arc_semaphore = (write_guard).remove(key);
        }
        //write_guard goes out of scope here

        //yield control back to the tokio runtime to allow other threads/tasks,
        //waiting for the write lock to continue
        task::yield_now().await;

        option_arc_semaphore
    }

    async fn get_semaphore_if_exists_read_guard(key: &String) -> Option<Arc<Semaphore>> {
        let mut result: Option<Arc<Semaphore>> = None;

        let read_guard = SEMAPHORE_MAP.read().await;
        let option_arc_semaphore = (read_guard).get(key);

        if let Some(arc_semaphore) = option_arc_semaphore {
            let new_arc_semaphore = arc_semaphore.clone();
            result = Some(new_arc_semaphore);
        }

        return result;
    }

    async fn create_new_semaphore_by_key(key: &String, allowed_concurrent_threads: usize) -> Arc<Semaphore> {

        trace!("Thread:{:?} create new semaphore for key={} allowed threads={}", thread::current().id(), key, allowed_concurrent_threads);

        let semaphore: Arc<Semaphore>;
        
        //use new scope for write_guard
        {
            let mut write_guard = SEMAPHORE_MAP.write().await;

            //perform another check in write local before creating a new semaphore
            let option_arc_semaphore = (write_guard).get(key);

            if let Some(semaphore) = option_arc_semaphore {

                //yield control back to the tokio runtime to allow other threads/tasks to continue
                task::yield_now().await;

                return semaphore.clone();
            }

            trace!("Thread:{:?} create a new semaphore for key={} with allowed threads={}", thread::current().id(), key, allowed_concurrent_threads);

            semaphore = Arc::new(Semaphore::new(allowed_concurrent_threads));

            write_guard.insert(key.clone(), semaphore.clone()); //insert a reference into hashmap
        }

        //The write guard goes out of scope here.
        //Now that the new key and semaphore has been added,
        //yield control back to the tokio runtime to allow other waiting threads/tasks,
        //waiting on the write guard to continue.

        task::yield_now().await;

        return semaphore;
    }
}