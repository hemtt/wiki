pub use arma3_wiki::model::Command;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Report {
    pub passed_commands: Vec<String>,
    pub failed_commands: HashMap<String, Vec<String>>,
    pub outdated_commands: Vec<String>,
    pub unknown_types_commands: Option<Vec<(String, String)>>,
    pub passed_event_handlers: Option<serde_json::Value>,
    pub failed_event_handlers: Option<serde_json::Value>,
    pub outdated_event_handlers: Option<serde_json::Value>,
    pub updated_version: Option<serde_json::Value>,
}

pub fn get_command_status(name: &str, report: &Report) -> CommandStatus {
    if report.passed_commands.iter().any(|s| s == name) {
        CommandStatus::Passed
    } else if report.failed_commands.contains_key(name) {
        CommandStatus::Failed
    } else if report.outdated_commands.iter().any(|s| s == name) {
        CommandStatus::Outdated
    } else {
        CommandStatus::Unknown
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandStatus {
    Passed,
    Failed,
    Outdated,
    Unknown,
}

impl CommandStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Passed => "Passed",
            Self::Failed => "Failed",
            Self::Outdated => "Outdated",
            Self::Unknown => "Unknown",
        }
    }
}

pub fn load_report(path: &str) -> std::io::Result<Report> {
    let content = fs_err::read_to_string(path)?;
    let report = serde_json::from_str(&content)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    Ok(report)
}

pub fn load_commands(dir: &str) -> std::io::Result<Vec<Command>> {
    let mut commands = Vec::new();
    let entries = fs_err::read_dir(dir)?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if !path.is_file() {
            continue;
        }

        let filename = path.file_name().expect("Failed to get file name").to_string_lossy();
        if !filename.ends_with(".yml") && !filename.ends_with(".yaml") {
            continue;
        }

        match load_single_command(&path) {
            Ok(cmd) => commands.push(cmd),
            Err(e) => {
                eprintln!("Warning: Failed to load {filename}: {e}");
            }
        }
    }

    commands.sort_by(|a, b| a.name().cmp(b.name()));
    Ok(commands)
}

fn load_single_command(path: &std::path::Path) -> std::io::Result<Command> {
    let content = fs_err::read_to_string(path)?;
    let cmd = serde_yaml::from_str(&content)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    Ok(cmd)
}
