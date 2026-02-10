use async_trait::async_trait;
use jiff::Timestamp;
use uuid::Uuid;
pub enum LogType {
    Operation,
    Common,
}

impl LogType {
    pub fn to_string(&self) -> String {
        match self {
            LogType::Operation => "operation".to_string(),
            LogType::Common => "common".to_string(),
        }
    }
}

pub struct Log {
    pub id: Uuid,
    pub agenda_id: Uuid,
    pub content: String,
    pub create_at: Timestamp,
    pub log_type: LogType,
}

pub struct LogCreate {
    pub agenda_id: Uuid,
    pub content: String,
    pub log_type: LogType,
}

#[async_trait]
pub trait LogRepo {
    type Error: std::error::Error + Send + Sync + 'static;
    async fn create_log(&self, new_log: &LogCreate) -> Result<Uuid, Self::Error>;

    //TODO
    // async fn delete_log(&self, id: Uuid) -> Result<(), Self::Error>;
    // async fn get_logs_by_agenda_id(&self, agenda_id: Uuid) -> Result<Vec<Log>, Self::Error>;
    // async fn get_logs_by_time_range(
    //     &self,
    //     start: Timestamp,
    //     end: Timestamp,
    // ) -> Result<Vec<Log>, Self::Error>;
}
