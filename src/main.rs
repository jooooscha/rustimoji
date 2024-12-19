use std::{collections::HashMap, env, ffi::OsString, fs::File, io::{self, BufRead}, path::{Path, PathBuf}, process::exit};
use rofi;
use serde::{Deserialize, Serialize};
use std::fs;
use bincode;
use std::io::{BufReader, BufWriter};
use glob::glob;

use clap::Parser;

const EMOJI_FILES_DIR: &str = "/nix/store/2d2sqja29sf9zk2rrnn41hrq0i5zljly-rofimoji-6.3.1/lib/python3.11/site-packages/picker/data/";
const CACHE_DIR: &str = ".cache/rustimoji/";
const ROFI_LINES: usize = 10;

#[derive(Parser)]
#[command(author, version, about, long_about = None)] // Optional metadata
struct Cli {

    // TODO:
    // #[arg(long)]
    // invalidate: bool

    #[arg(long)]
    list: bool,

    #[arg(long)]
    filter: Option<Vec<String>>,
}

// #[derive(Serialize, Deserialize)]
// struct Emoji {
//     file: OsString,
//     emoji: String,
// }

#[derive(Serialize, Deserialize, Debug)]
struct Emojies {
    map: HashMap<OsString, Vec<String>>,
}

impl Emojies {

    fn new() -> Self {
        Self { map: HashMap::new() }
    }

    /// Returns all emojies from all files.
    fn all(&self) -> Vec<&String> {
        self.map.values().flatten().collect()
    }


    /// Returns all emojies that originate from files that contain any word in `keywords` as a substring.
    /// In other words, filter the file names by keywords. Matches, when any keyword matches.
    fn filtered(&self, keywords: Vec<String>) -> Vec<&String> {
        self.map
            .iter()
            .filter(|(file, _)| {
                let matched = keywords.iter().any(|keyword| file.to_str().unwrap_or("").contains(keyword));
                if matched { println!("Selected file {file:?}"); }
                matched

            })
            .flat_map(|(_, emojies)| emojies.iter())
            .collect()
    }

    fn push(&mut self, file: OsString, emoji: String) {
        self.map.entry(file) // Get the entry for the key
            .or_insert_with(Vec::new) // If the key doesn't exist, insert an empty Vec
            .push(emoji); // Add the value to the Vec
    }

}

fn main() {

    let args = Cli::parse();

    if args.list {
        println!("List of emoji options");
        let files = get_emoji_list();
        println!("{files:#?}");
        exit(0);
    }

    let data = emojies();

    let emojies: Vec<&String> = if let Some(filter_keywords) = args.filter {
        data.filtered(filter_keywords)
    } else {
        data.all()
    };

    let mut rofi_window = rofi::Rofi::new(&emojies);
    rofi_window.lines(ROFI_LINES);

    // println!("Starting window");
    match rofi_window.run() {
        Ok(choice) => println!("Choice: {}", choice),
        Err(rofi::Error::Interrupted) => println!("Interrupted"),
        Err(e) => println!("Error: {}", e)
    }
}


/// Returns a list of emojies.
///
/// - If the cache is already built, returns the content of the cache
/// - Otherwse, fill cache and return emojies afterwards
fn emojies() -> Emojies {
    let home_dir = env::var("HOME").expect("HOME variable seems to not be set");
    let cache_dir = PathBuf::from(format!("{}/{}", home_dir, CACHE_DIR));

    if !cache_dir.exists() {
        fs::create_dir_all(&cache_dir).expect("Could not create cache directory");
        println!("Created directory: {}", cache_dir.display());
    }

    let cache_file_path = cache_dir.join("cache.bin");

    // no cache exists, create one
    if !cache_file_path.exists() {
        println!("Creating cache");
        let file = File::create(&cache_file_path).expect("Could not create cache file");

        // read emoji csv files

        let path = Path::new(EMOJI_FILES_DIR);

        let mut emoji_map = Emojies::new();

        for file_path in glob(path.join("**/*.csv").to_str().unwrap()).expect("Failed to read glob pattern") {

            let file_path = file_path.expect("Could not read file matches by glob");

            let path = file_path.as_path();
            if !path.is_file() {
                continue
            }

            let file = File::open(&path).expect("Could not open globbed file");

            let reader = io::BufReader::new(file);

            for line in reader.lines() {
                let line = line.expect("Could not read globbed file");
                let file_name: OsString = file_path.file_name().unwrap().to_os_string();

                emoji_map.push(file_name, line);
            }
        }

        let mut writer = BufWriter::new(file);
        bincode::serialize_into(&mut writer, &emoji_map).unwrap();
    }

    println!("Reading cache");

    let file = File::open(&cache_file_path).expect("Could not read cache file");
    let mut reader = BufReader::new(file);
    let decoded: Emojies = bincode::deserialize_from(&mut reader).unwrap();

    // println!("decoded: {:#?}", decoded);

    decoded
}

fn get_emoji_list() -> Vec<OsString> {

    // TODO: path

    let path = "/nix/store/2d2sqja29sf9zk2rrnn41hrq0i5zljly-rofimoji-6.3.1/lib/python3.11/site-packages/picker/data/";
    let pattern = format!("{path}**/*.csv");

    let mut list: Vec<OsString> = Vec::new();

    if let Ok(paths) = glob(&pattern) {
        for file in paths {
            let file = file.expect("Cannot read file from path");
            let file = file.file_name();
            if let Some(filename) = file {
                list.push(filename.to_os_string());
            }
        }
    } else {
        println!("Error {:?}", glob(&pattern));
        panic!("Did not find any emoji files");
    };

    list
}
