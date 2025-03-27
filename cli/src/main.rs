use clap::{Args, Parser, Subcommand, arg};
use hashassin_core::dump_hashes;
use hashassin_core::generate_hashes;
use hashassin_core::generate_passwords;

#[derive(Debug, Parser)]
/// Main structure for command-line argument parsing using Clap.
/// This struct is used to define the possible commands for the CLI, including `GenPasswords`, `GenHashes` and `DumpHashes`.
/// It parses the arguments based on user input and triggers the corresponding actions.
///
/// # Example:
///
/// ```sh
/// cargo run -- gen-passwords --chars 8 --threads 4 --num 1000
/// cargo run -- gen-hashes --in-file input.txt --out-file output.txt --threads 2 --algorithm sha256
/// ```
struct MyArgs {
    #[command(subcommand)]
    command: Commands,
}

fn main() {
    let args = MyArgs::parse();
    println!("{args:?}");
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
    }
}

/// Enum representing the available subcommands for the CLI.
/// This includes `GenPasswords` for password generation and `GenHashes` for generating hashes.
/// Each subcommand has its own set of arguments defined in separate structs
#[derive(Debug, Subcommand)]
enum Commands {
    // Clap crate maps GenPassword to actually gen-password args handling case and hyphenation automatically.
    GenPasswords(GenPasswordsArgs),
    GenHashes(GenHashesArgs),
    DumpHashes(DumpHashesArgs),
}

/// Arguments for the `gen-passwords` command.
/// These include options for password length, output file location, number of threads, and number of passwords to generate.
#[derive(Debug, Args)]
struct GenPasswordsArgs {
    /// Flag to read numbers to characters to be used in generated passwords
    #[arg(long, default_value_t = 4)]
    chars: u8,

    #[arg(long, default_value = "std")]
    out_file: String,

    #[arg(long, default_value_t = 1)]
    threads: usize,

    /// Number of passwords to generate
    #[arg(long, default_value_t = 1)]
    num: usize,
}

/// Arguments for the `gen-hashes` command.
/// These include options for the input file containing passwords, output file for hashes,
/// number of threads, and the hashing algorithm to use.
#[derive(Debug, Args)]
struct GenHashesArgs {
    /// Path to the file containing the plaintext passwords (one password per line)
    #[arg(long)]
    in_file: String,

    /// Path to the output file where hashes will be written
    #[arg(long, default_value = "std")]
    out_file: String,

    /// Number of threads to use for hash generation
    #[arg(long, default_value_t = 1)]
    threads: usize,

    /// Hashing algorithm to use (e.g., "sha256", "md5")
    #[arg(long, default_value = "sha256")]
    algorithm: String,
}

#[derive(Debug, Args)]
struct DumpHashesArgs {
    #[arg(long)]
    in_file: String,
}
