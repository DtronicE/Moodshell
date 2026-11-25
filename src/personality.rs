use chrono::{DateTime, Local, Timelike};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PersonalityMemory {
    /// Counts user task/focus interactions by hour of day (0..23)
    pub hourly_activity: HashMap<u32, u32>,
    /// Tracks last interaction timestamp per extracted topic/keyword
    pub topic_last_seen: HashMap<String, DateTime<Local>>,
    /// Tracks frequency per keyword
    pub topic_counts: HashMap<String, u32>,
}

impl PersonalityMemory {
    pub fn record_activity(&mut self, text: &str) {
        let now = Local::now();
        let hour = now.hour();
        *self.hourly_activity.entry(hour).or_insert(0) += 1;

        for word in Self::extract_keywords(text) {
            self.topic_last_seen.insert(word.clone(), now);
            *self.topic_counts.entry(word).or_insert(0) += 1;
        }
    }

    fn extract_keywords(text: &str) -> Vec<String> {
        let stop_words = [
            "the", "and", "for", "with", "fix", "add", "update", "finish", "work", "on", "to", "a",
            "in", "my", "project", "task",
        ];
        text.split(|c: char| !c.is_alphanumeric())
            .map(|w| w.to_lowercase())
            .filter(|w| w.len() >= 3 && !stop_words.contains(&w.as_str()))
            .collect()
    }

    /// Generates an intelligent, personalized observation based on habit history.
    pub fn generate_insight(&self) -> Option<String> {
        let now = Local::now();

        // 1. Check for neglected projects (3+ days since last touched)
        for (topic, last_seen) in &self.topic_last_seen {
            let days = (now - *last_seen).num_days();
            let count = self.topic_counts.get(topic).copied().unwrap_or(0);
            if days >= 3 && count >= 2 {
                let capitalized = Self::capitalize(topic);
                return Some(format!(
                    "You haven't touched your {} project in {} days. The {} deserves attention.",
                    topic, days, capitalized
                ));
            }
        }

        // 2. Check peak productivity hour habit
        if let Some((&best_hour, &hits)) = self.hourly_activity.iter().max_by_key(|(_, &v)| v) {
            if hits >= 3 {
                let top_topic = self
                    .topic_counts
                    .iter()
                    .max_by_key(|(_, &v)| v)
                    .map(|(k, _)| Self::capitalize(k))
                    .unwrap_or_else(|| "deep work".to_string());

                let formatted_hour = if best_hour == 0 {
                    "12 AM".to_string()
                } else if best_hour < 12 {
                    format!("{} AM", best_hour)
                } else if best_hour == 12 {
                    "12 PM".to_string()
                } else {
                    format!("{} PM", best_hour - 12)
                };

                return Some(format!(
                    "You usually work on {} around {}. Ready to keep the streak alive?",
                    top_topic, formatted_hour
                ));
            }
        }

        None
    }

    fn capitalize(s: &str) -> String {
        let mut chars = s.chars();
        match chars.next() {
            None => String::new(),
            Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        }
    }
}
