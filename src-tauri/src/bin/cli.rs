use clap::{Parser, Subcommand};
use forge_env_lib::core::{detect, hosts, mirrors, models, provider, storage};

#[derive(Parser)]
#[command(
    name = "forge-env",
    about = "Forge Env — Development environment manager CLI",
    version,
    long_about = "Manage language runtimes, mirrors, system dependencies, and environment configuration from the command line."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Show host and runtime status
    Status {
        /// Host ID to inspect (default: native)
        #[arg(long)]
        host: Option<String>,
    },

    /// List detected projects and suggested runtimes
    Projects,

    /// Apply a mirror preset
    Mirror {
        /// Preset name (Tsinghua, Aliyun, Huawei Cloud, Company Proxy)
        #[arg(short = 'p', long, default_value = "Tsinghua")]
        preset: String,
        /// Host ID target
        #[arg(long, default_value = "native")]
        host: String,
    },

    /// Export environment template bundle
    Export {
        /// Host ID to export from
        #[arg(long, default_value = "native")]
        host: String,
        /// Output file path (default: stdout)
        #[arg(short = 'o', long)]
        output: Option<String>,
    },

    /// Import environment template bundle
    Import {
        /// Input file path (or stdin with -)
        file: String,
    },

    /// Run environment health checks
    Doctor {
        /// Host ID to check
        #[arg(long, default_value = "native")]
        host: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Status { host } => cmd_status(host.as_deref()),
        Commands::Projects => cmd_projects(),
        Commands::Mirror { preset, host } => cmd_mirror(&preset, &host),
        Commands::Export { host, output } => cmd_export(&host, output.as_deref()),
        Commands::Import { file } => cmd_import(&file),
        Commands::Doctor { host } => cmd_doctor(&host),
    }
}

fn cmd_status(host_id: Option<&str>) {
    let hosts = hosts::discover_hosts();
    if hosts.is_empty() {
        eprintln!("No hosts detected.");
        return;
    }

    let target_id = host_id.unwrap_or("native");

    println!("=== Forge Env Status ===\n");

    println!("Hosts ({}):", hosts.len());
    for host in &hosts {
        let marker = if host.id == target_id { " ←" } else { "" };
        println!(
            "  [{}] {} — {} ({}){marker}",
            host.status, host.label, host.architecture, host.shell
        );
    }

    println!("\nRuntimes on {target_id}:");
    let runtimes = provider::detect_runtimes_for_host(target_id);
    if runtimes.is_empty() {
        println!("  No managed runtimes detected.");
    } else {
        for runtime in &runtimes {
            let active = runtime
                .installed
                .iter()
                .find(|i| i.active)
                .map(|i| i.version.as_str())
                .unwrap_or("none");
            println!(
                "  {} ({}) — active: {} — installed: {}",
                runtime.family,
                runtime.provider,
                active,
                runtime.installed.len()
            );
        }
    }

    println!("\nMirror preset:");
    let storage = storage::Storage::new().ok();
    let preset = storage
        .as_ref()
        .and_then(|s| s.load().ok())
        .and_then(|s| s.applied_mirror_preset)
        .unwrap_or_else(|| "none".to_string());
    println!("  Applied: {preset}");
}

fn cmd_projects() {
    println!("=== Detected Projects ===\n");

    let cwd = std::env::current_dir().unwrap_or_default();
    let projects = detect::scan_projects(cwd.as_path());
    if projects.is_empty() {
        println!("No project markers detected in the current directory.");
        return;
    }

    for project in &projects {
        println!(
            "{} [{}] — {}",
            project.name, project.health, project.path
        );
        println!("  Markers: {}", project.markers.join(", "));
        if !project.suggested_runtimes.is_empty() {
            println!("  Suggested runtimes:");
            for suggestion in &project.suggested_runtimes {
                println!("    {} {} — {}", suggestion.family, suggestion.version, suggestion.reason);
            }
        }
        if !project.risk_flags.is_empty() {
            println!("  Risk flags:");
            for flag in &project.risk_flags {
                println!("    ⚠ {flag}");
            }
        }
        println!();
    }
}

