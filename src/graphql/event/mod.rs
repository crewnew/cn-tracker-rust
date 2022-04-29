use super::{Id, SaveToDb, Variables, CLIENT};
use crate::capture::pc_common::Event;

const CREATE_EVENT_MUTATION: &str = include_str!("CreateEvent.graphql");

impl SaveToDb for Event {
    fn save_to_db(&self) -> anyhow::Result<()> {
        dbg!(self);
        let variables = Variables { data: self };
        CLIENT
            .query_with_vars::<Id, Variables>(CREATE_EVENT_MUTATION, variables)
            .map_err(|err| anyhow::anyhow!("Couldn't save data: {}", err))?;
        Ok(())
    }
}
