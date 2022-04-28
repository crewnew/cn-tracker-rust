use super::{Id, SaveToDb, Variables, CLIENT};
use crate::capture::pc_common::Event;

const CREATE_EVENT_MUTATION: &str = include_str!("CreateEvent.graphql");

impl SaveToDb for Event<'_> {
    fn save_to_db(&self) -> anyhow::Result<()> {
        let variables = Variables { data: self };
        let _data = CLIENT
            .query_with_vars::<Id, Variables>(CREATE_EVENT_MUTATION, variables)
            .map_err(|err| anyhow::anyhow!("Couldn't save data: {}", err))?;
        Ok(())
    }
}
