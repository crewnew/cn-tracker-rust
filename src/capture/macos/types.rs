// MacOS capture types (must be cross-platform)
use super::super::pc_common;
use crate::{
    prelude::*,
    scripting::{Variable, VariableMapType}
};
use std::{sync::Arc, rc::Rc, time::Duration, collections::HashMap};
use sysinfo::{Process, ProcessExt};

#[derive(Debug, Serialize, Deserialize)]
pub struct MacOSCaptureArgs {}

#[cfg(target_os = "macos")]
impl CapturerCreator for MacOSCaptureArgs {
    fn create_capturer(&self) -> anyhow::Result<Box<dyn Capturer>> {
        Ok(Box::new(super::appkit::MacOSCapturer::init()))
    }
}

#[cfg(not(target_os = "macos"))]
impl CapturerCreator for MacOSCaptureArgs {
    fn create_capturer(&self) -> anyhow::Result<Box<dyn Capturer>> {
        anyhow::bail!("Not on MacOS!")
    }
}



#[derive(Debug, Serialize, Deserialize, TypeScriptify, Clone)]
pub struct MacOSEventData {
    #[serde(default)]
    pub os_info: util::OsInfo,
    pub duration_since_user_input: Duration,
    pub windows: Vec<MacOSWindow>,
}

impl ExtractInfo for MacOSEventData {
    fn extract_info(&self) -> Option<Tags> {
        if pc_common::is_idle(self.duration_since_user_input) {
            return None;
        }

        let mut tags = Tags::new();

        self.os_info.to_partial_general_software(&mut tags);

        for window in &self.windows {
            let cls = Some((window.process.name.clone(), "".to_owned()));

            let window_title = match window.title {
                Some(ref string) => string.as_str(),
                None => "Unknown",
            };

            tags.extend(pc_common::match_software(
                window_title,
                &cls,
                Some(&window.process.exe),
                None,
                None,
            ));
        }

        Some(tags)
    }
}

#[derive(Debug, Serialize, Deserialize, TypeScriptify, Clone)]
pub struct MacOSWindow {
    pub title: Option<String>,
    pub process: MacOSProcessData,
}

impl From<MacOSWindow> for Variable {
    fn from(data: MacOSWindow) -> Self {
        Variable::Map(data.into())
    }
}

impl From<MacOSWindow> for VariableMapType {
    fn from(data: MacOSWindow) -> Self {
        let mut map = Self::default();
        if let Some(title) = data.title {
            map.insert(Rc::new("TITLE".into()), title.into());
        }
        let data = data.process;
        map.insert(Rc::new("NAME".into()), data.name.into());
        map.insert(Rc::new("CMD".into()), data.cmd.iter().map(|a| a.as_str()).collect::<String>().into());
        map.insert(Rc::new("EXE".into()), data.exe.into());
        map.insert(Rc::new("CWD".into()), data.cwd.into());
        map.insert(Rc::new("MEMORY".into()), (data.memory_kB as usize).into());
        map.insert(Rc::new("STATUS".into()), data.status.into());
        map.insert(Rc::new("START_TIME".into()), data.start_time.to_string().into());
        if let Some(cpu_usage) = data.cpu_usage {
            map.insert(Rc::new("CPU_USAGE".into()), cpu_usage.into()); 
        }
        map
    }
}

#[derive(Debug, Serialize, Deserialize, TypeScriptify, Clone)]
pub struct MacOSProcessData {
    pub name: String,
    pub cmd: Vec<String>,
    pub exe: String,
    pub cwd: String,
    pub memory_kB: i64,
    pub status: String,
    pub start_time: DateTime<Utc>,
    pub cpu_usage: Option<f32>, // can be NaN -> null
}

impl From<&Process> for MacOSProcessData {
    fn from(other: &Process) -> Self {
        MacOSProcessData {
            name: other.name().to_string(),
            exe: other.exe().to_string_lossy().to_string(),
            status: other.status().to_string(),
            cmd: other.cmd().to_vec(),
            cwd: other.cwd().to_string_lossy().to_string(),
            memory_kB: other.memory() as i64,
            start_time: util::unix_epoch_millis_to_date((other.start_time() as i64) * 1000),
            cpu_usage: Some(other.cpu_usage()),
        }
    }
}
