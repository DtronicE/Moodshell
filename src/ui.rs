use crate::animation::{AnimationEngine, ParticleKind};
use crate::emotion::Emotion;
use crate::physics::PhysicsBody;
use crate::storage::AppState;
use crate::tasks::TaskStatus;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph, Wrap},
    Frame,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    AddingTask,
}

pub struct UiState {
    pub input_mode: InputMode,
    pub input_buffer: String,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            input_mode: InputMode::Normal,
            input_buffer: String::new(),
        }
    }
}

pub fn render(
    f: &mut Frame,
    state: &AppState,
    physics: &PhysicsBody,
    anim: &AnimationEngine,
    ui_state: &UiState,
) {
    let root_bg = Block::default().style(Style::default().bg(Color::Black).fg(Color::White));
    f.render_widget(root_bg, f.area());

    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(3),
        ])
        .split(f.area());

    render_header(f, main_chunks[0], state);

    let body_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(main_chunks[1]);

    render_habitat(f, body_chunks[0], state, physics, anim);
    render_sidebar(f, body_chunks[1], state);
    render_footer(f, main_chunks[2], ui_state, state.pet.focus_mode);
}

fn render_header(f: &mut Frame, area: Rect, state: &AppState) {
    let pet = &state.pet;
    let focus_badge = if pet.focus_mode {
        Span::styled(
            " [FOCUS] ",
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
    } else {
        Span::raw("")
    };

    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            format!(" {} ", pet.name),
            Style::default()
                .fg(Color::LightGreen)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("Lv.{} ", pet.level),
            Style::default().fg(Color::Yellow),
        ),
        Span::styled(
            format!("| {} ", pet.emotion.label()),
            Style::default().fg(emotion_color(pet.emotion)),
        ),
        Span::styled(
            format!("| Streak:{} ", pet.streak),
            Style::default().fg(Color::LightRed),
        ),
        focus_badge,
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray))
            .title(" MoodShell "),
    );
    f.render_widget(header, area);
}

fn render_habitat(
    f: &mut Frame,
    area: Rect,
    state: &AppState,
    physics: &PhysicsBody,
    anim: &AnimationEngine,
) {
    let habitat_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(if state.pet.focus_mode {
            Color::Cyan
        } else {
            Color::Gray
        }))
        .title(" Habitat ");

    let inner = habitat_block.inner(area);
    f.render_widget(habitat_block, area);

    if inner.width < 10 || inner.height < 6 {
        return;
    }

    // Speech banner pinned cleanly at top of habitat so it never overlaps the sprite
    let speech_h = 3.min(inner.height.saturating_sub(5));
    let speech_area = Rect::new(
        inner.x + 1,
        inner.y,
        inner.width.saturating_sub(2),
        speech_h,
    );
    f.render_widget(
        Paragraph::new(state.pet.speech.as_str())
            .wrap(Wrap { trim: true })
            .style(Style::default().fg(Color::White))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(emotion_color(state.pet.emotion)))
                    .title(format!(" {} ", state.pet.name)),
            ),
        speech_area,
    );

    let play_top = inner.y + speech_h;
    let play_h = inner.height.saturating_sub(speech_h + 1);
    if play_h < 2 {
        return;
    }

    // Ground floor line
    let floor_y = play_top + play_h - 1;
    let floor_line: String = "─".repeat(inner.width as usize);
    f.render_widget(
        Paragraph::new(floor_line).style(Style::default().fg(Color::DarkGray)),
        Rect::new(inner.x, floor_y, inner.width, 1),
    );

    // Particles
    for p in &anim.particles {
        let px = inner.x + (p.x as u16).min(inner.width.saturating_sub(1));
        let py = play_top + (p.y as u16).min(play_h.saturating_sub(1));
        let color = match p.kind {
            ParticleKind::Heart => Color::LightRed,
            ParticleKind::Star | ParticleKind::Sparkle => Color::Yellow,
            ParticleKind::Rain => Color::Blue,
            ParticleKind::Zzz => Color::Magenta,
        };
        f.render_widget(
            Paragraph::new(p.kind.glyph()).style(Style::default().fg(color)),
            Rect::new(px, py, 1, 1),
        );
    }

    // Multi-line companion sprite
    let art_lines: Vec<String> = if anim.current_art.is_empty() {
        state
            .pet
            .emotion
            .animated_sprite(
                state.pet.level,
                anim.frame_count,
                physics.impact_frames,
                physics.vy,
                physics.vx,
            )
            .lines()
            .map(|s| s.to_string())
            .collect()
    } else {
        anim.current_art.clone()
    };

    let sprite_h = (art_lines.len() as u16).min(play_h);
    let sprite_w = (art_lines
        .iter()
        .map(|l| l.chars().count())
        .max()
        .unwrap_or(8) as u16)
        .min(inner.width);

    let max_x = inner.width.saturating_sub(sprite_w);
    let max_y = play_h.saturating_sub(sprite_h);
    let pet_x = inner.x + (physics.x as u16).min(max_x);
    let pet_y = play_top + (physics.y as u16).min(max_y);

    let text = Text::from(
        art_lines
            .into_iter()
            .take(sprite_h as usize)
            .map(Line::from)
            .collect::<Vec<_>>(),
    );

    f.render_widget(
        Paragraph::new(text).style(
            Style::default()
                .fg(emotion_color(state.pet.emotion))
                .add_modifier(Modifier::BOLD),
        ),
        Rect::new(pet_x, pet_y, sprite_w, sprite_h),
    );
}

