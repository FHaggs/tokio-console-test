// tokio_tracing_example.rs
// A simple async program using Tokio and Tracing to generate events viewable in Tokio Console.

use tokio::{task, time::{sleep, Duration}};
use tracing::{info, span, Level};
use tracing_subscriber;
use tracing_subscriber::prelude::*;

#[tokio::main]
async fn main() {
    // Set up tracing subscriber with Tokio Console layer
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(console_subscriber::spawn())
        .init();

    let main_span = span!(Level::INFO, "main");
    let _enter = main_span.enter();

    info!("Starting async tasks");

    let task1 = task::spawn(do_work("Task 1", 10));
    let task2 = task::spawn(do_work("Task 2", 12));
    let task3 = task::spawn(looping_task());

    let _ = tokio::join!(task1, task2, task3);

    info!("All tasks completed");
}

async fn do_work(name: &str, duration: u64) {
    let span = span!(Level::INFO, "work", task_name = name);
    let _enter = span.enter();

    info!(task_name = name, "Starting work");
    sleep(Duration::from_secs(duration)).await;
    info!(task_name = name, "Finished work");
}

async fn looping_task() {
    let span = span!(Level::INFO, "looping_task");
    let _enter = span.enter();

    loop {
        info!("Looping task is running...");
        sleep(Duration::from_secs(5)).await;
    }
}

