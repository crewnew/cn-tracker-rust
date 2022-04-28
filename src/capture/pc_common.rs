#![allow(clippy::trivial_regex)]

use crate::{
    scripting::{Variable, VariableMapType},
    util,
};
use regex::Regex;

use std::{convert::TryFrom, sync::atomic::AtomicUsize};
use sysinfo::ProcessExt;

lazy_static::lazy_static! {
    static ref FORMATTED_TITLE_MATCH: Regex = Regex::new(r#"🛤([a-z]{2,5})🠚(.*)🠘"#).unwrap();

    static ref FORMATTED_TITLE_SPLIT: Regex = Regex::new("🙰").unwrap();
    static ref FORMATTED_TITLE_KV: Regex = Regex::new("^([a-z0-9]+)=(.*)$").unwrap();
    static ref JSON_TITLE: Regex = Regex::new(r#"\{".*[^\\]"}"#).unwrap();

    pub static ref KEYSTROKES: AtomicUsize = AtomicUsize::new(0);
    pub static ref MOUSE_CLICKS: AtomicUsize = AtomicUsize::new(0);
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event<'a> {
    pub windows: Vec<Window>,
    pub rule_id: Option<&'a str>,
    pub keyboard: usize,
    pub mouse: usize,
    pub seconds_since_last_input: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Window {
    pub title: Option<String>,
    pub process: Process,
}

impl From<Window> for VariableMapType {
    fn from(window: Window) -> Self {
        let mut map = Self::default();
        if let Some(title) = window.title {
            map.insert("TITLE", title.into());
        }
        map.insert("NAME", window.process.name.into());
        map.insert("CMD", window.process.cmd.into());
        map.insert("EXE", window.process.exe.into());
        map.insert("CWD", window.process.cwd.into());
        map.insert("MEMORY", (window.process.memory as usize).into());
        map.insert("STATUS", window.process.status.into());
        map.insert("START_TIME", window.process.start_time.to_string().into());
        if let Some(cpu_usage) = window.process.cpu_usage {
            map.insert("CPU_USAGE", cpu_usage.into());
        }
        map
    }
}

impl TryFrom<&VariableMapType> for Window {
    type Error = anyhow::Error;
    fn try_from(variable_map: &VariableMapType) -> Result<Self, Self::Error> {
        let title: Option<String> = match variable_map.get("TITLE") {
            Some(Variable::RcStr(string)) => Some((**string).clone()),
            None => None,
            _ => anyhow::bail!("TITLE is not a String"),
        };
        let name = match variable_map.get("NAME") {
            Some(Variable::RcStr(string)) => (**string).clone(),
            _ => anyhow::bail!("NAME is not a String"),
        };
        let cmd = match variable_map.get("CMD") {
            Some(Variable::RcStr(string)) => (**string).clone(),
            _ => anyhow::bail!("CMD is not a String"),
        };
        let exe = match variable_map.get("EXE") {
            Some(Variable::RcStr(string)) => (**string).clone(),
            _ => anyhow::bail!("EXE is not a String"),
        };
        let cwd = match variable_map.get("CWD") {
            Some(Variable::RcStr(string)) => (**string).clone(),
            _ => anyhow::bail!("CWD is not a String"),
        };
        let memory = match variable_map.get("MEMORY") {
            Some(Variable::Int(int)) => *int as i64,
            _ => anyhow::bail!("MEMORY is not an Int"),
        };
        let status = match variable_map.get("STATUS") {
            Some(Variable::RcStr(string)) => (**string).clone(),
            _ => anyhow::bail!("STATUS is not a String"),
        };
        let start_time = match variable_map.get("START_TIME") {
            Some(Variable::RcStr(string)) => (**string).clone(),
            _ => anyhow::bail!("START_TIME is not a String"),
        };
        let cpu_usage = match variable_map.get("CPU_USAGE") {
            Some(Variable::Float(float)) => Some(*float),
            None => None,
            _ => anyhow::bail!("CPU_USAGE is not a Float"),
        };
        Ok(Window {
            title,
            process: Process {
                name,
                cmd,
                exe,
                cwd,
                memory,
                status,
                start_time,
                cpu_usage,
            },
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Process {
    pub name: String,
    pub cmd: String,
    pub exe: String,
    pub cwd: String,
    pub memory: i64,
    pub status: String,
    pub start_time: String,
    pub cpu_usage: Option<f32>,
}

impl From<&sysinfo::Process> for Process {
    fn from(process: &sysinfo::Process) -> Self {
        Process {
            name: process.name().to_string(),
            exe: process.exe().to_string_lossy().to_string(),
            status: process.status().to_string(),
            cmd: process.cmd().to_vec().concat(),
            cwd: process.cwd().to_string_lossy().to_string(),
            memory: process.memory() as i64,
            start_time: util::unix_epoch_millis_to_date((process.start_time() as i64) * 1000)
                .to_string(),
            cpu_usage: Some(process.cpu_usage()),
        }
    }
}
