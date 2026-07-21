//! Alioth CLI Binary
//!
//! 提供代码生成命令行工具

use alioth_gen::cli::CliRunner;

fn main() {
    if let Err(e) = CliRunner::run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