fn cmd_mirror(preset: &str, host_id: &str) {
    let mirror_preset = mirrors::preset_by_name(preset);
    match mirror_preset {
        Some(p) => {
            println!("Mirror preset: {preset}");
            println!("  npm registry: {}", p.npm_registry);
            println!("  pip index: {}", p.pip_index_url);
            println!("  cargo sparse: {}", p.cargo_sparse_registry);
            println!("\nApplying to host {host_id}...");
            match mirrors::apply_preset_for_host(host_id, preset) {
                Ok(changes) => {
                    for change in &changes {
                        println!("  ✓ {change}");
                    }
                    println!("\nMirror preset applied successfully.");
                }
                Err(e) => {
                    eprintln!("Error: {}", e.message);
                    std::process::exit(1);
                }
            }
        }
        None => {
            eprintln!("Unknown preset: {preset}");
            eprintln!("Available presets: Tsinghua, Aliyun, Huawei Cloud, Company Proxy");
            std::process::exit(1);
        }
    }
}

fn cmd_export(host_id: &str, output: Option<&str>) {
    println!("Exporting environment template from {host_id}...");

    let runtimes = provider::detect_runtimes_for_host(host_id);
    let hosts_list = hosts::discover_hosts();

    let snapshot = models::TemplateSnapshot {
        schema_version: 2,
        generated_at: chrono_now(),
        hosts: hosts_list,
        host_snapshots: vec![],
        runtimes,
        system_dependencies: vec![],
        applied_mirror_preset: None,
    };

    let json = serde_json::to_string_pretty(&snapshot).unwrap_or_else(|_| "{}".to_string());

    match output {
        Some(path) => {
            std::fs::write(path, &json).expect("Failed to write export file");
            println!("Exported to {path}");
        }
        None => {
            println!("\n{json}");
        }
    }
}

fn cmd_import(file: &str) {
    let json = if file == "-" {
        let mut buf = String::new();
        std::io::Read::read_to_string(&mut std::io::stdin(), &mut buf).expect("Failed to read stdin");
        buf
    } else {
        std::fs::read_to_string(file).unwrap_or_else(|e| {
            eprintln!("Failed to read {file}: {e}");
            std::process::exit(1);
        })
    };

    match serde_json::from_str::<models::TemplateSnapshot>(&json) {
        Ok(snapshot) => {
            println!("Bundle schema version: {}", snapshot.schema_version);
            println!("Hosts: {}", snapshot.hosts.len());
            println!("Runtime families: {}", snapshot.runtimes.len());
            println!("\nRun the full Tauri app for interactive import: npm run tauri:dev");
        }
        Err(e) => {
            eprintln!("Invalid bundle: {e}");
            std::process::exit(1);
        }
    }
}

fn cmd_doctor(host_id: &str) {
    println!("=== Forge Env Doctor ({host_id}) ===\n");

    let runtimes = provider::detect_runtimes_for_host(host_id);
    let mut issues = 0;

    for runtime in &runtimes {
        let has_installed = !runtime.installed.is_empty();
        let has_active = runtime.installed.iter().any(|i| i.active);

        if !has_installed {
            println!("⚠ {} — not installed ({})", runtime.family, runtime.provider);
            issues += 1;
        } else if !has_active {
            println!("⚠ {} — installed but no active version", runtime.family);
            issues += 1;
        } else {
            let active = runtime.installed.iter().find(|i| i.active).unwrap();
            println!("✓ {} {} — {}", runtime.family, active.version, runtime.provider);
        }
    }

    if issues == 0 {
        println!("\nAll runtimes look healthy.");
    } else {
        println!("\n{issues} issue(s) found. Consider installing missing runtimes via the Forge Env GUI.");
    }
}

fn chrono_now() -> String {
    format!(
        "{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    )
}
