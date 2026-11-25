mod animation;
mod config;
mod emotion;
mod personality;
mod pet;
mod physics;
mod reminder;
mod storage;
mod tasks;
mod ui;

use crate::animation::{AnimationEngine, ParticleKind};
use crate::config::{AppConfig, AUTOSAVE_INTERVAL_SECS, TICK_RATE_MS};
use crate::physics::PhysicsBody;
use crate::reminder::ReminderEngine;
use crate::storage::AppState;
use crate::ui::{InputMode, UiState};
use anyhow::Result;
use clap::{Parser, Subcommand};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use futures::StreamExt;
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::time::{Duration, Instant};
use tokio::time::interval;

#[derive(Parser)]
#[command(name = "moodshell")]
#[command(about = "A living, emotionally reactive ASCII/Unicode terminal companion")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Add a new task with an optional deadline in minutes
    AddTask {
        /// Description of the task
        title: String,
        /// Optional deadline in minutes from now
        #[arg(short, long)]
        due_mins: Option<i64>,
    },
    /// Mark a pending task as completed
    Done {
        /// Optional task ID prefix or keyword match
        query: Option<String>,
    },
    /// Display current companion stats and vitals
    Stats,
    /// Pet your companion to boost affection and motivation
    Pet,
    /// Display current level and evolutionary tier
    Level,
    /// Toggle Focus Mode on or off
    FocusMode,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let cfg = AppConfig::load()?;
    let mut state = AppState::load_or_default(&cfg.data_file)?;

    // Handle CLI subcommands directly (persisting state to JSON)
    if let Some(cmd) = cli.command {
        match cmd {
            Commands::AddTask { title, due_mins } => {
                let task = state.tasks.add_task(title.clone(), due_mins);
                state.personality.record_activity(&title);
                state.pet.last_interaction = chrono::Local::now();
                state.pet.speech =
                    format!("Added \"{}\" [{}]! Let's crush it!", task.title, task.id);
                state.save(&cfg.data_file)?;
                println!(
                    "{}  Added task [{}]: \"{}\"",
                    state.pet.emotion.face(state.pet.level, 0, false),
                    task.id,
                    task.title
                );
            }
            Commands::Done { query } => {
                if let Some(completed) = state.tasks.complete_task(query.as_deref()) {
                    state.personality.record_activity(&completed.title);
                    state.pet.on_task_completed(&completed.title);
                    state.save(&cfg.data_file)?;
                    println!(
                        "{}  {}",
                        state.pet.emotion.face(state.pet.level, 0, false),
                        state.pet.speech
                    );
                } else {
                    println!("(・_・;) No matching pending task found.");
                }
            }
            Commands::Stats => {
                let s = &state.pet.stats;
                println!(
                    "{}  {} — Mood: {} | Streak: {}",
                    state.pet.emotion.face(state.pet.level, 0, false),
                    state.pet.name,
                    state.pet.emotion.label(),
                    state.pet.streak
                );
                println!(
                    "Health: {:.0}% | Motivation: {:.0}% | Energy: {:.0}% | Focus: {:.0}% | Trust: {:.0}%",
                    s.health, s.motivation, s.energy, s.focus, s.trust
                );
                println!("Total Focus Time: {} mins", state.pet.total_focus_minutes);
            }
            Commands::Pet => {
                state.pet.on_petted();
                state.save(&cfg.data_file)?;
                println!(
                    "{}  {}",
                    state.pet.emotion.face(state.pet.level, 0, false),
                    state.pet.speech
                );
            }
            Commands::Level => {
                println!(
                    "{}  Level {} ({}) — XP: {}/{}",
                    state.pet.emotion.face(state.pet.level, 0, false),
                    state.pet.level,
                    state.pet.evolution_title(),
                    state.pet.xp,
                    state.pet.xp_for_next_level()
                );
            }
            Commands::FocusMode => {
                state.pet.toggle_focus_mode();
                state.save(&cfg.data_file)?;
                println!(
                    "{}  {}",
                    state.pet.emotion.face(state.pet.level, 0, false),
                    state.pet.speech
                );
            }
        }
        return Ok(());
    }

    // Launch Fullscreen Living Terminal Companion TUI
    run_tui(cfg, state).await
}

