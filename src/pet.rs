use crate::emotion::Emotion;
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PetStats {
    pub health: f32,     // 0.0 - 100.0
    pub motivation: f32, // 0.0 - 100.0
    pub energy: f32,     // 0.0 - 100.0
    pub focus: f32,      // 0.0 - 100.0
    pub trust: f32,      // 0.0 - 100.0
}

impl Default for PetStats {
    fn default() -> Self {
        Self {
            health: 85.0,
            motivation: 75.0,
            energy: 90.0,
            focus: 70.0,
            trust: 80.0,
        }
    }
}

impl PetStats {
    pub fn clamp_all(&mut self) {
        self.health = self.health.clamp(0.0, 100.0);
        self.motivation = self.motivation.clamp(0.0, 100.0);
        self.energy = self.energy.clamp(0.0, 100.0);
        self.focus = self.focus.clamp(0.0, 100.0);
        self.trust = self.trust.clamp(0.0, 100.0);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pet {
    pub name: String,
    pub level: u32,
    pub xp: u32,
    pub streak: u32,
    pub emotion: Emotion,
    pub speech: String,
    pub stats: PetStats,
    pub focus_mode: bool,
    pub focus_started_at: Option<DateTime<Local>>,
    pub total_focus_minutes: u64,
    pub last_interaction: DateTime<Local>,
}

impl Default for Pet {
    fn default() -> Self {
        Self {
            name: "Byte".to_string(),
            level: 1,
            xp: 0,
            streak: 0,
            emotion: Emotion::Happy,
            speech: "Boot sequence complete! Ready to build something cool?".to_string(),
            stats: PetStats::default(),
            focus_mode: false,
            focus_started_at: None,
            total_focus_minutes: 0,
            last_interaction: Local::now(),
        }
    }
}

impl Pet {
    pub fn xp_for_next_level(&self) -> u32 {
        100 + (self.level - 1) * 50
    }

    pub fn gain_xp(&mut self, amount: u32) -> bool {
        self.xp += amount;
        let mut leveled_up = false;
        while self.xp >= self.xp_for_next_level() {
            self.xp -= self.xp_for_next_level();
            self.level += 1;
            leveled_up = true;
        }
        leveled_up
    }

    pub fn on_task_completed(&mut self, task_title: &str) -> bool {
        self.last_interaction = Local::now();
        self.streak += 1;
        self.stats.health += 4.0;
        self.stats.motivation += 12.0;
        self.stats.focus += 8.0;
        self.stats.trust += 6.0;
        self.stats.energy -= 3.0;
        self.stats.clamp_all();

        let leveled = self.gain_xp(45 + (self.streak.min(10) * 5));

        if leveled {
            self.emotion = Emotion::Proud;
            self.speech = format!(
                "LEVEL {} REACHED! \"{}\" crushed! Look at my new form!",
                self.level, task_title
            );
        } else if self.streak >= 3 {
            self.emotion = Emotion::Proud;
            self.speech = format!(
                "Great job! {} tasks in a row! \"{}\" is history!",
                self.streak, task_title
            );
        } else {
            self.emotion = Emotion::Excited;
            self.speech = format!("Great job! Finished \"{}\"!", task_title);
        }

        leveled
    }

    pub fn on_task_missed(&mut self, task_title: &str) {
        self.streak = 0;
        self.stats.health -= 8.0;
        self.stats.motivation -= 15.0;
        self.stats.trust -= 12.0;
        self.stats.clamp_all();

        self.emotion = Emotion::Sad;
        self.speech = format!("You forgot \"{}\"... (╥﹏╥)", task_title);
    }

    pub fn on_petted(&mut self) {
        self.last_interaction = Local::now();
        self.stats.motivation += 5.0;
        self.stats.energy += 5.0;
        self.stats.trust += 3.0;
        self.stats.clamp_all();
        self.gain_xp(10);

        self.emotion = Emotion::Happy;
        self.speech = "Hehe, thanks for the headpats! Back to work?".to_string();
    }

    pub fn toggle_focus_mode(&mut self) -> Option<u64> {
        self.last_interaction = Local::now();
        if !self.focus_mode {
            self.focus_mode = true;
            self.focus_started_at = Some(Local::now());
            self.emotion = Emotion::Proud;
            self.speech =
                "Focus Mode ON. Sitting quietly while you enter flow state...".to_string();
            None
        } else {
            self.focus_mode = false;
            let elapsed_mins = self
                .focus_started_at
                .map(|start| (Local::now() - start).num_minutes().max(1) as u64)
                .unwrap_or(1);
            self.focus_started_at = None;
            self.total_focus_minutes += elapsed_mins;

            let xp_reward = (elapsed_mins as u32) * 5;
            self.gain_xp(xp_reward);
            self.stats.focus += (elapsed_mins as f32 * 0.8).min(25.0);
            self.stats.motivation += 10.0;
            self.stats.trust += 5.0;
            self.stats.clamp_all();

            self.emotion = Emotion::Excited;
            self.speech = format!(
                "Focus complete! {}m logged (+{} XP). Amazing discipline!",
                elapsed_mins, xp_reward
            );
            Some(elapsed_mins)
        }
    }

    pub fn evolution_title(&self) -> &'static str {
        match self.level {
            1..=9 => "Stage I: Terminal Sprout",
            10..=49 => "Stage II: Ascended Daemon",
            50..=99 => "Stage III: Kernel Sage",
            _ => "Stage IV: Celestial Root Guardian",
        }
    }
}
