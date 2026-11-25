use chrono::{DateTime, Duration, Local};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TaskStatus {
    Pending,
    Completed,
    Missed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub created_at: DateTime<Local>,
    pub due_at: Option<DateTime<Local>>,
    pub completed_at: Option<DateTime<Local>>,
    pub status: TaskStatus,
    pub warned_soon: bool,
    pub penalized_missed: bool,
}

impl Task {
    pub fn new(title: String, due_in_minutes: Option<i64>) -> Self {
        let now = Local::now();
        let due_at = due_in_minutes.map(|mins| now + Duration::minutes(mins));
        Self {
            id: Uuid::new_v4().to_string()[..8].to_string(),
            title,
            created_at: now,
            due_at,
            completed_at: None,
            status: TaskStatus::Pending,
            warned_soon: false,
            penalized_missed: false,
        }
    }

    pub fn is_overdue(&self, now: DateTime<Local>) -> bool {
        if self.status != TaskStatus::Pending {
            return false;
        }
        match self.due_at {
            Some(due) => now > due,
            None => false,
        }
    }

    pub fn is_due_soon(&self, now: DateTime<Local>) -> bool {
        if self.status != TaskStatus::Pending {
            return false;
        }
        match self.due_at {
            Some(due) => due > now && (due - now) <= Duration::minutes(30),
            None => false,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TaskManager {
    pub tasks: Vec<Task>,
}

impl TaskManager {
    pub fn add_task(&mut self, title: String, due_in_minutes: Option<i64>) -> Task {
        let task = Task::new(title, due_in_minutes);
        self.tasks.push(task.clone());
        task
    }

    pub fn complete_task(&mut self, query: Option<&str>) -> Option<Task> {
        let now = Local::now();
        let idx = if let Some(q) = query {
            let q_lower = q.to_lowercase();
            self.tasks.iter().position(|t| {
                t.status == TaskStatus::Pending
                    && (t.id.to_lowercase().starts_with(&q_lower)
                        || t.title.to_lowercase().contains(&q_lower))
            })
        } else {
            self.tasks
                .iter()
                .position(|t| t.status == TaskStatus::Pending)
        };

        if let Some(i) = idx {
            self.tasks[i].status = TaskStatus::Completed;
            self.tasks[i].completed_at = Some(now);
            Some(self.tasks[i].clone())
        } else {
            None
        }
    }

    pub fn pending_tasks(&self) -> Vec<&Task> {
        self.tasks
            .iter()
            .filter(|t| t.status == TaskStatus::Pending)
            .collect()
    }

    pub fn completed_count(&self) -> usize {
        self.tasks
            .iter()
            .filter(|t| t.status == TaskStatus::Completed)
            .count()
    }
}
