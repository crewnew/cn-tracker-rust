use super::*;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "src/graphql/user_ssid/InsertUserSsid.graphql",
    normalization = "rust",
    skip_serializing_none = "true"
)]
struct InsertUserSsid;
use insert_user_ssid::*;

pub fn insert_user_ssid(ssid: impl Into<String>) -> anyhow::Result<String> {
    let ssids_insert_input = SsidsInsertInput {
        id: None,
        created_at: None,
        updated_at: None,
        user_ssids: None,
        name: Some(ssid.into()),
    };

    let ssids_obj_rel_insert_input = SsidsObjRelInsertInput {
        data: ssids_insert_input,
        on_conflict: None,
    };

    let user_ssid_insert_input = UserSsidInsertInput {
        id: None,
        ssid_id: None,
        user: None,
        user_id: Some(USER_ID.clone()),
        created_at: None,
        updated_at: None,
        ssid: Some(ssids_obj_rel_insert_input),
    };

    let request_body = InsertUserSsid::build_query(Variables {
        data: user_ssid_insert_input,
    });

    debug!("Inserting User SSID");

    let response_data: Response<ResponseData> = HTTP_CLIENT
        .post(ENDPOINT.as_str())
        .json(&request_body)
        .send()?
        .json()?;

    if let Some(errors) = response_data.errors {
        anyhow::bail!("Query Failed with Errors: {:?}", errors);
    }

    debug!("Inserted User SSID");

    Ok(response_data.data.unwrap().insert_user_ssid_one.unwrap().id)
}
