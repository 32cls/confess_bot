# AGENTS.md — Confess Discord Bot

## Quick commands
- Build: `cargo build`
- Run: `cargo run`
- Docker: `docker compose up`
- Docker standalone: `docker build -t confess . && docker run confess`

## Prerequisites
- `DISCORD_TOKEN` — bot token (loaded via dotenv)
- `GUILD_ID` — Discord guild ID (panics if missing; unwrapped in `main.rs:32`)
- `RUST_LOG` — set to `info` (or `debug`) for log output; defaults to `warn` (no logs visible)
- Redis — connects to `redis://localhost:6379` by default; override with `REDIS_URL`

## Source layout
- `src/main.rs` — bootstrap, Poise framework setup, event handler, Redis connection
- `src/commands.rs` — `/set-channel`, `/confess`, scheduler (`start_confession_scheduler`)
- `src/persistence.rs` — `fetch_channel_id` helper (reads from Redis)
- `.env` — local env vars; `.dockerignore` used by Docker build

## Architecture
- Framework: Poise (serenity-next branch) with **both** slash commands and prefix (mention) commands
- Commands auto-register on bot `Ready` event via `GuildId::set_commands` (main.rs:38)
- Confessions stored in **Redis** (`confessions:queue` list, `confessions:channel` key)
- Scheduler runs every 1 minute (defined in `commands.rs:13`); uses `tokio::time::interval` + `futures::stream::unfold` to pick a random confession with probability proportional to queue size
- `Data` struct holds a `redis::aio::ConnectionManager`

## Gotchas
- `GUILD_ID` is unwrapped in the `Ready` handler — bot panics on startup if missing.
- `REDIS_URL` falls back to `redis://localhost:6379` — no validation that Redis is reachable at startup; connection error happens at `ConnectionManager` init (main.rs:80).
- `RUST_LOG` is not set in `.env` by default — no log output in Docker unless explicitly configured.
- No tests exist.
- Rust edition 2024.
