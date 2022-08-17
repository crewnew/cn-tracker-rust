use super::*;

mod insert_user_ssid_mutation;
mod user_ssid_query;

use insert_user_ssid_mutation::insert_user_ssid;
use user_ssid_query::get_user_ssid;

pub fn get_or_insert_user_ssid(ssid: impl Into<String>) -> anyhow::Result<String> {
    let ssid = ssid.into();

    match get_user_ssid(ssid.clone()) {
        Ok(id) => return Ok(id),
        Err(err) => {
            debug!("{}", err);
        }
    };

    Ok(insert_user_ssid(ssid)?)
}
