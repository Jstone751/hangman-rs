use std::path::PathBuf;
use clap::{Parser, Subcommand};
use chrono::Local;
use log::{debug, info, warn, error};
use fern::{Dispatch, colors::{Color, ColoredLevelConfig}};

#[derive(Parser, Clone)]
#[command(version, author, about)]
struct Cli {
    #[arg(short, long, default_value = "0")]
    debug: u8,

    #[command(subcommand)]
    subcommands: Commands,

    //TODO: Add a config file
}

#[derive(Subcommand, Debug, Clone)]
enum Commands {
    /// Guess a list of characters
    Guess {
        /// The list of characters to check
        #[arg(short, long)]
        guess: String,
    },
    /// Query to see if a character has already been guessed
    ///
    /// If no character is provided, the entire list of guessed
    /// characters will be returned with a status if it is correct
    Query {
        ///The list of characters to query
        #[arg(short, long)]
        check: Option<String>,
    },
    /// Start a new game
    ///
    /// Optionally, provide a filename to select a random word from
    /// If no filename is provided, a random word will be selected
    /// From a wordlist online
    New {
        #[arg(short, long, value_name = "OPTIONAL FILE")]
        file: Option<PathBuf>,
    },
    /// Save the current game from the program's internal file to a custom file
    Save {
        #[arg(short, long, value_name = "FILE")]
        file: PathBuf,
    },
    /// Load a game from a file to program's internal file
    Load {
        #[arg(short, long, value_name = "FILE")]
        file: PathBuf,
    },
    /// Show the current word with the guessed characters
    Show,
    /// Generate completion scripts for various shells
    Completions {
        #[arg(short, long, value_name = "DIRECTORY")]
        directory: Option<PathBuf>,
    },
}

fn handle_guess(guess: String) {
    println!("Guessing: {}", guess);
}

fn handle_query(check: Option<String>) {
    match check {
        Some(check) => println!("Checking: {}", check),
        None => println!("Querying all"),
    }
}

fn handle_new(file: Option<PathBuf>) {
    match file {
        Some(file) => println!("Starting new game with file: {:?}", file),
        None => println!("Starting new game with random word"),
    }
}

fn handle_save(file: PathBuf) {
    println!("Saving game to file: {:?}", file);
}

fn handle_load(file: PathBuf) {
    println!("Loading game from file: {:?}", file);
}

fn handle_show() {
    println!("Showing current game");
}

fn handle_completions(directory: Option<PathBuf>) {
    match directory {
        Some(directory) => println!("Generating completions for directory: {:?}", directory),
        None => println!("Generating completions for current directory"),
    }
}
fn init_logger(debug: u8) -> Result<(), fern::InitError> {
    let level = match debug {
        0 => log::LevelFilter::Error,
        1 => log::LevelFilter::Warn,
        2 => log::LevelFilter::Info,
        3 => log::LevelFilter::Debug,
        _ => log::LevelFilter::Error,
    };
    let colors = ColoredLevelConfig::new()
        .info(Color::Blue)
        .warn(Color::Yellow)
        .error(Color::Red)
        .debug(Color::Green)
        .trace(Color::Magenta);

    Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                colors.color(record.level()),
                record.target(),
                message
            ))
        })
        .level(level)
        .chain(std::io::stdout())
        .apply()?;
    Ok(())
}

fn main() {
    let cli = Cli::parse();
    
    // Initialize the logger
    if let Err(e) = init_logger(cli.debug) {
        eprintln!("Failed to initialize logger: {:?}", e);
        std::process::exit(1);
    }
    debug!("Successfully initialized logger");
    
    match cli.subcommands {
        Commands::Guess { guess } => { debug!("Running the handler for guess function"); handle_guess(guess); },
        Commands::Query { check } => { debug!("Running the handler for query function"); handle_query(check); },
        Commands::New { file } => { debug!("Running the handler for new function"); handle_new(file); },
        Commands::Save { file } => { debug!("Running the handler for save function"); handle_save(file); },
        Commands::Load { file } => { debug!("Running the handler for load function"); handle_load(file); },
        Commands::Show => { debug!("Running the handler for show function"); handle_show(); }
        Commands::Completions { directory } => { debug!("Running the handler for completions function"); handle_completions(directory); },
    }
}