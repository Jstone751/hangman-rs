use chrono::Local;
use clap::{Parser, Subcommand};
use fern::{
    colors::{Color, ColoredLevelConfig},
    Dispatch,
};
use figment::value::{Dict, Map, Value};
use figment::{
    providers::{Format, Toml},
    Error, Figment, Profile, Provider,
};
use log::{debug, error, info};
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use std::io::Write;
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
    strikes: u8,
}

impl Default for Config {
    //noinspection SpellCheckingInspection
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
                PathBuf::from(format!(
                    "{}/.config/hangman_current_game.toml",
                    std::env::var("HOME").unwrap()
                )),
                PathBuf::from(format!(
                    "{}/.config/hangman.log",
                    std::env::var("HOME").unwrap()
                )),
            )
        };
        Config {
            wordlist: None,
            savefile: Some(savefile),
            logfile: Some(logfile),
            strikes: 8,
        }
    }
}

impl Provider for Config {
    fn metadata(&self) -> figment::Metadata {
        figment::Metadata::named("Default config")
    }

    fn data(
        &self,
    ) -> Result<Map<Profile, Dict>, Error> {
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
        let mut dict = Dict::new();
        dict.insert("wordlist".to_string(), Value::from(wordlist_conv));
        dict.insert("savefile".to_string(), Value::from(savefile_conv));
        dict.insert("logfile".to_string(), Value::from(logfile_conv));
        dict.insert("strikes".to_string(), Value::from(self.strikes));
        Ok(figment::value::Map::from([(
            Profile::Default,
            dict,
        )]))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Savefile {
    pub(crate) word: String,
    pub(crate) guessed: Vec<char>,
    pub(crate) correct: Vec<char>,
    pub(crate) incorrect: Vec<char>,
    pub(crate) strikes_left: u8,
}

impl Default for Savefile {
    fn default() -> Self {
        Self {
            word: "".to_string(),
            guessed: vec![],
            correct: vec![],
            incorrect: vec![],
            strikes_left: 8,
        }
    }
}

impl Provider for Savefile {
    fn metadata(&self) -> figment::Metadata {
        figment::Metadata::named("Default savefile")
    }

    fn data(&self) -> Result<Map<Profile, Dict>, Error> {
        let mut dict = Dict::new();
        dict.insert("word".to_string(), Value::from(self.word.clone()));
        dict.insert("guessed".to_string(), Value::from(self.guessed.clone()));
        dict.insert("correct".to_string(), Value::from(self.correct.clone()));
        dict.insert("incorrect".to_string(), Value::from(self.incorrect.clone()));
        dict.insert("strikes_left".to_string(), Value::from(self.strikes_left));
        Ok(figment::value::Map::from([(
            Profile::Default,
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

//noinspection SpellCheckingInspection
fn handle_new(file: Option<PathBuf>, savefile_path: PathBuf) {
    //noinspection SpellCheckingInspection
    let mut random_word: String;

    if let Some(file_path) = file {
        info!("Starting new game with wordfile: {:?}", file_path);
        if !file_path.exists() {
            error!("Wordlist file does not exist, exiting");
            std::process::exit(1);
        } else if file_path.is_dir() {
            error!("Given wordlist is a directory, exiting");
            std::process::exit(1);
        } else {
            let wordlist = std::fs::read_to_string(file_path)
                .unwrap()
                .lines()
                .map(|s| s.to_string())
                .collect::<Vec<String>>();
            random_word = wordlist[thread_rng().gen_range(0..wordlist.len())].to_string();
            debug!(
                "Successfully generated random word from file: {}",
                random_word
            );
        }
    } else {
        let api_response = reqwest::blocking::get("https://random-word-api.vercel.app/api?words=1")
            .expect("Failed to get random word from api!");
        random_word = api_response.text().unwrap();
        debug!(
            "Successfully generated random word from API: {}",
            random_word
        );
    }

    random_word = random_word
        .trim_matches(|x| x == '[' || x == ']' || x == '"')
        .parse()
        .unwrap();

    // Load the existing savefile
    let mut savefile: Savefile = Figment::new()
        .merge(Toml::file(&savefile_path))
        .extract()
        .expect("Failed to load savefile");

    // Update the word field
    savefile.word = random_word.clone();
    let mut file = std::fs::File::create(&savefile_path).expect("Failed to create new savefile");
    file.write_all(
        toml::to_string(&Savefile {
            word: random_word,
            guessed: vec![],
            correct: vec![],
            incorrect: vec![],
            strikes_left: 8,
        })
        .expect("Failed to serialize savefile")
        .as_bytes(),
    )
    .unwrap();
}

fn verify_toml_file(file: &PathBuf) -> bool {
    file.exists() && file.is_file() && file.extension() == Some("toml".as_ref())
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

    let savefile: PathBuf = figment
        .extract::<Config>()
        .expect("Failed to extract configuration")
        .savefile
        .unwrap_or(Config::default().savefile.unwrap());
    debug!("Current received savefile: {:?}", savefile);
    info!("Savefile does not exist, creating new savefile");
    if !savefile.exists() {
        info!("Savefile does not exist, creating new savefile");
        if let Some(parent) = savefile.parent() {
            std::fs::create_dir_all(parent).expect("Failed to create savefile directory");
        }
        let mut file = std::fs::File::create(&savefile).expect("Failed to create new savefile");
        file.write_all(
            toml::to_string(&Savefile::default())
                .expect("Failed to serialize savefile")
                .as_bytes(),
        )
        .expect("Failed to write savefile");
    }

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
            handle_new(file, savefile);
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
