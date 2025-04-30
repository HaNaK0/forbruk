use std::{
    fs::{self, File, OpenOptions},
    io::{BufReader, Write},
    path::{Path, PathBuf},
};

use anyhow::{Context, anyhow};
use chrono::{Local, NaiveDateTime};
use clap::{Args, Parser, Subcommand, ValueEnum};
use ron::ser::PrettyConfig;
use serde::{Deserialize, Serialize};

/// A simple program to track how much stuff is used
#[derive(Parser, Debug)]
#[command(version, about)]
struct Cli {
    #[command(subcommand)]
    sub_command: Commands,
}

/// Subcommands for the application
#[derive(Subcommand, Debug)]
enum Commands {
    /// Add an item that has been opened or a new Thermos
    Add {
        /// The item type to add of
        item: InventoryItem,
        /// The ammount of the item to add
        #[arg(default_value_t = 1)]
        ammount: i8,
        /// A time, will default to current time
        #[arg(long, short)]
        time: Option<chrono::NaiveTime>,
        /// A date, will default to current date
        #[arg(long, short)]
        date: Option<chrono::NaiveDate>,
    },
    /// Set the settings stored in settings.ron
    Set(Settings),
}

/// An enum with the different values that can be added
#[derive(Debug, ValueEnum, Clone, Copy)]
enum InventoryItem {
    Milk,
    Coffe,
    Mugs,
    Sugar,
    Sticks,
    Thermos,
}

#[derive(Args, Clone, Debug, Deserialize, Serialize)]
struct Settings {
    /// Set the current boat
    #[arg(short, long)]
    boat: Option<String>,
}

fn main() {
    let args = Cli::parse();
    
    let result = match &args.sub_command {
        Commands::Add { item, ammount, date, time } => add(item, ammount, time, date),
        Commands::Set(settings) => set(settings),
    };

    if let Err(err) = result {
        println!("Failed to {:?}: {}", args.sub_command, err)
    }
}

fn load_settings() -> anyhow::Result<Settings> {
    let path = Path::new("settings.ron");
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    Ok(ron::de::from_reader(reader)?)
}

fn add(item: &InventoryItem, ammount: &i8, time: &Option<chrono::NaiveTime>, date: &Option<chrono::NaiveDate>) -> anyhow::Result<()> {
    let settings = load_settings()?;
    let mut path = PathBuf::new();
    path.push("data");
    if !path.exists() {
        fs::create_dir_all(&path)?;
    }

    path.push(
        settings
            .boat
            .ok_or(anyhow!("No boat is set!, use set -b to set a boat"))?,
    );
    path.set_extension("csv");
    let mut f = OpenOptions::new()
        .append(true)
        .create(true)
        .open(&path)
        .with_context(|| format!("failed to open {:?}", &path))?;

    let time = time.unwrap_or_else(|| Local::now().naive_local().time());
    let date = date.unwrap_or_else(|| Local::now().naive_local().date());
    let date_time = NaiveDateTime::new(date, time);

    writeln!(f, "{},{:?},{}", date_time.and_local_timezone(Local).unwrap(), item, ammount)?;
    Ok(())
}

fn set(settings: &Settings) -> anyhow::Result<()> {
    let path = Path::new("settings.ron");

    let settings: Settings = if path.exists() {
        let read_settings: Settings = load_settings()?;

        Settings {
            boat: if let Some(boat) = &settings.boat {
                Some(boat.clone())
            } else {
                read_settings.boat
            },
        }
    } else {
        settings.clone()
    };

    let result = ron::ser::to_string_pretty(&settings, PrettyConfig::default())?;

    File::create(path)?.write_all(result.as_bytes())?;
    Ok(())
}
