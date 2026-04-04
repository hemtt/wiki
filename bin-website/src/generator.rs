use crate::models::{Command, CommandStatus, Report, get_command_status};
use serde_json::json;
use std::io;

pub fn generate_metadata(
    commands: &[(String, Command)],
    report: &Report,
    output_dir: &str,
) -> io::Result<()> {
    // Create subdirectories for data
    fs_err::create_dir_all(output_dir)?;
    fs_err::create_dir_all(format!("{output_dir}/commands"))?;

    // Generate commands.json with only basic info
    generate_commands_file(commands, report, output_dir)?;

    // Generate individual command files for passed commands
    generate_individual_commands(commands, report, output_dir)?;

    // Generate filters.json with available filters
    generate_filters_file(commands)?;

    println!("✓ Generated commands.json");
    println!("✓ Generated individual command files");
    println!("✓ Generated filters.json");

    Ok(())
}

fn generate_commands_file(
    commands: &[(String, Command)],
    report: &Report,
    output_dir: &str,
) -> io::Result<()> {
    let mut commands_data: Vec<serde_json::Value> = commands
        .iter()
        .map(|(filename, cmd)| {
            let status = get_command_status(filename, report);
            let mut cmd_json = json!({
                "name": cmd.name(),
                "id": filename, // Use filename as ID for linking to individual page
                "description": cmd.description(),
                "groups": cmd.groups(),
                "status": status.as_str(),
            });

            // Add errors if command has failed
            if let Some(errors) = report.failed_commands.get(cmd.name()) {
                cmd_json["errors"] = json!(errors);
            }

            cmd_json
        })
        .collect();

    // Add failed commands that don't have YAML definitions
    for (cmd_name, errors) in &report.failed_commands {
        // Check if we already have this command from YAML
        if !commands_data
            .iter()
            .any(|c| c["name"].as_str() == Some(cmd_name))
        {
            let cmd_json = json!({
                "name": cmd_name,
                "id": cmd_name,
                "description": "No documentation available",
                "groups": [],
                "status": "Failed",
                "errors": errors,
            });
            commands_data.push(cmd_json);
        }
    }

    // Sort by name
    commands_data.sort_by(|a, b| {
        let a_name = a["name"].as_str().unwrap_or("");
        let b_name = b["name"].as_str().unwrap_or("");
        a_name.cmp(b_name)
    });

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();

    let json = serde_json::json!({
        "timestamp": now,
        "version": "1.0",
        "commands": commands_data,
        "passed": report.passed_commands.len(),
        "failed": report.failed_commands.len(),
        "total": commands_data.len()
    });

    let path = format!("{output_dir}/commands.json");
    fs_err::write(&path, serde_json::to_string_pretty(&json)?)?;
    Ok(())
}

fn generate_individual_commands(
    commands: &[(String, Command)],
    report: &Report,
    output_dir: &str,
) -> io::Result<()> {
    for (filename, cmd) in commands {
        let status = get_command_status(filename, report);

        // Only generate individual files for passed commands
        if status != CommandStatus::Passed {
            continue;
        }

        let mut cmd_json = json!({
            "name": cmd.name(),
            "id": filename,
            "description": cmd.description(),
            "groups": cmd.groups(),
            "status": status.as_str(),
            "syntax": cmd.syntax(),
            "examples": cmd.examples(),
            "see_also": cmd.see_also(),
            "argument_loc": format!("{:?}", cmd.argument_loc()),
            "effect_loc": format!("{:?}", cmd.effect_loc()),
            "problem_notes": cmd.problem_notes(),
        });

        // Add errors if any (shouldn't happen for Passed, but for safety)
        if let Some(errors) = report.failed_commands.get(cmd.name()) {
            cmd_json["errors"] = json!(errors);
        }

        if !cmd.alias().is_empty() {
            cmd_json["alias"] = json!(cmd.alias());
        }

        let filename = format!("{filename}.json");
        let path = format!("{output_dir}/commands/{filename}");
        fs_err::write(&path, serde_json::to_string_pretty(&cmd_json)?)?;
    }

    Ok(())
}

fn generate_filters_file(commands: &[(String, Command)]) -> io::Result<()> {
    let mut groups = std::collections::BTreeSet::new();
    let mut statuses = std::collections::BTreeSet::new();

    for (_, cmd) in commands {
        for group in cmd.groups() {
            groups.insert(group.clone());
        }
        // For now, statuses are added dynamically based on report
        statuses.insert("Passed".to_string());
        statuses.insert("Failed".to_string());
        // statuses.insert("Outdated".to_string());
    }

    let filters = json!({
        "groups": groups.iter().collect::<Vec<_>>(),
        "statuses": statuses.iter().collect::<Vec<_>>(),
        "sortOptions": [
            {"value": "name", "label": "Name (A-Z)"},
            {"value": "name-desc", "label": "Name (Z-A)"},
            {"value": "status", "label": "Status"},
            {"value": "group", "label": "Group"}
        ]
    });

    let path = "dist-website/assets/data/filters.json";
    fs_err::create_dir_all("dist-website/assets/data")?;
    fs_err::write(path, serde_json::to_string_pretty(&filters)?)?;
    Ok(())
}
