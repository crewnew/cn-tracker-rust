use super::*;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "src/graphql/user_ssid/UserSsidQuery.graphql",
    skip_serializing_none = "true",
    normalization = "rust"
)]
struct UserSsidQuery;
use self::user_ssid_query::*;

pub fn get_user_ssid(ssid: impl Into<String>) -> anyhow::Result<String> {
    let request_body = UserSsidQuery::build_query(Variables { data: ssid.into() });

    let response_data: Response<ResponseData> = HTTP_CLIENT
        .post(ENDPOINT.as_str())
        .json(&request_body)
        .send()?
        .json()?;

    if let Some(errors) = response_data.errors {
        anyhow::bail!("Query Failed with Errors: {:?}", errors);
    }

    Ok(response_data
        .data
        .unwrap()
        .user_ssid
        .into_iter()
        .next()
        .ok_or_else(|| anyhow!("Couldn't find SSID"))?
        .ssid
        .id)
}
