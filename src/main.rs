use std::path::PathBuf;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, author, about)]
struct Cli {
    #[command(subcommand)]
    subcommands: Commands,

    /// Debug level
    #[arg(short, long, action = clap::ArgAction::SetTrue, default_value = "0")]
    debug: bool,
    
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
}

fn main() {
    let cli = Cli::parse();
}