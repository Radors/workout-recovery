use anyhow::{Context, Result, anyhow};
use chrono::{DateTime, Utc};
use clap::{
    Arg, ArgAction, ArgMatches, Command, builder::NonEmptyStringValueParser, command, value_parser,
};
use directories::ProjectDirs;
use rand::{Rng, distr::Alphanumeric};
use serde::{Deserialize, Serialize};
use serde_json;
use std::{
    cmp,
    fmt::{self, Display, Formatter},
    fs::{self, File},
    path::{Path, PathBuf},
    time::SystemTime,
};

struct Config {
    storage_path: PathBuf,
}

impl Config {
    fn setup() -> Result<Self> {
        let dirs = ProjectDirs::from("", "", "workout-recovery-data")
            .context("Unable to determine path for local storage")?;
        let storage_directory = Path::new(dirs.config_dir());
        fs::create_dir_all(storage_directory)
            .context("Failed to create directories for local storage")?;
        let storage_path = storage_directory.join("workout-recovery.json");
        if !Path::exists(&storage_path) {
            let default_data = Storage {
                sessions: Vec::new(),
            };
            let new_file = File::create(&storage_path)
                .context("Unable to create a new file for local storage")?;
            serde_json::to_writer(new_file, &default_data)
                .context("Failed writing to newly created local file")?;
        };
        Ok(Config { storage_path })
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct Session {
    identifier: String,
    description: String,
    timestamp: SystemTime,
}

impl Display for Session {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let timestamp: DateTime<Utc> = self.timestamp.into();
        let duration = Utc::now() - timestamp;
        let delta_days = duration.num_days();
        let delta_hours = duration.num_hours() % 24;
        let delta_minutes = duration.num_minutes() % 60;
        writeln!(f, "{:>15} {}", "[Identifier]", self.identifier)?;
        writeln!(f, "{:>15} {}", "[Description]", self.description)?;
        writeln!(
            f,
            "{:>15} Days: {} | Hours: {} | Minutes: {}",
            "[Time Elapsed]", delta_days, delta_hours, delta_minutes,
        )
    }
}

#[derive(Serialize, Deserialize)]
struct Storage {
    sessions: Vec<Session>,
}

impl Storage {
    fn read(config: &Config) -> Result<Self> {
        let file =
            File::open(&config.storage_path).context("Unable to read existing local file")?;
        let storage: Storage = serde_json::from_reader(file)
            .context("Failed converting local file contents into json")?;
        Ok(storage)
    }
    fn save(&self, config: &Config) -> Result<()> {
        let file = File::create(&config.storage_path)
            .context("Unable to open existing storage file when saving")?;
        serde_json::to_writer(file, &self)
            .context("Failed writing to existing storage file when saving")?;
        Ok(())
    }
    fn add(&mut self, description: &str) {
        let identifier = new_id(self);
        println!("Adding new workout session with identifier {identifier} ...");
        self.sessions.push(Session {
            identifier: identifier,
            description: description.to_owned(),
            timestamp: SystemTime::now(),
        });
        println!("Successfully added new workout session");
    }
    fn remove(&mut self, identifier: &str) -> Result<()> {
        let Some(index) = self
            .sessions
            .iter()
            .position(|s| s.identifier == identifier)
        else {
            return Err(anyhow!(
                "Identifier {identifier} was not found. Review identifiers with `list` command."
            ));
        };
        self.sessions.remove(index);
        println!("Successfully removed previous workout session with identifier {identifier}");
        Ok(())
    }
}

fn main() -> Result<()> {
    let config = Config::setup()?;
    let mut storage = Storage::read(&config)?;

    let add_cmd = Command::new("add").about("Add a new workout session").arg(
        Arg::new("description")
            .help("A short description of the workout session")
            .value_parser(NonEmptyStringValueParser::new())
            .required(true),
    );

    let remove_cmd = Command::new("remove")
        .about("Remove a previous workout session")
        .arg(
            Arg::new("identifier")
                .help("Identifier of the session to remove")
                .value_parser(NonEmptyStringValueParser::new())
                .required(true),
        );

    let list_cmd = Command::new("list")
        .about("List all recent workout sessions in order")
        .arg(
            Arg::new("number")
                .short('n')
                .long("number")
                .action(ArgAction::Set)
                .value_parser(value_parser!(usize))
                .help("Number of sessions to display"),
        );

    let root_cmd = command!()
        .subcommands([add_cmd, remove_cmd, list_cmd])
        .arg_required_else_help(true);

    let matches = root_cmd.get_matches();
    match matches.subcommand() {
        Some(("add", submatches)) => add(submatches, &mut storage),
        Some(("remove", submatches)) => remove(submatches, &mut storage)?,
        Some(("list", submatches)) => list(submatches, &storage),
        _ => unreachable!("should exhaustively check every parsed subcommand"),
    };

    storage.save(&config)?;
    Ok(())
}

fn new_id(storage: &Storage) -> String {
    loop {
        let new_id = generate_one_id();
        if !storage.sessions.iter().any(|s| s.identifier == new_id) {
            break new_id;
        }
    }
}

fn generate_one_id() -> String {
    rand::rng()
        .sample_iter(&Alphanumeric)
        .take(4)
        .map(char::from)
        .collect()
}

fn add(submatches: &ArgMatches, storage: &mut Storage) {
    let description = submatches
        .get_one::<String>("description")
        .expect("description should be parsed to be a valid string");
    storage.add(description);
}

fn remove(submatches: &ArgMatches, storage: &mut Storage) -> Result<()> {
    let identifier = submatches
        .get_one::<String>("identifier")
        .expect("identifier should be parsed to be a valid string");
    storage.remove(identifier)?;
    Ok(())
}

fn list(submatches: &ArgMatches, storage: &Storage) {
    if let Some(num) = submatches.get_one::<usize>("number") {
        output_list(*num, storage);
    } else {
        output_list(10, storage);
    }
}

fn output_list(count_requested: usize, storage: &Storage) {
    let mut count = cmp::min(count_requested, storage.sessions.len());
    let selected_sessions = storage.sessions.iter().rev().take(count).rev();
    for session in selected_sessions {
        if count != 1 {
            println!("{session}")
        } else {
            print!("{session}")
        }
        count -= 1;
    }
}
