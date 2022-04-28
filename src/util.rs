use anyhow::Context;
use chrono::{Date, DateTime, NaiveDate, NaiveDateTime, Utc};
pub fn unix_epoch_millis_to_date(timestamp: i64) -> DateTime<Utc> {
    let timestamp_s = timestamp / 1000;
    let timestamp_us = (timestamp % 1000) * 1_000_000;
    let naive_datetime = NaiveDateTime::from_timestamp(timestamp_s, timestamp_us as u32);
    DateTime::from_utc(naive_datetime, Utc)
}

/*fn timestamp_to_iso_string(timestamp: i64) -> String {
    unix_epoch_millis_to_date(timestamp).to_rfc3339()
}*/

pub fn iso_string_to_datetime(s: &str) -> anyhow::Result<DateTime<Utc>> {
    // https://tc39.es/proposal-temporal/docs/iso-string-ext.html
    // allow time zone suffix, e.g. 2007-12-03T10:15:30+01:00[Europe/Paris]
    if s.ends_with("]") {
        let splitchar = s.rfind("[").context("Invalid date, broken TZ")?;
        let (s, _tz) = (&s[0..splitchar], &s[splitchar..]);
        //let tz = chrono_tz::Tz::from_str(tz)
        //    .map_err(|e| anyhow::anyhow!("could not parse tz: {e}"))?;

        return Ok(
            DateTime::<chrono::FixedOffset>::parse_from_rfc3339(s)?.with_timezone(&chrono::Utc)
        );
    }
    Ok(DateTime::<chrono::FixedOffset>::parse_from_rfc3339(s)
        .context("iso_string_to_datetime")?
        .with_timezone(&chrono::Utc))
}

pub fn iso_string_to_date(s: &str) -> anyhow::Result<Date<Utc>> {
    Ok(Date::from_utc(
        NaiveDate::parse_from_str(s, "%F").context("iso_string_to_date")?,
        Utc,
    ))
}

pub fn random_uuid() -> String {
    uuid::Uuid::new_v4().hyphenated().to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsInfo {
    // e.g. "Arch Linux" or "Windows"
    pub os_type: String,
    pub version: String,
    pub batteries: Option<i32>, // useful for determining pc vs laptop
    pub hostname: String,
    pub username: Option<String>,
    pub machine_id: Option<String>,
}
// TODO: remove defaults after rewrite
impl Default for OsInfo {
    fn default() -> OsInfo {
        OsInfo {
            os_type: "Arch Linux".to_string(),
            version: "rolling".to_string(),
            batteries: Some(0),
            hostname: "phirearch".to_string(),
            machine_id: None,
            username: Some("".to_string()),
        }
    }
}

pub fn get_os_info() -> OsInfo {
    let os_info1 = os_info::get();
    let batteries = battery::Manager::new()
        .and_then(|e| e.batteries())
        .map(|e| e.count() as i32)
        .ok();
    let machine_id = std::fs::read_to_string("/etc/machine-id")
        .map(|e| e.trim().to_string())
        .ok();
    OsInfo {
        os_type: os_info1.os_type().to_string(),
        version: format!("{}", os_info1.version()),
        hostname: whoami::hostname(),
        machine_id,
        batteries,
        username: Some(whoami::username()),
    }
}
