use std::fs;

use clap::Parser;
use ezio::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
enum Target {
    #[serde(alias = "riscv")]
    RISCV,
    #[serde(alias = "wasm")]
    WASM,
    #[serde(alias = "shuorv")]
    SHUORV,
}

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    #[serde(default)]
    optimization: Vec<String>,
    #[serde(default)]
    emit_ir: bool,
    target: Target,
}

/// Come language build system
#[derive(clap::Parser, Debug)]
struct Args {
    #[command(subcommand)]
    action: Action,
}

#[derive(clap::Subcommand, Debug, PartialEq, PartialOrd, Ord, Eq, Clone)]
enum Action {
    /// Build a come project
    Build,
    /// Create a new come project
    New { name: String },
}

fn main() {
    let args = Args::parse();
    let current_dir = std::env::current_dir().unwrap();
    match args.action {
        Action::Build => {
            let config_path = current_dir.join("road.toml");
            let target_dir = current_dir.join("target");
            let target_filename = current_dir.file_name().unwrap();
            let config = file::read(config_path);
            let config: Config = toml::from_str(&config).unwrap();
            match config.target {
                Target::RISCV => {
                    compile_to_asm(&target_dir, &current_dir, target_filename, config);
                    std::process::Command::new("shuasm")
                        .arg("-i")
                        .arg(format!(
                            "{}.asm",
                            target_dir.join(target_filename).display()
                        ))
                        .arg("-o")
                        .arg(current_dir.join(format!("{}.bin", target_filename.to_str().unwrap())))
                        .output()
                        .expect("failed to execute assembler");
                }
                Target::WASM => {
                    unimplemented!()
                }
                Target::SHUORV => {
                    unimplemented!()
                }
            }
        }
        Action::New { name } => {
            let project_dir = current_dir.join(name);
            fs::create_dir_all(&project_dir).unwrap();
            let config = Config {
                optimization: vec![
                    "RemoveOnlyOnceStore".to_string(),
                    "RemoveLoadDirectlyAfterStore".to_string(),
                    "RemoveUnusedRegister".to_string(),
                    "MemoryToRegister".to_string(),
                    "RemoveUnusedRegister".to_string(),
                ],
                emit_ir: false,
                target: Target::RISCV,
            };
            let config = toml::to_string(&config).unwrap();
            file::write(project_dir.join("road.toml"), &config);
            file::write(project_dir.join("main.come"), "fn main() -> () {}");
        }
    }
}

fn compile_to_asm(
    target_dir: &std::path::PathBuf,
    current_dir: &std::path::PathBuf,
    target_filename: &std::ffi::OsStr,
    config: Config,
) {
    fs::create_dir_all(&target_dir).unwrap();
    let mut compiler_cmd = std::process::Command::new("come");
    compiler_cmd
        .arg("-i")
        .arg(current_dir.join("main.come").display().to_string())
        .arg("-o")
        .arg(format!(
            "{}.asm",
            target_dir.join(target_filename).display()
        ));
    if config.emit_ir {
        compiler_cmd
            .arg("--emit-ir")
            .arg(format!("{}.ir", target_dir.join(target_filename).display()));
    }
    if !config.optimization.is_empty() {
        let optimization = config.optimization.join(",");
        compiler_cmd.arg("-O").arg(format!("{}", optimization));
    }
    compiler_cmd.output().expect("failed to execute compiler");
}
