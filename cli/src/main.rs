#![deny(clippy::unwrap_used, clippy::expect_used)]
use clap::{Args, Parser, Subcommand};
use hashassin_client::handle_crack::handle_crack;
use hashassin_client::handle_upload::handle_upload;
use hashassin_core::crack::{crack_passwords, load_hashes, load_rainbow_table};
use hashassin_core::dump_hashes;
use hashassin_core::dump_rainbow_table;
use hashassin_core::generate_hashes;
use hashassin_core::generate_passwords;
use hashassin_core::generate_rainbow_table;
use hashassin_server::server;

#[derive(Debug, Parser)]
struct MyArgs {
    #[command(subcommand)]
    command: Commands,
}

fn main() {
    let args = MyArgs::parse();
    match args.command {
        Commands::GenPasswords(args) => {
            if let Err(e) = generate_passwords::generate_passwords(
                args.chars,
                args.out_file,
                args.threads,
                args.num,
            ) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
        Commands::GenHashes(args) => {
            if let Err(e) = generate_hashes::generate_hashes(
                args.in_file,
                args.out_file,
                args.threads,
                args.algorithm,
            ) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
        Commands::DumpHashes(args) => {
            if let Err(e) = dump_hashes::dump_hashes(&args.in_file) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
        Commands::GenRainbowTable(args) => {
            if let Err(e) = generate_rainbow_table::generate_rainbow_table(
                args.num_links,
                args.threads,
                args.out_file,
                args.algorithm,
                args.in_file,
            ) {
                eprintln!("Error generating rainbow table: {}", e);
                std::process::exit(1);
            }
        }
        Commands::DumpRainbowTable(args) => {
            if let Err(e) = dump_rainbow_table::dump_rainbow_table(&args.in_file) {
                eprintln!("Error dumping rainbow table: {}", e);
                std::process::exit(1);
            }
        }
        Commands::Crack(args) => {
            match load_rainbow_table(&args.in_file) {
                Ok(table) => match load_hashes(&args.hashes, &table.algorithm) {
                    Ok(hashes) => {
                        if let Err(e) = crack_passwords(
                            table,
                            hashes,
                            args.threads,
                            args.out_file.as_deref(), // pass Option<&str>
                        ) {
                            eprintln!("Error cracking passwords: {}", e);
                        }
                    }
                    Err(e) => {
                        eprintln!("Error loading hashes: {}", e);
                    }
                },
                Err(e) => {
                    eprintln!("Error loading rainbow table: {}", e);
                }
            }
        }
        Commands::Server(args) => {
            let async_threads = match args.async_threads {
                Some(n) if n > 0 => n,
                _ => 1,
            };
            let runtime = tokio::runtime::Builder::new_multi_thread()
                .worker_threads(async_threads)
                .enable_all()
                .build();

            match runtime {
                Ok(rt) => {
                    let result = rt.block_on(server::start_server(
                        args.bind,
                        args.port,
                        args.compute_threads,
                        args.cache_size,
                    ));

                    match result {
                        Ok(_) => println!("Server shut down gracefully."),
                        Err(e) => eprintln!("Server encountered an error: {}", e),
                    }
                }
                Err(e) => {
                    eprintln!("Failed to initialize Tokio runtime: {}", e);
                }
            }
        }
        Commands::Client(client_args) => {
            match client_args.command {
                ClientCommand::Upload(upload_args) => {
                    // Handle upload command
                    let result =
                        handle_upload(&upload_args.server, &upload_args.in_file, &upload_args.name);
                    if let Err(e) = result {
                        eprintln!("Error uploading rainbow table: {}", e);
                    }
                }
                ClientCommand::Crack(crack_client_args) => {
                    // Handle crack command
                    let result = handle_crack(
                        &crack_client_args.server,
                        &crack_client_args.in_file,
                        crack_client_args.out_file.as_deref(),
                    );
                    if let Err(e) = result {
                        eprintln!("Error cracking passwords: {}", e);
                    }
                }
            }
        }
    }
}

#[derive(Debug, Subcommand)]
enum Commands {
    GenPasswords(GenPasswordsArgs),
    GenHashes(GenHashesArgs),
    DumpHashes(DumpHashesArgs),
    GenRainbowTable(GenRainbowTableArgs),
    DumpRainbowTable(DumpRainbowTableArgs),
    Crack(CrackArgs),
    Server(ServerArgs),
    Client(ClientArgs),
}

#[derive(Debug, Args)]
struct GenPasswordsArgs {
    #[arg(long, default_value_t = 4)]
    chars: u8,
    #[arg(long, default_value = "std")]
    out_file: String,
    #[arg(long, default_value_t = 1)]
    threads: usize,
    #[arg(long, default_value_t = 1)]
    num: usize,
}

#[derive(Debug, Args)]
struct GenHashesArgs {
    #[arg(long)]
    in_file: String,
    #[arg(long, default_value = "std")]
    out_file: String,
    #[arg(long, default_value_t = 1)]
    threads: usize,
    #[arg(long, default_value = "sha256")]
    algorithm: String,
}

#[derive(Debug, Args)]
struct DumpHashesArgs {
    #[arg(long)]
    in_file: String,
}

#[derive(Debug, Args)]
struct GenRainbowTableArgs {
    #[arg(long, default_value_t = 5)]
    num_links: usize,
    #[arg(long, default_value_t = 1)]
    threads: usize,

    #[arg(long, required = true)]
    out_file: String,
    #[arg(long, default_value = "md5")]
    algorithm: String,

    #[arg(long, required = true)]
    in_file: String,
}

#[derive(Debug, Args)]
struct DumpRainbowTableArgs {
    #[arg(long, required = true)]
    in_file: String,
}

#[derive(Debug, Args)]
struct CrackArgs {
    #[arg(long, required = true)]
    in_file: String,

    #[arg(long)]
    hashes: String,

    #[arg(long)]
    out_file: Option<String>, // optional

    #[arg(long, default_value_t = 1)]
    threads: usize,
}

#[derive(Debug, Args)]
struct ServerArgs {
    #[arg(long, default_value_t = String::from("127.0.0.1"))]
    bind: String,

    #[arg(long, default_value_t = 2025)]
    port: u16,

    #[arg(long, default_value_t = 1)]
    compute_threads: usize,

    #[arg(long)]
    async_threads: Option<usize>,

    /// Optional cache size (max: i32::MAX bytes)
    #[arg(long, value_parser = cache_size_within_i32)]
    cache_size: Option<u32>,
}

fn cache_size_within_i32(val: &str) -> Result<u32, String> {
    match val.parse::<u64>() {
        Ok(v) if v <= i32::MAX as u64 => Ok(v as u32),
        Ok(_) => Err(format!("cache-size must be <= {} bytes", i32::MAX)),
        Err(e) => Err(format!("Invalid number: {}", e)),
    }
}

#[derive(Debug, Args)]
struct ClientArgs {
    #[command(subcommand)]
    command: ClientCommand,
}

#[derive(Debug, Subcommand)]
enum ClientCommand {
    /// Upload a rainbow table to the server
    Upload(UploadArgs),

    /// Request cracking of hashes file by server
    Crack(CrackClientArgs),
}

#[derive(Debug, Args)]
struct UploadArgs {
    #[arg(long)]
    server: String,

    #[arg(long, value_name = "FILE")]
    in_file: String,

    #[arg(long)]
    name: String,
}

#[derive(Debug, Args)]
struct CrackClientArgs {
    #[arg(long)]
    server: String,

    #[arg(long, value_name = "FILE")]
    in_file: String,

    #[arg(long, value_name = "FILE")]
    out_file: Option<String>,
}
