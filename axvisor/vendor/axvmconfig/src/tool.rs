use std::env;
use std::error::Error;
use std::fs;
use std::path::Path;

use clap::{Args, Parser, Subcommand};

use crate::templates::get_vm_config_template;
use crate::AxVMCrateConfig;

#[derive(Parser)]
#[command(name = "axvmconfig")]
#[command(about = "A simple VM configuration tool for ArceOS-Hypervisor.", long_about = None)]
#[command(args_conflicts_with_subcommands = true)]
#[command(flatten_help = true)]
pub struct CLI {
    #[command(subcommand)]
    pub subcmd: CLISubCmd,
}

#[derive(Subcommand)]
#[command(args_conflicts_with_subcommands = true)]
#[command(flatten_help = true)]
pub enum CLISubCmd {
    /// Parse the configuration file and check its validity.
    Check(CheckArgs),
    /// Generate a template configuration file.
    Generate(TemplateArgs),
}

#[derive(Debug, Args)]
pub struct CheckArgs {
    #[arg(short, long)]
    config_path: String,
}

#[derive(Debug, Args)]
pub struct TemplateArgs {
    /// The architecture of the VM, currently only support "riscv64", "aarch64" and "x86_64".
    #[arg(short = 'a', long)]
    arch: String,
    /// The ID of the VM.
    #[arg(short = 'i', long, default_value_t = 0)]
    id: usize,
    /// The name of the VM.
    #[arg(short = 'n', long, default_value_t = String::from("GuestVM"))]
    name: String,
    /// The type of the VM, 0 for HostVM, 1 for RTOS, 2 for Linux.
    #[arg(short = 't', long, default_value_t = 1)]
    vm_type: usize,
    /// The number of CPUs of the VM.
    #[arg(short = 'c', long, default_value_t = 1)]
    cpu_num: usize,
    /// The entry point of the VM.
    #[arg(short = 'e', long, default_value_t = 1)]
    entry_point: usize,
    /// The path of the kernel image, if the image_location is "fs", it should be the path of the kernel image file inside the ArceOS's rootfs.
    #[arg(short = 'k', long)]
    kernel_path: String,
    /// The load address of the kernel image.
    #[arg(short = 'l', long, value_parser = parse_usize)]
    kernel_load_addr: usize,
    /// The location of the kernel image：
    /// - "fs" for the kernel image file inside the ArceOS's rootfs
    /// - "memory" for the kernel image file in the memory.
    #[arg(long, default_value_t = String::from("fs"))]
    image_location: String,
    /// The command line of the kernel.
    #[arg(long)]
    cmdline: Option<String>,
    /// The output path of the template file.
    #[arg(short = 'O', long, value_name = "DIR", value_hint = clap::ValueHint::DirPath)]
    output: Option<std::path::PathBuf>,
}

/// Parse a single key-value pair
fn parse_usize(s: &str) -> Result<usize, Box<dyn Error + Send + Sync + 'static>> {
    if s.starts_with("0x") {
        Ok(usize::from_str_radix(&s[2..], 16)?)
    } else if s.starts_with("0b") {
        Ok(usize::from_str_radix(&s[2..], 2)?)
    } else {
        Ok(s.parse()?)
    }
}

pub fn run() {
    let cli = CLI::parse();
    match cli.subcmd {
        CLISubCmd::Check(args) => {
            let file_path = &args.config_path;
            // Check if the file exists.
            if !Path::new(file_path).exists() {
                eprintln!("Error: File '{}' does not exist.", file_path);
                std::process::exit(1);
            }
            let file_content = match fs::read_to_string(file_path) {
                Ok(content) => content,
                Err(err) => {
                    eprintln!("Error: Failed to read file '{}': {}", file_path, err);
                    std::process::exit(1);
                }
            };

            match AxVMCrateConfig::from_toml(&file_content) {
                Ok(config) => {
                    println!("Config file '{}' is valid.", file_path);
                    println!("Config: {:#x?}", config);
                }
                Err(err) => {
                    eprintln!("Error: Config file '{}' is invalid: {}", file_path, err);
                    std::process::exit(1);
                }
            }
        }
        CLISubCmd::Generate(args) => {
            let kernel_path = if args.image_location == "memory" {
                Path::new(&args.kernel_path)
                    .canonicalize()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string()
            } else {
                args.kernel_path.clone()
            };

            let template = get_vm_config_template(
                args.id,
                args.name + "-" + args.arch.as_str(),
                args.vm_type,
                args.cpu_num,
                args.entry_point,
                kernel_path,
                args.kernel_load_addr,
                args.image_location,
                args.cmdline,
            );

            let template_toml = toml::to_string(&template).unwrap();

            let target_path = match args.output {
                Some(relative_path) => relative_path,
                None => env::current_dir().unwrap().join("template.toml"),
            };

            match fs::write(&target_path, template_toml) {
                Ok(_) => {
                    println!("Template file '{:?}' has been generated.", target_path);
                }
                Err(err) => {
                    eprintln!(
                        "Error: Failed to write template file '{:?}': {}",
                        target_path, err
                    );
                    std::process::exit(1);
                }
            }
        }
    }
}
