use std::io;

mod generator;
mod models;

const DIST_COMMANDS: &str = "dist/commands";
const DIST_REPORT: &str = "dist/report.json";
const OUTPUT_DIR: &str = "dist-website/assets/data";

#[tokio::main]
async fn main() -> io::Result<()> {
    println!("🔨 Generating HEMTT Wiki metadata...");

    // Load report
    let report = models::load_report(DIST_REPORT)?;
    println!(
        "📊 Loaded report with {} passed commands",
        report.passed_commands.len()
    );

    // Load commands
    let commands = models::load_commands(DIST_COMMANDS)?;
    println!("📖 Loaded {} command definitions", commands.len());

    // Generate metadata
    generator::generate_metadata(&commands, &report, OUTPUT_DIR)?;

    println!("✅ Metadata generated successfully!");
    println!("📁 Output directory: {OUTPUT_DIR}");

    Ok(())
}
