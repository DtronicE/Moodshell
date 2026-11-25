use crate::config::IDLE_THRESHOLD_SECS;
use crate::emotion::Emotion;
use crate::personality::PersonalityMemory;
use crate::pet::Pet;
use crate::tasks::{TaskManager, TaskStatus};
use chrono::Local;

pub struct ReminderEngine;

impl ReminderEngine {
    /// Evaluates deadlines, idle decay, and personality triggers.
    pub fn evaluate(pet: &mut Pet, tasks: &mut TaskManager, memory: &PersonalityMemory) {
        let now = Local::now();

        // 1. Check overdue tasks
        for task in &mut tasks.tasks {
            if task.is_overdue(now) && !task.penalized_missed {
                task.status = TaskStatus::Missed;
                task.penalized_missed = true;
                pet.on_task_missed(&task.title);
                return;
            }
        }

        // 2. Check upcoming deadlines (within 30 mins)
        for task in &mut tasks.tasks {
            if task.is_due_soon(now) && !task.warned_soon {
                task.warned_soon = true;
                pet.emotion = Emotion::Concerned;
                pet.speech = format!(
                    "Deadline soon! \"{}\" is due in under 30 minutes!",
                    task.title
                );
                return;
            }
        }

        // 3. Skip idle decay if in Focus Mode
        if pet.focus_mode {
            return;
        }

        // 4. Evaluate idle duration
        let idle_secs = (now - pet.last_interaction).num_seconds();
        if idle_secs > IDLE_THRESHOLD_SECS * 3 {
            if pet.emotion != Emotion::Lonely {
                pet.emotion = Emotion::Lonely;
                pet.stats.motivation -= 0.2;
                pet.stats.clamp_all();
                pet.speech = Emotion::Lonely.random_thought().to_string();
            }
        } else if idle_secs > IDLE_THRESHOLD_SECS {
            if !matches!(pet.emotion, Emotion::Sleepy | Emotion::Angry) {
                if !tasks.pending_tasks().is_empty() {
                    pet.emotion = Emotion::Angry;
                    pet.speech = "(¬_¬) Still scrolling? You still have pending tasks!".to_string();
                } else {
                    pet.emotion = Emotion::Sleepy;
                    pet.speech = Emotion::Sleepy.random_thought().to_string();
                }
            }
        } else if rand::random::<u8>() < 3 {
            // Occasional proactive personality insight
            if let Some(insight) = memory.generate_insight() {
                pet.speech = insight;
            }
        }
    }
}
