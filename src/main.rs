use clap::{Parser, Subcommand};

#[derive(Parser, Debug, Default)]
#[command(version, about, author)]
struct Cli {
    /// Name of the user
    name: Option<String>,

    #[command(subcommand)]
    command: Option<Commands>,

    /// Debug level
    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,
}

#[derive(Subcommand, Debug)]
enum Commands {
    // Does testing
    Test {
        // Test item
        #[arg(short, long)]
        list: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    if let Some(name) = cli.name.as_deref() {
        println!("Value for name: {}", name);
    }

    match cli.debug {
        0 => println!("Debug is off"),
        1 => println!("Debug is on"),
        2 => println!("Debug is very on"),
        _ => println!("Don't be silly!"),
    }

    match &cli.command {
        Some(Commands::Test { list }) => {
            if *list {
                println!("Printing list");
            } else {
                println!("Not printing list");
            }
        }
        None => {}
    }
}
