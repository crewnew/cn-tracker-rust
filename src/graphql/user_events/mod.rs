use super::*;
use crate::capture::pc_common::{Event, Process};

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "src/graphql/user_events/InsertUserEvent.graphql",
    response_derive = "Debug",
    normalization = "rust",
    skip_serializing_none = "true"
)]
pub struct InsertUserEvent;
use insert_user_event::*;

pub fn send_user_event(user_event: impl Into<Variables>) -> anyhow::Result<()> {
    let request_body = InsertUserEvent::build_query(user_event.into());

    debug!("Sending User Event");

    let response_data: Response<ResponseData> = HTTP_CLIENT
        .post(ENDPOINT.as_str())
        .json(&request_body)
        .send()?
        .json()?;

    if let Some(errors) = response_data.errors {
        anyhow::bail!("Query Failed with Errors: {:?}", errors);
    }

    debug!("User Event Sent Successfully");

    Ok(())
}

impl From<Event> for Variables {
    fn from(event: Event) -> Self {
        let Event {
            windows,
            rule,
            network,
            keyboard,
            mouse,
            seconds_since_last_input,
            ..
        } = event;

        let mut user_event_windows = vec![];

        for window in windows.into_iter() {
            let title = window.title;
            let Process {
                cmd,
                cpu_usage,
                exe,
                memory,
                name,
                start_time,
                status,
                ..
            } = window.process;

            let user_process = UserProcessesInsertInput {
                id: None,
                created_at: Some("now()".to_owned()),
                updated_at: Some("now()".to_owned()),
                user: None,
                user_event_windows: None,
                user_id: Some(USER_ID.clone()),
                cmd: Some(cmd),
                cpu_usage,
                exe: Some(exe),
                memory: Some(memory as i64),
                name: Some(name),
                start_time: Some(start_time as i64),
                status: Some(status),
            };

            let user_process = UserProcessesObjRelInsertInput {
                data: user_process,
                on_conflict: None,
            };

            let user_event_window = UserEventWindowsInsertInput {
                id: None,
                created_at: Some("now()".to_owned()),
                updated_at: Some("now()".to_owned()),
                user_event_id: None,
                user_process_id: None,
                user_event: None,
                user_id: Some(USER_ID.clone()),
                is_focused: Some(true),
                user_process: Some(user_process),
                title,
            };

            user_event_windows.push(user_event_window);
        }

        let user_event_window_arr_rel_insert_input = UserEventWindowsArrRelInsertInput {
            data: user_event_windows,
            on_conflict: None,
        };

        let user_events_insert_input = UserEventsInsertInput {
            id: None,
            project_rule: None,
            created_at: Some("now()".to_owned()),
            updated_at: Some("now()".to_owned()),
            user_event_files: None,
            user_id: Some(USER_ID.clone()),
            project_rule_id: rule.map(|r| r.id),
            keyboard: Some(keyboard as i64),
            mouse: Some(mouse as i64),
            seconds_since_last_input: Some(seconds_since_last_input as i64),
            ssid_id: network.map(|n| n.id),
            user_event_windows: Some(user_event_window_arr_rel_insert_input),
        };

        Self {
            data: user_events_insert_input,
        }
    }
}
