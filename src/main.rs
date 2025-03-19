use crossterm::event::{self, Event, KeyCode};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::text::Text;
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::thread;
use tokio::sync::mpsc;
use tokio::time::{Duration, sleep};
use tracing::{info, span, Level};
use tracing_appender::non_blocking;
use tracing_subscriber::prelude::*;

// Configuration structure for TUI
struct TuiConfig {
    poll_interval: Duration,
}

// Enum to represent tasks
#[derive(Debug, Clone)]
enum Task {
    Task1,
    Task2,
    LoopingTask,
    Break,
}

// Function to initialize logging
fn setup_logging() {
    let file_appender = tracing_appender::rolling::daily("./logs", "tracing.log");
    let (non_blocking, _guard) = non_blocking(file_appender);

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_writer(non_blocking))
        .with(console_subscriber::spawn())
        .init();
}

// Function to handle TUI rendering and input
fn run_tui(tx: mpsc::UnboundedSender<Task>) -> io::Result<()> {
    let config = TuiConfig {
        poll_interval: Duration::from_millis(100),
    };

    enable_raw_mode()?;
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let mut tasks = Vec::new();

    print!("\x1B[2J\x1B[1;1H");

    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(90), Constraint::Percentage(10)])
                .split(f.area());

            let tasks_text = Text::from(tasks.join("\n"));
            let task_widget = Paragraph::new(tasks_text)
                .block(Block::default().title("Tasks").borders(Borders::ALL));
            f.render_widget(task_widget, chunks[0]);

            let instructions = Paragraph::new("Press '1', '2', or '3' to start tasks. Press 'q' to quit.")
                .block(Block::default().borders(Borders::ALL));
            f.render_widget(instructions, chunks[1]);
        })?;

        if event::poll(config.poll_interval)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('1') => {
                        tasks.push("Task 1 triggered".to_string());
                        tx.send(Task::Task1).unwrap();
                    }
                    KeyCode::Char('2') => {
                        tasks.push("Task 2 triggered".to_string());
                        tx.send(Task::Task2).unwrap();
                    }
                    KeyCode::Char('3') => {
                        tasks.push("Looping Task triggered".to_string());
                        tx.send(Task::LoopingTask).unwrap();
                    }
                    KeyCode::Char('q') => {
                        tx.send(Task::Break).unwrap();
                        break;
                    }
                    _ => {}
                }
            }
        }
    }

    disable_raw_mode()?;
    Ok(())
}

// Function to handle task spawning from channel messages
async fn handle_tasks(mut rx: mpsc::UnboundedReceiver<Task>) {
    while let Some(task) = rx.recv().await {
        match task {
            Task::Task1 => {
                tokio::spawn(do_work("Task 1", 10));
            }
            Task::Task2 => {
                tokio::spawn(do_work("Task 2", 12));
            }
            Task::LoopingTask => {
                tokio::spawn(looping_task());
            }
            Task::Break => {
                break;
            }
        }
    }
}

// Function for doing work
async fn do_work(name: &str, duration: u64) {
    let span = span!(Level::INFO, "work", task_name = name);
    let _enter = span.enter();

    info!(task_name = name, "Starting work");
    sleep(Duration::from_secs(duration)).await;
    info!(task_name = name, "Finished work");
}

// Function for looping task
async fn looping_task() {
    let span = span!(Level::INFO, "looping_task");
    let _enter = span.enter();

    loop {
        info!("Looping task is running...");
        sleep(Duration::from_secs(5)).await;
    }
}

#[tokio::main]
async fn main() -> io::Result<()> {
    setup_logging();

    let (tx, rx) = mpsc::unbounded_channel::<Task>();

    let tui_handle = thread::spawn(move || run_tui(tx));
    handle_tasks(rx).await;

    tui_handle.join().unwrap()?;

    Ok(())
}