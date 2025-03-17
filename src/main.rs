use crossterm::event::{self, Event, KeyCode};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::text::Text;
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io;
use std::thread;
use tokio::sync::mpsc;
use tokio::time::{Duration, sleep};
use tracing::{Level, info, span};
use tracing_appender::non_blocking;
use tracing_subscriber;
use tracing_subscriber::prelude::*; // Import the channel

#[tokio::main]
async fn main() -> io::Result<()> {
    // Redirect logs to a file instead of stdout
    let file_appender = tracing_appender::rolling::daily("./logs", "tracing.log");
    let (non_blocking, _guard) = non_blocking(file_appender);

    // Set up tracing subscriber with Tokio Console layer
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_writer(non_blocking))
        .with(console_subscriber::spawn())
        .init();

    
    // Create a channel to communicate with the Tokio runtime
    let (tx, mut rx) = mpsc::unbounded_channel::<String>();
    
    // Spawn a separate thread for the TUI
    thread::spawn(move || {
        enable_raw_mode().unwrap();
        let stdout = io::stdout();
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut tasks = Vec::new();

        // Clear
        print!("\x1B[2J\x1B[1;1H");
        loop {
            terminal
                .draw(|f| {
                    let chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([Constraint::Percentage(90), Constraint::Percentage(10)])
                        .split(f.area());

                    let tasks_text = {
                        let task_list = tasks.join("\n");
                        Text::from(task_list)
                    };

                    let task_widget = Paragraph::new(tasks_text)
                        .block(Block::default().title("Tasks").borders(Borders::ALL));
                    f.render_widget(task_widget, chunks[0]);

                    let instructions =
                        Paragraph::new("Press '1', '2', or '3' to start tasks. Press 'q' to quit.")
                            .block(Block::default().borders(Borders::ALL));
                    f.render_widget(instructions, chunks[1]);
                })
                .unwrap();

            if event::poll(Duration::from_millis(100)).unwrap() {
                if let Event::Key(key) = event::read().unwrap() {
                    match key.code {
                        KeyCode::Char('1') => {
                            tasks.push("Task 1 triggered".to_string());
                            tx.send("task1".to_string()).unwrap(); // Send message to spawn Task 1
                        }
                        KeyCode::Char('2') => {
                            tasks.push("Task 2 triggered".to_string());
                            tx.send("task2".to_string()).unwrap(); // Send message to spawn Task 2
                        }
                        KeyCode::Char('3') => {
                            tasks.push("Looping Task triggered".to_string());
                            tx.send("looping_task".to_string()).unwrap(); // Send message to spawn Looping Task
                        }
                        KeyCode::Char('q') => {
                            tx.send("BREAK".to_string()).unwrap();
                            break;
                        }, // Quit if 'q' is pressed
                        _ => {}
                    }
                }
            }
        }

        disable_raw_mode().unwrap();
    });

    loop {
        if let Some(task) = rx.recv().await {
            match task.as_str() {
                "task1" => {
                    tokio::spawn(do_work("Task 1", 10)); // Spawn Task 1
                }
                "task2" => {
                    tokio::spawn(do_work("Task 2", 12)); // Spawn Task 2
                }
                "looping_task" => {
                    tokio::spawn(looping_task()); // Spawn Looping Task
                }
                "BREAK" => {
                    break;
                }
                _ => {
                    // If it's an invalid task, just continue
                    continue;
                }
            }
        }
    }

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