fn render_sidebar(f: &mut Frame, area: Rect, state: &AppState) {
    let vitals_h = 9.min(area.height / 2).max(4);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(vitals_h), Constraint::Min(4)])
        .split(area);

    let vitals_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(" Vitals ");
    let vitals_inner = vitals_block.inner(chunks[0]);
    f.render_widget(vitals_block, chunks[0]);

    let s = &state.pet.stats;
    let xp_pct = ((state.pet.xp as f64 / state.pet.xp_for_next_level() as f64) * 100.0) as u16;
    let metrics = [
        ("HP ", s.health as u16, Color::LightRed),
        ("MOT", s.motivation as u16, Color::Yellow),
        ("NRG", s.energy as u16, Color::LightGreen),
        ("FCS", s.focus as u16, Color::Cyan),
        ("TRT", s.trust as u16, Color::LightMagenta),
        ("XP ", xp_pct.min(100), Color::White),
    ];

    let rows = vitals_inner.height.min(metrics.len() as u16);
    if rows > 0 {
        let constraints: Vec<Constraint> = (0..rows).map(|_| Constraint::Length(1)).collect();
        let gauge_rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(vitals_inner);

        for (idx, (label, val, color)) in metrics.iter().take(rows as usize).enumerate() {
            let gauge = Gauge::default()
                .gauge_style(Style::default().fg(*color).bg(Color::DarkGray))
                .percent((*val).min(100))
                .label(format!("{} {}%", label, val));
            f.render_widget(gauge, gauge_rows[idx]);
        }
    }

    let items: Vec<ListItem> = state
        .tasks
        .tasks
        .iter()
        .rev()
        .take(12)
        .map(|t| {
            let (icon, style) = match t.status {
                TaskStatus::Pending => ("[ ]", Style::default().fg(Color::White)),
                TaskStatus::Completed => ("[+]", Style::default().fg(Color::Green)),
                TaskStatus::Missed => ("[x]", Style::default().fg(Color::Red)),
            };
            let due_str = t
                .due_at
                .map(|d| format!(" ({})", d.format("%H:%M")))
                .unwrap_or_default();
            ListItem::new(format!("{} {} {}{}", icon, t.id, t.title, due_str)).style(style)
        })
        .collect();

    let tasks_list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray))
            .title(format!(" Tasks ({}) ", state.tasks.completed_count())),
    );
    f.render_widget(tasks_list, chunks[1]);
}

fn render_footer(f: &mut Frame, area: Rect, ui_state: &UiState, focus_mode: bool) {
    let text = match ui_state.input_mode {
        InputMode::Normal => {
            let focus_label = if focus_mode {
                "[f] End Focus"
            } else {
                "[f] Focus"
            };
            format!(
                " [a] Add  [d] Done  [p] Pet  [j] Jump  {}  [q] Quit ",
                focus_label
            )
        }
        InputMode::AddingTask => {
            format!(
                " Task (@30m): {}_  (Enter=save, Esc=cancel)",
                ui_state.input_buffer
            )
        }
    };

    let footer = Paragraph::new(text)
        .style(Style::default().fg(Color::Cyan))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        );
    f.render_widget(footer, area);
}

fn emotion_color(emotion: Emotion) -> Color {
    match emotion {
        Emotion::Happy => Color::LightGreen,
        Emotion::Sad => Color::Blue,
        Emotion::Excited => Color::Yellow,
        Emotion::Angry => Color::LightRed,
        Emotion::Sleepy => Color::Magenta,
        Emotion::Lonely => Color::DarkGray,
        Emotion::Proud => Color::LightCyan,
        Emotion::Concerned => Color::LightYellow,
    }
}
