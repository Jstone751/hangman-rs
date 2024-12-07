use chrono::Local;
use clap::{Parser, Subcommand};
use fern::{
    colors::{Color, ColoredLevelConfig},
    Dispatch,
};
use figment::value::Value;
use figment::{
    providers::{Format, Toml},
    Figment, Provider,
};
use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Parser, Clone)]
#[command(version, author, about)]
struct Cli {
    /// The debug level to use, default is 0, meaning errors only. Max is 3
    #[arg(short, long, default_value = "0")]
    debug: u8,

    /// The configuration file to use, if any. Uses HANGMAN_CONFIG by default, if not provided uses ~/.config/hangman.toml
    #[arg(short, long)]
    config: Option<PathBuf>,

    #[command(subcommand)]
    subcommands: Commands,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Config {
    wordlist: Option<PathBuf>,
    savefile: Option<PathBuf>,
    logfile: Option<PathBuf>,
    debug: u8,
}

impl Default for Config {
    fn default() -> Self {
        let (savefile, logfile) = if cfg!(windows) {
            (
                PathBuf::from(format!(
                    r#"{}\.hangman-internal-savefile.toml"#,
                    std::env::var("HOMEPATH").unwrap()
                )),
                PathBuf::from(format!(
                    r#"{}\.hangman.log"#,
                    std::env::var("HOMEPATH").unwrap()
                )),
            )
        } else {
            (
                PathBuf::from("~/.config/hangman_current_game.toml"),
                PathBuf::from("~/.config/hangman.log"),
            )
        };
        Config {
            wordlist: None,
            savefile: Some(savefile),
            logfile: Some(logfile),
            debug: 0,
        }
    }
}

impl Provider for Config {
    fn metadata(&self) -> figment::Metadata {
        figment::Metadata::named("Default config")
    }

    fn data(
        &self,
    ) -> Result<figment::value::Map<figment::Profile, figment::value::Dict>, figment::Error> {
        let wordlist_conv = match &self.wordlist {
            None => "None",
            Some(pathbuf) => pathbuf.to_str().unwrap(),
        };
        let savefile_conv = match &self.savefile {
            None => "None",
            Some(pathbuf) => pathbuf.to_str().unwrap(),
        };
        let logfile_conv = match &self.logfile {
            None => "None",
            Some(pathbuf) => pathbuf.to_str().unwrap(),
        };
        let mut dict = figment::value::Dict::new();
        dict.insert("wordlist".to_string(), Value::from(wordlist_conv));
        dict.insert("savefile".to_string(), Value::from(savefile_conv));
        dict.insert("logfile".to_string(), Value::from(logfile_conv));
        dict.insert("debug".to_string(), Value::from(self.debug));
        Ok(figment::value::Map::from([(
            figment::Profile::Default,
            dict,
        )]))
    }
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

fn verify_toml_file(file: &PathBuf) -> bool {
    file.is_file() && file.exists() && file.extension() == Some("toml".as_ref())
}

fn main() {
    let cli = Cli::parse();

    // Load configuration file if provided
    let mut using_default_config = false;
    let mut figment: Figment = Figment::new().merge(Config::default());
    if let Some(config) = cli.config {
        // Handle the configuration file
        debug!("Loading configuration file: {:?}", config);
        if verify_toml_file(&config) {
            info!(
                "Provided configuration file, {} is a valid TOML file",
                config.to_str().unwrap().to_string()
            );
            figment = Figment::new().merge(Toml::file(config));
        } else {
            error!(
                "Configuration file provided is not a valid TOML file, trying HANGMAN_CONFIG next"
            );
        }
    } else {
        let env_config = std::env::var("HANGMAN_CONFIG");
        match env_config {
            Ok(file) => {
                info!("HANGMAN_CONFIG, {} is set. Validating...", &file);
                let path = PathBuf::from(file.clone());
                if verify_toml_file(&path) {
                    info!("HANGMAN_CONFIG, {} is a valid TOML file", &file);
                    figment = Figment::new().merge(Toml::file(path));
                } else {
                    error!("HANGMAN_CONFIG, {} is not a valid TOML file", file);
                    error!("Tip! If not using HANGMAN_CONFIG, unset the variable using your shell's `unset` function");
                    debug!("Using default configuration");
                    using_default_config = true;
                }
            }
            Err(err) => {
                info!("HANGMAN_CONFIG is not set. Using default configuration file");
                debug!("For debug purposes, the OS provided error is: {:?}", err);
                using_default_config = true;
            }
        }
    }
    if using_default_config {
        info!("Loading default internal configuration");
    }

    // Extract the debug level from the configuration
    let config: Config = figment.extract().expect("Failed to extract configuration");
    let debug_level = if cli.debug == 0 { config.debug } else { cli.debug };

    // Initialize the logger
    if let Err(e) = init_logger(debug_level) {
        eprintln!("Failed to initialize logger: {:?}", e);
        std::process::exit(1);
    }
    debug!("Successfully initialized logger");

    match cli.subcommands {
        Commands::Guess { guess } => {
            debug!("Running the handler for guess function");
            handle_guess(guess);
        }
        Commands::Query { check } => {
            debug!("Running the handler for query function");
            handle_query(check);
        }
        Commands::New { file } => {
            debug!("Running the handler for new function");
            handle_new(file);
        }
        Commands::Save { file } => {
            debug!("Running the handler for save function");
            handle_save(file);
        }
        Commands::Load { file } => {
            debug!("Running the handler for load function");
            handle_load(file);
        }
        Commands::Show => {
            debug!("Running the handler for show function");
            handle_show();
        }
        Commands::Completions { directory } => {
            debug!("Running the handler for completions function");
            handle_completions(directory);
        }
    }
}