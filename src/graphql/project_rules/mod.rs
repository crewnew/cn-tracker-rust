use super::*;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "src/graphql/project_rules/UserProjectRulesQuery.graphql",
    response_derives = "Debug",
    normalization = "rust"
)]
struct UserProjectRulesQuery;

/// Gets the User's projects and the rules associated with those projects.
pub fn get_user_rules() -> anyhow::Result<Vec<(String, String)>> {
    use user_project_rules_query::ResponseData;

    let mut vec: Vec<(String, String)> = vec![];

    let request_body = UserProjectRulesQuery::build_query(user_project_rules_query::Variables {});

    debug!("Getting User Rules");

    let response_data: Response<ResponseData> = HTTP_CLIENT
        .post(ENDPOINT.as_str())
        .json(&request_body)
        .send()?
        .json()?;

    let user_projects = match response_data.data {
        Some(data) => data.user_project,
        None => anyhow::bail!("Query Failed with Errors: {:?}", response_data.errors),
    };

    for project in user_projects.into_iter() {
        for project_rule in project.project.project_rules {
            let project_id = project_rule.id;
            let rule_body = project_rule.rule.body;
            vec.push((project_id, rule_body));
        }
    }

    debug!("User Rules Retrieved");

    Ok(vec)
}
