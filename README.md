# semaphore-key
Control concurrent thread access by key using a shared semaphore.

Internally this library holds a static hashmap of Semaphores values by key of type string which is conveniently managed throuh the public API, removing the overhead of maintaining such a map and synchronization in your own projects. See the examples for implementation details.

## Usage

### In your project

Add the semaphore-key dependency as indicated below and have a look at the example code.
More runnable examples can be found in the 'examples' directory in github.

```toml
[dependencies]
semaphore-key = "1.0.0"
```

```rust
use log::{info};
use semaphore_key::SemaphoreKey;
use std::{thread, time::Duration};

#[tokio::main]
async fn main() {
    
    simple_logger::init_with_level(log::Level::Info).unwrap();

    //Spawn 3 tasks in parallel.
    //The method "do_work" only allows 1 thread access at a time for a specific key.
    //Tasks one and two are using the same key, "foo", and will execute one after another,
    //while task three is using key a different key, "bar", and will execute simultaneously with task one.

    let join_handle_one = tokio::spawn(async {
        do_work("foo").await;
    });

    let join_handle_two = tokio::spawn(async {
        do_work("foo").await;
    });

    let join_handle_three = tokio::spawn(async {
        do_work("bar").await;
    });

    tokio::join!(join_handle_one, join_handle_two, join_handle_three);

    //remove created semaphore from internal static store
    SemaphoreKey::remove_if_exists(&"foo".to_string()).await;
    SemaphoreKey::remove_if_exists(&"bar".to_string()).await;
}

//do_work only allows 1 thread access at a time for a specific key
async fn do_work(key: &str) {

    let allowed_concurrent_threads = 1;

    info!("Thread:{:?} entering method", thread::current().id());

    let semaphore = SemaphoreKey::get_or_create_semaphore(&key.to_string(), allowed_concurrent_threads).await;

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