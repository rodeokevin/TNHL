mod app;
mod banner;
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
        boxscore::{BoxscoreCommand, BoxscoreSource},
        game_story::{GameStoryCommand, GameStorySource},
        games::{GamesCommand, GamesSource},
        playoff_bracket::{PlayoffBracketCommand, PlayoffBracketSource},
        standings::{StandingsCommand, StandingsSource},
        teams_stats::{TeamStatsCommand, TeamStatsSource},
    },
    state::team_stats::team_picker::TeamAbbrev,
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
    let (boxscore_cmd_tx, boxscore_cmd_rx) = tokio::sync::mpsc::channel(8);
    let (game_story_tx, game_story_rx) = tokio::sync::mpsc::channel(8);
    let (team_stats_tx, team_stats_rx) = tokio::sync::mpsc::channel(8);
    let (playoff_bracket_tx, playoff_bracket_rx) = tokio::sync::mpsc::channel(8);

    // Date is configured in here
    let mut app = App::new(
        games_cmd_tx.clone(),
        standings_cmd_tx.clone(),
        boxscore_cmd_tx.clone(),
        game_story_tx.clone(),
        team_stats_tx.clone(),
        playoff_bracket_tx.clone(),
    );
    let cancel = CancellationToken::new();
    let _ = run_app(
        &mut terminal,
        &mut app,
        cancel.clone(),
        games_cmd_rx,
        standings_cmd_rx,
        boxscore_cmd_rx,
        game_story_rx,
        team_stats_rx,
        playoff_bracket_rx,
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
    boxscore_rx: Receiver<BoxscoreCommand>,
    game_story_rx: Receiver<GameStoryCommand>,
    team_stats_rx: Receiver<TeamStatsCommand>,
    playoff_bracket_rx: Receiver<PlayoffBracketCommand>,
) -> io::Result<()>
where
    io::Error: From<B::Error>,
{
    let (tx, mut rx) = tokio::sync::mpsc::channel::<AppEvent>(32);

    // Spawn standings source
    let standings_source = Box::new(StandingsSource::new(
        standings_rx,
        app.state.date_state.date.to_string(),
    ));
    let standings_tx = tx.clone();
    let standings_cancel = cancel.clone();
    tokio::spawn(async move {
        standings_source.run(standings_tx, standings_cancel).await;
    });

    // Spawn games source
    let games_source = GamesSource::new(games_rx, app.state.date_state.date.to_string());
    let games_tx = tx.clone();
    let games_cancel = cancel.clone();
    tokio::spawn(async move {
        Box::new(games_source).run(games_tx, games_cancel).await;
    });

    // Spawn boxscore source
    let boxscore_source = BoxscoreSource::new(boxscore_rx);
    let boxscore_tx = tx.clone();
    let boxscore_cancel = cancel.clone();
    tokio::spawn(async move {
        Box::new(boxscore_source)
            .run(boxscore_tx, boxscore_cancel)
            .await;
    });

    // Spawn game story source
    let game_story_source = GameStorySource::new(game_story_rx);
    let game_story_tx = tx.clone();
    let game_story_cancel = cancel.clone();
    tokio::spawn(async move {
        Box::new(game_story_source)
            .run(game_story_tx, game_story_cancel)
            .await;
    });

    // Spawn team stats source
    let team_stats_source = TeamStatsSource::new(team_stats_rx, TeamAbbrev::default(), app.state.date_state.year);
    let team_stats_tx = tx.clone();
    let team_stats_cancel = cancel.clone();
    tokio::spawn(async move {
        Box::new(team_stats_source)
            .run(team_stats_tx, team_stats_cancel)
            .await;
    });

    // Spawn playoff bracket source
    let playoff_bracket_source =
        PlayoffBracketSource::new(playoff_bracket_rx, app.state.date_state.year);
    let playoff_bracket_tx = tx.clone();
    let playoff_bracket_cancel = cancel.clone();
    tokio::spawn(async move {
        Box::new(playoff_bracket_source)
            .run(playoff_bracket_tx, playoff_bracket_cancel)
            .await;
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
                        _ => {}
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
        terminal.draw(|f| ui::render::render(f, app))?;

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
