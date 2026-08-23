# TNHL

TNHL is a terminal-based NHL data dashboard built with Rust. It provides an interactive TUI for browsing games, standings, player/team stats, and playoff information by pulling data from the NHL API. (See https://github.com/Zmalski/NHL-API-Reference).

![Demo GIF](assets/demo.gif)

## Features

- View daily NHL games (scoring, stats, boxscore)
- Real-time tracking for live games
- Browse league standings
- Explore team scoring and goalie stats
- Display playoff brackets and series details
- Change dates and/or teams directly in the UI

## Installation

Using cargo:

```bash
cargo run
```

## Usage

- `1` – Games
- `2` – Standings
- `3` – Team Stats
- `4` – Playoffs
- `?` – Open help screen
- `Ctrl+c/q` – Quit

## Configuration

TNHL reads an optional `tnhl.toml` config file, auto-generated with defaults on
first run. Its location depends on your OS:

- Linux: `~/.config/tnhl/tnhl.toml`
- macOS: `~/Library/Application Support/tnhl/tnhl.toml`
- Windows: `%APPDATA%\tnhl\tnhl.toml`

Available keys:

| Key             | Type   | Default            | Description                                                                                          |
| --------------- | ------ | ------------------ | ---------------------------------------------------------------------------------------------------- |
| `timezone`      | string | `America/Montreal` | Timezone for displayed game start times. Any [IANA tz name](https://en.wikipedia.org/wiki/List_of_tz_database_time_zones) (e.g. `US/Eastern`). |
| `favorite_team` | string | none               | Your team's 3-letter code (e.g. `MTL`, case-insensitive). Becomes the default team on the Team Stats page and is highlighted in gold in the standings and today's matchups. |
| `log_level`     | string | `error`            | Logging verbosity written to `app.log`: `off`, `trace`, `debug`, `info`, `warn`, or `error`.         |

Example:

```toml
timezone = "US/Eastern"
favorite_team = "TOR"
log_level = "info"
```
