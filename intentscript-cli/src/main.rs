use clap::{Parser, Subcommand};
use std::process;

mod commands;
mod error;
mod formatter;

use commands::{build, explain, fmt, lint, run};

#[derive(Parser)]
#[command(name = "intentscript")]
#[command(version, about = "IntentScript compiler and runtime", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Compile IntentScript source to IR
    Build {
        /// Input source file
        input: String,
        /// Output IR file
        #[arg(short, long)]
        output: Option<String>,
        /// Output JSON diagnostics for CI
        #[arg(long)]
        json: bool,
    },
    /// Execute IR with runtime
    Run {
        /// Input IR file
        input: String,
        /// Host adapter to use (default: mock)
        #[arg(long)]
        host: Option<String>,
        /// Output JSON diagnostics for CI
        #[arg(long)]
        json: bool,
    },
    /// Lint IntentScript source
    Lint {
        /// Input source file
        input: String,
        /// Output JSON diagnostics for CI
        #[arg(long)]
        json: bool,
    },
    /// Format IntentScript source
    Fmt {
        /// Input source file
        input: String,
        /// Check formatting without writing
        #[arg(long)]
        check: bool,
        /// Output JSON diagnostics for CI
        #[arg(long)]
        json: bool,
    },
    /// Explain execution plan
    Explain {
        /// Input IR file
        input: String,
        /// Optional execution log file
        #[arg(long)]
        log: Option<String>,
    },
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Build { input, output, json } => build::execute(&input, output.as_deref(), json),
        Commands::Run { input, host, json } => run::execute(&input, host.as_deref(), json),
        Commands::Lint { input, json } => lint::execute(&input, json),
        Commands::Fmt { input, check, json } => fmt::execute(&input, check, json),
        Commands::Explain { input, log } => explain::execute(&input, log.as_deref()),
    };

    match result {
        Ok(exit_code) => process::exit(exit_code),
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    }
}
