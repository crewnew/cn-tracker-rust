use super::{Id, ENDPOINT, HTTP_CLIENT};
use crate::capture::pc_common::NetworkInfo;

lazy_static! {
    static ref NETWORKS_ENDPOINT: String = format!("{}/ssids", ENDPOINT.as_str());
    static ref USER_SSID_ENDPOINT: String = format!("{}/user/ssid", ENDPOINT.as_str());
}

pub fn get_network_info(ssid: impl Into<String>) -> anyhow::Result<NetworkInfo> {
    #[derive(Debug, Deserialize)]
    struct Data {
        ssids: Vec<Id>,
    }
    #[derive(Serialize, Debug)]
    struct Network<'a> {
        name: &'a str,
    }
    #[derive(Serialize, Debug)]
    struct OnConflict<'a> {
        constraint: &'a str,
        update_columns: [&'a str; 0],
    }
    #[derive(Serialize, Debug)]
    struct Ssid<'a> {
        data: Network<'a>,
        on_conflict: OnConflict<'a>,
    }
    #[derive(Serialize, Debug)]
    struct SsidWrapper<'a> {
        ssid: Ssid<'a>,
    }
    #[derive(Serialize, Debug)]
    struct UserSsid<'a> {
        argument: SsidWrapper<'a>,
    }

    let ssid = ssid.into();

    debug!("{}", ssid);

    let url = format!("{}?name={}", NETWORKS_ENDPOINT.as_str(), ssid);

    let mut networks = HTTP_CLIENT.get(url).send()?.json::<Data>()?.ssids;

    debug!("{:?}", networks);

    if networks.is_empty() {
        let payload = UserSsid {
            argument: SsidWrapper {
                ssid: Ssid {
                    data: Network { name: &ssid },
                    on_conflict: OnConflict {
                        constraint: "ssids_name_key",
                        update_columns: [],
                    },
                },
            },
        };

        debug!("Payload: {}", serde_json::to_string(&payload)?);

        let id = HTTP_CLIENT
            .post(USER_SSID_ENDPOINT.as_str())
            .json(&payload)
            .send()?
            .text()?;
        error!("{}", id);

        return Ok(NetworkInfo { id, name: ssid });
    }

    let id = networks.pop().unwrap().id;

    debug!("{}", id);

    Ok(NetworkInfo { id, name: ssid })
}
