use super::{Id, Variables, CLIENT};
use crate::capture::pc_common::{get_network_ssid, NetworkInfo};

const CREATE_NETWORK_INFO_MUTATION: &str = include_str!("CreateNetwork.graphql");
const GET_NETWORK_ID_QUERY: &str = include_str!("QueryNetwork.graphql");

pub fn get_network_info() -> anyhow::Result<NetworkInfo> {
    #[derive(Debug, Deserialize)]
    struct Data {
        networks: Vec<Id>,
    }
    #[derive(Serialize)]
    struct Network<'a> {
        name: &'a str,
    }

    let ssid = get_network_ssid().ok_or_else(|| anyhow!("Couldn't get Network SSID"))?;

    debug!("{}", ssid);

    let variables = Variables { data: &ssid };

    let mut data = CLIENT
        .query_with_vars::<Data, Variables>(GET_NETWORK_ID_QUERY, variables)
        .map_err(|err| anyhow!("Couldn't save data: {}", err))?;

    debug!("{:?}", data);

    if data.networks.is_empty() {
        #[derive(Debug, Deserialize)]
        struct CreateNetworkResult {
            #[serde(rename = "create_networks_item")]
            network: Id,
        }
        let variables = Variables {
            data: &Network { name: &ssid },
        };
        let data = CLIENT
            .query_with_vars::<CreateNetworkResult, Variables>(
                CREATE_NETWORK_INFO_MUTATION,
                variables,
            )
            .map_err(|err| anyhow!("Couldn't save data: {}", err))?;

        return Ok(NetworkInfo {
            id: data.network.id,
            name: ssid,
        });
    }

    let id = data.networks.pop().unwrap().id;

    debug!("{}", id);

    Ok(NetworkInfo { id, name: ssid })
}
