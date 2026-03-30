mod app;
mod config;
mod input;
mod models;
mod sources;
mod state;
mod ui;

use crate::{
    app::App,
    sources::{
        AppEvent, Source,
        games::{GamesCommand, GamesSource},
        standings::{StandingsCommand, StandingsSource},
    },
};

use simplelog::*;
use std::fs::File;

use ratatui::{
    Terminal,
    backend::{Backend, CrosstermBackend},
    crossterm::{
        event::{DisableMouseCapture, EnableMouseCapture, Event, EventStream},
        execute,
        terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
    },
};
use std::{error::Error, io};
use tokio::sync::mpsc::Receiver;
use tokio::time::Duration;
use tokio_util::sync::CancellationToken;

use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stderr = io::stderr();
    execute!(stderr, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stderr);
    let mut terminal = Terminal::new(backend)?;

    let log_file = File::create("app.log")?;
    WriteLogger::init(LevelFilter::Debug, Config::default(), log_file)?;

    // create app and run it
    let (games_cmd_tx, games_cmd_rx) = tokio::sync::mpsc::channel(8);
    let (standings_cmd_tx, standings_cmd_rx) = tokio::sync::mpsc::channel(8);

    let mut app = App::new(games_cmd_tx.clone(), standings_cmd_tx.clone());
    let cancel = CancellationToken::new();
    let _ = run_app(
        &mut terminal,
        &mut app,
        cancel.clone(),
        games_cmd_rx,
        standings_cmd_rx,
    )
    .await;

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

async fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    cancel: CancellationToken,
    games_rx: Receiver<GamesCommand>,
    standings_rx: Receiver<StandingsCommand>,
) -> io::Result<()>
where
    io::Error: From<B::Error>,
{
    let (tx, mut rx) = tokio::sync::mpsc::channel::<AppEvent>(32);

    // Spawn standings source
    let standings_source = Box::new(StandingsSource::new(standings_rx));
    let standings_tx = tx.clone();
    let standings_cancel = cancel.clone();
    tokio::spawn(async move {
        standings_source.run(standings_tx, standings_cancel).await;
    });

    // Spawn games source
    let games_source = GamesSource::new(games_rx);
    let games_tx = tx.clone();
    let games_cancel = cancel.clone();
    tokio::spawn(async move {
        Box::new(games_source).run(games_tx, games_cancel).await;
    });

    // Spawn terminal event reader
    let input_tx = tx.clone();
    let input_cancel = cancel.clone();
    tokio::spawn(async move {
        let mut reader = EventStream::new();
        loop {
            tokio::select! {
                _ = input_cancel.cancelled() => break,
                event = reader.next() => {
                    match event {
                        Some(Ok(Event::Key(key))) => {
                            let _ = input_tx.send(AppEvent::Input(key)).await;
                        }
                        Some(Err(e)) => {
                            log::error!("Terminal event error: {}", e);
                            break;
                        }
                        None => break,
                        _ => {} // Ignore mouse/resize for now
                    }
                }
            }
        }
    });

    // Spawn tick timer
    let tick_tx = tx;
    let tick_cancel = cancel.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_millis(200));
        loop {
            tokio::select! {
                _ = tick_cancel.cancelled() => break,
                _ = interval.tick() => {
                    let _ = tick_tx.send(AppEvent::Tick).await;
                }
            }
        }
    });

    // Main event loop
    loop {
        terminal.draw(|f| ui::render(f, app))?;

        if let Some(event) = rx.recv().await {
            app.state.handle_event(event);
            if app.state.should_quit {
                break;
            }
        } else {
            break;
        }
    }

    Ok(())
}
