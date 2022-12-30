# semaphore-key
Control concurrent thread access by key using a shared semaphore.

Internally this library holds a static hashmap of Semaphores values by key of type string which is conveniently managed throuh the public API, removing the overhead of maintaining such a map and synchronization in your own projects. See the examples for implementation details.

## Usage

### In your project

Add the semaphore-key dependency as indicated below and have a look at the example code.
More runnable examples can be found in the 'examples' directory in github.

```toml
[dependencies]
semaphore-key = "0.2.0"
```

```rust
use semaphore_key::SemaphoreKey;

//do_work only allows 1 thread access at a time for a specific key
async fn do_work(key: &str) {

    let allowed_concurrent_threads = 1;

    info!("Thread:{:?} entering method", thread::current().id());

    let semaphore = SemaphoreKey::get_semaphore_by_key(&key.to_string(), allowed_concurrent_threads).await;

    let _permit = semaphore.acquire().await.unwrap();

    info!(
        "Thread:{:?} going to rest for 5 seconds",
        thread::current().id()
    );

    //rest for 5 seconds
    thread::sleep(Duration::from_millis(5000));

    info!("Thread:{:?} done with resting", thread::current().id());
}
```