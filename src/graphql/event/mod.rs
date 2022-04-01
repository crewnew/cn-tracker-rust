use super::{Id, Variables, CLIENT, SaveToDb};
use crate::{prelude::NewDbEvent, util::random_uuid};

const CREATE_EVENT_MUTATION: &str = include_str!("CreateEvent.graphql");

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Event {
    id: String,
    duration: i64,
    keyboard: usize,
    mouse: usize,
    timestamp: i64,
    event_data: EventData,
}

// ToDo: Remove later.
impl From<NewDbEvent> for Event {
    fn from(new_db_event: NewDbEvent) -> Self {
        Self {
            id: random_uuid(),
            duration: new_db_event.duration_ms,
            keyboard: 0,
            mouse: 0,
            timestamp: new_db_event.timestamp_unix_ms.0.timestamp(),
            event_data: EventData::new(new_db_event.data_type, new_db_event.data),
        }
    }
}

#[async_trait]
impl SaveToDb for Event {
    async fn save_to_db(&self) -> anyohw::Result<()> {
        let variables = Variables { data: self };
        let data = CLIENT
            .query_with_vars::<Id, Variables>(CREATE_EVENT_MUTATION, variables)
            .await
            .map_err(|err| anyhow::anyhow!("Couldn't save data: {}", err))?;
        Ok(()) 
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct EventData {
    id: String,
    #[serde(rename = "type")]
    event_type: String,
    value: String,
}

impl EventData {
    pub fn new(event_type: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            id: random_uuid(),
            event_type: event_type.into(),
            value: value.into(),
        }
    }
}
