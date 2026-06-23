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
        /// Host adapter to use (default: real)
        #[arg(long)]
        host: Option<String>,
        /// Task input as key=value (repeatable)
        #[arg(long = "input", value_name = "KEY=VALUE")]
        inputs: Vec<String>,
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

fn parse_input_args(args: &[String]) -> error::Result<Vec<(String, String)>> {
    let mut parsed = Vec::new();
    for arg in args {
        let (key, value) = arg
            .split_once('=')
            .ok_or_else(|| error::CliError::InvalidInput(format!(
                "Invalid --input '{}': expected key=value",
                arg
            )))?;
        parsed.push((key.to_string(), value.to_string()));
    }
    Ok(parsed)
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Build { input, output, json } => build::execute(&input, output.as_deref(), json),
        Commands::Run {
            input,
            host,
            inputs,
            json,
        } => match parse_input_args(&inputs) {
            Ok(parsed_inputs) => run::execute(&input, host.as_deref(), &parsed_inputs, json),
            Err(e) => Err(e),
        },
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
