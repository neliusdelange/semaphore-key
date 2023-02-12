extern crate semaphore_key;

use log::{info};
use semaphore_key::SemaphoreKey;
use std::{thread, time::Duration};

#[tokio::main]
async fn main() {
    
    simple_logger::init_with_level(log::Level::Info).unwrap();

    //Spawn 3 tasks in parallel.
    //The method "do_work" allows 1 threads access at a time for a specific key.
    //Task one is using the key, "foo".
    //Task one is using the key, "bar".
    //Task one is using the key, "me".
    //All three tasks will execute in parallel, because they are using different keys

    let join_handle_one = tokio::spawn(async {
        do_work("foo").await;
    });

    let join_handle_two = tokio::spawn(async {
        do_work("bar").await;
    });

    let join_handle_three = tokio::spawn(async {
        do_work("me").await;
    });

    let (one, two, three) = tokio::join!(join_handle_one, join_handle_two, join_handle_three);

    one.unwrap();
    two.unwrap();
    three.unwrap();

    //optional remove created semaphore from internal static store,
    //if not needed anymore, otherwise keep in for the next method call.
    SemaphoreKey::remove_if_exists(&"foo".to_string()).await;
    SemaphoreKey::remove_if_exists(&"bar".to_string()).await;
    SemaphoreKey::remove_if_exists(&"me".to_string()).await;
}

//do_work allows 1 threads access at a time for a specific key
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