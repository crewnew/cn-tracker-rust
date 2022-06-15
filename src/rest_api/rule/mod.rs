use super::{ENDPOINT, HTTP_CLIENT};
use crate::scripting::Rule;

lazy_static! {
    static ref RULES_ENDPOINT: String = format!("{}/items/rules", ENDPOINT.as_str());
}

pub fn get_rules() -> anyhow::Result<Vec<Rule>> {
    #[derive(Serialize, Deserialize)]
    struct Data {
        data: Vec<Rule>,
    }

    let data: Data = HTTP_CLIENT.get(RULES_ENDPOINT.as_str()).send()?.json()?;

    Ok(data.data)
}
