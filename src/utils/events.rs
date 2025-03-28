use chrono::{DateTime, Utc};

pub struct NotificationEvent {
    pub summary: String,
    pub date: DateTime<Utc>,
}