async fn run_tui(cfg: AppConfig, mut state: AppState) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut physics = PhysicsBody::default();
    let mut anim = AnimationEngine::default();
    let mut ui_state = UiState::default();

    let mut ticker = interval(Duration::from_millis(TICK_RATE_MS));
    let mut autosave_ticker = interval(Duration::from_secs(AUTOSAVE_INTERVAL_SECS));
    let mut reader = event::EventStream::new();
    let mut last_tick = Instant::now();
    let mut last_disk_mod = std::fs::metadata(&cfg.data_file)
        .and_then(|m| m.modified())
        .ok();

    let res = loop {
        tokio::select! {
            _ = ticker.tick() => {
                let now = Instant::now();
                let dt = (now - last_tick).as_secs_f64().min(0.1);
                last_tick = now;

                let size = terminal.size()?;
                let habitat_w = (size.width as f64 * 0.60).max(20.0);
                let habitat_h = (size.height.saturating_sub(8) as f64).max(8.0);

                ReminderEngine::evaluate(&mut state.pet, &mut state.tasks, &state.personality);

                let sprite_w = state.pet.emotion.face(state.pet.level, physics.impact_frames, anim.blink).chars().count() as f64;
                physics.update(dt, habitat_w, habitat_h, sprite_w, state.pet.focus_mode);
                anim.update(dt, state.pet.emotion, state.pet.level, physics.impact_frames, physics.vx, physics.vy, physics.x, physics.y, habitat_w);

                terminal.draw(|f| ui::render(f, &state, &physics, &anim, &ui_state))?;
            }

            _ = autosave_ticker.tick() => {
                // Hot-reload if user ran `moodshell add-task` in another terminal tab
                if let Ok(meta) = std::fs::metadata(&cfg.data_file) {
                    if let Ok(modified) = meta.modified() {
                        if Some(modified) != last_disk_mod {
                            if let Ok(reloaded) = AppState::load_or_default(&cfg.data_file) {
                                state = reloaded;
                                last_disk_mod = Some(modified);
                                continue;
                            }
                        }
                    }
                }
                let _ = state.save(&cfg.data_file);
                last_disk_mod = std::fs::metadata(&cfg.data_file).and_then(|m| m.modified()).ok();
            }

            Some(Ok(event)) = reader.next() => {
                if let Event::Key(key) = event {
                    if key.kind != KeyEventKind::Press {
                        continue;
                    }
                    match ui_state.input_mode {
                        InputMode::Normal => match key.code {
                            KeyCode::Char('q') => break Ok(()),
                            KeyCode::Char('p') => {
                                state.pet.on_petted();
                                physics.jump(18.0, 0.0);
                                anim.spawn_burst(physics.x + 2.0, physics.y, ParticleKind::Heart, 8);
                                let _ = state.save(&cfg.data_file);
                            }
                            KeyCode::Char('j') => {
                                physics.jump(26.0, 8.0);
                                anim.spawn_burst(physics.x + 2.0, physics.y, ParticleKind::Sparkle, 5);
                            }
                            KeyCode::Char('d') => {
                                if let Some(done) = state.tasks.complete_task(None) {
                                    state.personality.record_activity(&done.title);
                                    let leveled = state.pet.on_task_completed(&done.title);
                                    physics.jump(28.0, 4.0);
                                    anim.spawn_burst(
                                        physics.x + 2.0,
                                        physics.y,
                                        ParticleKind::Star,
                                        if leveled { 25 } else { 12 },
                                    );
                                    let _ = state.save(&cfg.data_file);
                                } else {
                                    state.pet.speech = "No pending tasks left! Add one with [a].".to_string();
                                }
                            }
                            KeyCode::Char('f') => {
                                if let Some(_mins) = state.pet.toggle_focus_mode() {
                                    physics.jump(25.0, 0.0);
                                    anim.spawn_burst(physics.x + 2.0, physics.y, ParticleKind::Star, 16);
                                }
                                let _ = state.save(&cfg.data_file);
                            }
                            KeyCode::Char('a') => {
                                ui_state.input_mode = InputMode::AddingTask;
                                ui_state.input_buffer.clear();
                            }
                            _ => {}
                        },
                        InputMode::AddingTask => match key.code {
                            KeyCode::Esc => {
                                ui_state.input_mode = InputMode::Normal;
                                ui_state.input_buffer.clear();
                            }
                            KeyCode::Enter => {
                                let raw = ui_state.input_buffer.trim().to_string();
                                if !raw.is_empty() {
                                    let (title, due_mins) = parse_inline_deadline(&raw);
                                    let task = state.tasks.add_task(title.clone(), due_mins);
                                    state.personality.record_activity(&title);
                                    state.pet.last_interaction = chrono::Local::now();
                                    state.pet.speech = format!("Got it! Added \"{}\".", task.title);
                                    anim.spawn_burst(physics.x + 2.0, physics.y, ParticleKind::Sparkle, 6);
                                    let _ = state.save(&cfg.data_file);
                                }
                                ui_state.input_mode = InputMode::Normal;
                                ui_state.input_buffer.clear();
                            }
                            KeyCode::Backspace => {
                                ui_state.input_buffer.pop();
                            }
                            KeyCode::Char(c) => {
                                ui_state.input_buffer.push(c);
                            }
                            _ => {}
                        },
                    }
                }
            }
        }
    };

    state.save(&cfg.data_file)?;
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    res
}

/// Parses optional trailing `@30m` or `@2h` syntax when adding tasks inside the TUI.
fn parse_inline_deadline(input: &str) -> (String, Option<i64>) {
    if let Some((title_part, tag)) = input.rsplit_once('@') {
        let tag = tag.trim().to_lowercase();
        if let Some(mins_str) = tag.strip_suffix('m') {
            if let Ok(m) = mins_str.parse::<i64>() {
                return (title_part.trim().to_string(), Some(m));
            }
        } else if let Some(hrs_str) = tag.strip_suffix('h') {
            if let Ok(h) = hrs_str.parse::<i64>() {
                return (title_part.trim().to_string(), Some(h * 60));
            }
        }
    }
    (input.to_string(), None)
}
