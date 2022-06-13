use serde_json::Value;

use super::{SaveToDb, ENDPOINT, HTTP_CLIENT};
use crate::capture::pc_common::Event;

lazy_static! {
    static ref EVENTS_ENDPOINT: String = format!("{}/items/events", ENDPOINT.as_str());
}

impl SaveToDb for Event {
    fn save_to_db(&self) -> anyhow::Result<()> {
        let result: Value = HTTP_CLIENT
            .post(EVENTS_ENDPOINT.as_str())
            .json(self)
            .send()?
            .json()?;
        if result.get("data").is_none() {
            anyhow::bail!("SaveToDb for Event failed: {:?}", result);
        }
        Ok(())
    }
}
