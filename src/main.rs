pub mod args;
pub mod utils;

pub mod impersonate;
pub mod token;

use args::*;
use impersonate::*;
use token::*;

use env_logger::Builder;
use log::error;

use std::io::{self, Write};

fn main() {
    // Check if arguments exist → CLI mode
    if std::env::args().len() > 1 {
        let common_args = extract_args();

        // Build logger based on verbosity
        Builder::new()
            .filter(Some("irs"), common_args.verbose)
            .filter_level(log::LevelFilter::Error)
            .init();

        // Run CLI logic
        run_cli(&common_args);
        return;
    }

    // Interactive shell mode (no args)
    Builder::new()
        .filter(Some("irs"), log::LevelFilter::Error)
        .filter_level(log::LevelFilter::Error)
        .init();

    println!("Launching interactive shell...");
    run_interactive();
}

/// CLI mode logic
fn run_cli(opt: &Options) {
    match opt.mode {
        Mode::List => {
            if let Err(err) = enabling_sedebug() {
                error!("Failed to enable SeDebug: {err}");
            }
            if let Err(err) = enum_token() {
                error!("Failed to list tokens: {err}");
            }
        }

        Mode::Exec => {
            enabling_seimpersonate().ok();
            enabling_sedebug().ok();

            if let Err(err) = run_command(opt.pid, opt.cmd.clone()) {
                error!("Exec mode failed: {err}");
            }
        }

        Mode::Spawn => {
            enabling_seimpersonate().ok();
            enabling_sedebug().ok();

            if let Err(err) = spawn_process(opt.pid, opt.cmd.clone()) {
                error!("Spawn mode failed: {err}");
            }
        }

        _ => error!("Unknown mode"),
    }
}

/// Interactive shell
fn run_interactive() {
    loop {
        print!("pg> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).unwrap() == 0 {
            break;
        }

        let mut parts = input.trim().split_whitespace();
        let command = match parts.next() {
            Some(c) => c.to_lowercase(),
            None => continue,
        };

        match command.as_str() {
            "list" => {
                enabling_sedebug().ok();
                if let Err(err) = enum_token() {
                    println!("Failed to list tokens: {err}");
                }
            }

            "exec" => {
                let pid = match parts.next() {
                    Some(p) => p.parse::<u32>().unwrap_or(0),
                    None => {
                        println!("Usage: exec <pid> <command>");
                        continue;
                    }
                };
                let cmd = parts.collect::<Vec<&str>>().join(" ");
                if cmd.is_empty() {
                    println!("Usage: exec <pid> <command>");
                    continue;
                }

                enabling_seimpersonate().ok();
                enabling_sedebug().ok();

                if let Err(err) = run_command(pid, cmd) {
                    println!("Exec failed: {err}");
                }
            }

            "spawn" => {
                let pid = match parts.next() {
                    Some(p) => p.parse::<u32>().unwrap_or(0),
                    None => {
                        println!("Usage: spawn <pid> <binary_path>");
                        continue;
                    }
                };
                let binary = parts.collect::<Vec<&str>>().join(" ");
                if binary.is_empty() {
                    println!("Usage: spawn <pid> <binary_path>");
                    continue;
                }

                enabling_seimpersonate().ok();
                enabling_sedebug().ok();

                if let Err(err) = spawn_process(pid, binary) {
                    println!("Spawn failed: {err}");
                }
            }

            "help" => {
                println!("Available commands:");
                println!("  list");
                println!("  exec <pid> <command>");
                println!("  spawn <pid> <binary>");
                println!("  exit");
            }

            "exit" | "quit" => break,

            other => println!("Unknown command: {}", other),
        }
    }
}
