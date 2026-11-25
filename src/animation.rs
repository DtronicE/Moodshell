use crate::emotion::Emotion;
use rand::Rng;
use serde::Deserialize;
use std::process::Command;

#[derive(Debug, Clone, Copy)]
pub enum ParticleKind {
    Heart,
    Star,
    Rain,
    Zzz,
    Sparkle,
}

impl ParticleKind {
    pub fn glyph(&self) -> &'static str {
        match self {
            ParticleKind::Heart => "♥",
            ParticleKind::Star => "★",
            ParticleKind::Rain => "│",
            ParticleKind::Zzz => "z",
            ParticleKind::Sparkle => "✦",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Particle {
    pub x: f64,
    pub y: f64,
    pub vx: f64,
    pub vy: f64,
    pub ttl: f64,
    pub kind: ParticleKind,
}

#[derive(Deserialize)]
struct PythonArtResponse {
    lines: Vec<String>,
}

#[derive(Debug, Default)]
pub struct AnimationEngine {
    pub particles: Vec<Particle>,
    pub frame_count: u64,
    pub blink: bool,
    pub current_art: Vec<String>,
    last_art_key: Option<(Emotion, u32, u64)>,
    spawn_accumulator: f64,
}

impl AnimationEngine {
    pub fn spawn_burst(&mut self, origin_x: f64, origin_y: f64, kind: ParticleKind, count: usize) {
        let mut rng = rand::thread_rng();
        for _ in 0..count {
            self.particles.push(Particle {
                x: origin_x + rng.gen_range(-3.0..3.0),
                y: origin_y + rng.gen_range(-1.0..1.0),
                vx: rng.gen_range(-12.0..12.0),
                vy: rng.gen_range(-16.0..-5.0),
                ttl: rng.gen_range(0.8..1.8),
                kind,
            });
        }
    }

    fn fetch_python_art(emotion: Emotion, level: u32, frame_idx: u64) -> Option<Vec<String>> {
        let output = Command::new("python")
            .args([
                "scripts/companion.py",
                emotion.label(),
                &level.to_string(),
                &frame_idx.to_string(),
            ])
            .output()
            .ok()?;

        if !output.status.success() {
            return None;
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let parsed: PythonArtResponse = serde_json::from_str(stdout.trim()).ok()?;
        if parsed.lines.is_empty() {
            None
        } else {
            Some(parsed.lines)
        }
    }

    pub fn update(
        &mut self,
        dt: f64,
        emotion: Emotion,
        level: u32,
        impact_frames: u8,
        vx: f64,
        vy: f64,
        pet_x: f64,
        pet_y: f64,
        bounds_w: f64,
    ) {
        self.frame_count = self.frame_count.wrapping_add(1);
        self.blink = (self.frame_count % 120) > 114;

        let art_phase = self.frame_count / 10;
        let key = (emotion, level, art_phase);
        if self.last_art_key != Some(key) {
            self.last_art_key = Some(key);
            if let Some(py_lines) = Self::fetch_python_art(emotion, level, art_phase) {
                self.current_art = py_lines;
            } else {
                self.current_art = emotion
                    .animated_sprite(level, self.frame_count, impact_frames, vy, vx)
                    .lines()
                    .map(|l| l.to_string())
                    .collect();
            }
        }

        self.spawn_accumulator += dt;
        let mut rng = rand::thread_rng();

        match emotion {
            Emotion::Sad | Emotion::Lonely => {
                if self.spawn_accumulator >= 0.08 {
                    self.spawn_accumulator = 0.0;
                    self.particles.push(Particle {
                        x: rng.gen_range(1.0..(bounds_w - 1.0).max(2.0)),
                        y: 1.0,
                        vx: -1.5,
                        vy: rng.gen_range(14.0..22.0),
                        ttl: 2.0,
                        kind: ParticleKind::Rain,
                    });
                }
            }
            Emotion::Sleepy => {
                if self.spawn_accumulator >= 0.65 {
                    self.spawn_accumulator = 0.0;
                    self.particles.push(Particle {
                        x: pet_x + 4.0,
                        y: (pet_y - 1.0).max(1.0),
                        vx: 2.0,
                        vy: -3.5,
                        ttl: 1.8,
                        kind: ParticleKind::Zzz,
                    });
                }
            }
            Emotion::Excited | Emotion::Proud => {
                if self.spawn_accumulator >= 0.25 {
                    self.spawn_accumulator = 0.0;
                    self.particles.push(Particle {
                        x: pet_x + rng.gen_range(-4.0..6.0),
                        y: pet_y,
                        vx: rng.gen_range(-5.0..5.0),
                        vy: rng.gen_range(-9.0..-3.0),
                        ttl: 1.2,
                        kind: ParticleKind::Star,
                    });
                }
            }
            _ => {
                self.spawn_accumulator = 0.0;
            }
        }

        for p in &mut self.particles {
            p.x += p.vx * dt;
            p.y += p.vy * dt;
            if matches!(p.kind, ParticleKind::Heart | ParticleKind::Star) {
                p.vy += 12.0 * dt;
            }
            p.ttl -= dt;
        }

        self.particles.retain(|p| p.ttl > 0.0 && p.y >= 0.0);
    }
}
