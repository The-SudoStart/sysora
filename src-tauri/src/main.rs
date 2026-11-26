// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod cli;

use clap::Parser;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    
    // Check if any recognized CLI subcommand or output flag was provided
    let is_cli = args.iter().skip(1).any(|a| {
        matches!(
            a.as_str(),
            "status"
                | "processes"
                | "disk"
                | "battery"
                | "fans"
                | "--json"
                | "-j"
                | "-h"
                | "--help"
                | "-V"
                | "--version"
        )
    });

    if is_cli {
        let parsed = cli::Cli::parse();
        cli::run_cli(parsed);
        return;
    }

    sysora_lib::run();
}
