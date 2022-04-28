use super::CLIENT;
use crate::scripting::Rule;
use gql_client::GraphQLError;

const QUERY: &str = include_str!("QueryRules.graphql");

pub fn get_rules() -> Result<Vec<Rule>, GraphQLError> {
    #[derive(Serialize, Deserialize)]
    struct Rules {
        rules: Vec<Rule>,
    }

    let data = CLIENT.query::<Rules>(QUERY)?;

    Ok(data.rules)
}
