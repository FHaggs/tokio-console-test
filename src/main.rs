// tokio_tracing_example.rs
// A simple async program using Tokio, Tracing, and Ratatui for TUI.

use std::io;
use std::sync::{Arc, Mutex};
use std::thread;
use tokio::{task, time::{sleep, Duration}};
use tracing::{info, span, Level};
use tracing_subscriber;
use tracing_subscriber::prelude::*;
use ratatui::{backend::CrosstermBackend, Terminal};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::layout::{Layout, Constraint, Direction};
use ratatui::text::Text;
use crossterm::event::{self, Event, KeyCode};
use crossterm::terminal::{enable_raw_mode, disable_raw_mode};
use tracing_appender::non_blocking;


#[tokio::main]
async fn main() -> io::Result<()> {
    // Set up tracing subscriber with Tokio Console layer
    let file_appender = tracing_appender::rolling::daily("./logs", "tracing.log");
    let (non_blocking, _guard) = non_blocking(file_appender);
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_writer(non_blocking))
        .with(console_subscriber::spawn())
        .init();

    let tasks = Arc::new(Mutex::new(Vec::new()));

    // Spawn a separate thread for the TUI
    let tasks_clone = Arc::clone(&tasks);
    thread::spawn(move || {
        enable_raw_mode().unwrap();
        let stdout = io::stdout();
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend).unwrap();

        loop {
            terminal.draw(|f| {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Percentage(90),
                        Constraint::Percentage(10),
                    ]).split(f.area());

                let tasks_text = {
                    let tasks = tasks_clone.lock().unwrap();
                    let task_list = tasks.join("\n");
                    Text::from(task_list)
                };

                let task_widget = Paragraph::new(tasks_text).block(Block::default().title("Tasks").borders(Borders::ALL));
                f.render_widget(task_widget, chunks[0]);

                let instructions = Paragraph::new("Press '1', '2', or '3' to start tasks. Press 'q' to quit.")
                    .block(Block::default().borders(Borders::ALL));
                f.render_widget(instructions, chunks[1]);
            }).unwrap();

            if event::poll(Duration::from_millis(100)).unwrap() {
                if let Event::Key(key) = event::read().unwrap() {
                    match key.code {
                        KeyCode::Char('1') => {
                            tasks_clone.lock().unwrap().push("Task 1 triggered".to_string());
                        }
                        KeyCode::Char('2') => {
                            tasks_clone.lock().unwrap().push("Task 2 triggered".to_string());
                        }
                        KeyCode::Char('3') => {
                            tasks_clone.lock().unwrap().push("Looping Task triggered".to_string());
                        }
                        KeyCode::Char('q') => break,
                        _ => {}
                    }
                }
            }
        }

        disable_raw_mode().unwrap();
    });

    // Run tasks in the background
    let task1 = task::spawn(do_work("Task 1", 10));
    let task2 = task::spawn(do_work("Task 2", 12));
    let task3 = task::spawn(looping_task());

    let _ = tokio::join!(task1, task2, task3);

    Ok(())
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

