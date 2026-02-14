use async_trait::async_trait;
use jiff::Timestamp;
use std::error::Error;
use uuid::Uuid;

pub enum AgendaStatus {
    Stored,
    Ongoing,
    Terminated,
}

impl AgendaStatus {
    pub fn to_string(&self) -> String {
        match self {
            AgendaStatus::Stored => "stored".to_string(),
            AgendaStatus::Ongoing => "ongoing".to_string(),
            AgendaStatus::Terminated => "terminated".to_string(),
        }
    }
}

pub struct Agenda {
    pub id: Uuid,
    pub title: String,
    pub agenda_status: AgendaStatus,
    pub initiate_at: Timestamp,
    pub terminate_at: Timestamp,
}

pub struct AgendaCreate {
    pub title: String,
    pub agenda_status: AgendaStatus,
    pub terminate_at: Timestamp,
}

pub struct AgendaUpdate {
    pub title: Option<String>,
    pub agenda_status: Option<AgendaStatus>,
    pub terminate_at: Option<Timestamp>,
}

#[async_trait]
pub trait AgendaRepo {
    type Error: Error + Send + Sync + 'static;
    async fn create_agenda(&self, agenda: &AgendaCreate) -> Result<Uuid, Self::Error>;

    async fn delete_agenda_by_id(&self, id: Uuid) -> Result<(), Self::Error>;
    async fn update_agenda(&self, id: Uuid, update: &AgendaUpdate) -> Result<(), Self::Error>;
    async fn get_agenda_by_id(&self, id: Uuid) -> Result<Option<Agenda>, Self::Error>;
    async fn get_agendas_by_title(&self, title: &str) -> Result<Vec<Agenda>, Self::Error>;
    async fn get_agendas_by_status(&self, status: Option<&str>)
    -> Result<Vec<Agenda>, Self::Error>;
    async fn count_agendas_by_status(&self, status: Option<&str>) -> Result<u64, Self::Error>;
    async fn get_agendas_by_terminate_time_range(
        &self,
        start: Timestamp,
        end: Timestamp,
    ) -> Result<Vec<Agenda>, Self::Error>;
    // More query methods if needed
}
