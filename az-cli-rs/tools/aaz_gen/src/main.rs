use aaz_gen::{emitter, parser};

use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "aaz-gen", about = "Generate Rust azrs commands from Python AAZ files")]
struct Cli {
    /// Path to the AAZ `latest/` directory (e.g. .../network/aaz/latest/network/)
    #[arg(short, long)]
    input: PathBuf,

    /// Output directory for generated Rust files
    #[arg(short, long)]
    output: PathBuf,

    /// Service name (e.g. "network", "compute")
    #[arg(short, long)]
    service: String,

    /// Only print what would be generated, don't write files
    #[arg(long)]
    dry_run: bool,

    /// Dump the parsed IR as JSON instead of emitting Rust
    #[arg(long)]
    dump_ir: bool,
}

fn main() {
    let cli = Cli::parse();

    eprintln!("Parsing AAZ files from: {}", cli.input.display());
    let module = parser::parse_aaz_directory(&cli.input, &cli.service);

    let total_commands: usize = module.groups.iter().map(|g| g.commands.len()).sum();
    eprintln!(
        "Found {} command groups, {} commands total.",
        module.groups.len(),
        total_commands
    );

    if total_commands == 0 {
        eprintln!("No commands found. Check your --input path.");
        std::process::exit(1);
    }

    if cli.dump_ir {
        let json = serde_json::to_string_pretty(&module).unwrap();
        println!("{json}");
        return;
    }

    if cli.dry_run {
        eprintln!("\nDry run — would generate:");
        for group in &module.groups {
            let mod_name = group.name.replace('-', "_").to_lowercase();
            eprintln!("  {}/{mod_name}.rs ({} commands)", cli.output.display(), group.commands.len());
            for cmd in &group.commands {
                eprintln!(
                    "    {} {} — {} {}",
                    cmd.cli_path,
                    if cmd.is_lro { "(LRO)" } else { "" },
                    cmd.operation.method,
                    cmd.operation.url_template
                );
            }
        }
        eprintln!("\n  {}/mod.rs", cli.output.display());
        return;
    }

    // Emit
    match emitter::emit_service(&module, &cli.output) {
        Ok(()) => {
            eprintln!(
                "\nGenerated {} files in {}",
                module.groups.len() + 1,
                cli.output.display()
            );
        }
        Err(e) => {
            eprintln!("Error writing files: {e}");
            std::process::exit(1);
        }
    }
}
