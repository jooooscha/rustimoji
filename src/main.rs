use std::{env, ffi::OsString, fs::File, io::{self, BufRead}, path::{Path, PathBuf}, process::{exit, Command}};
use rofi;
use serde::{Deserialize, Serialize};
use std::fs;
use bincode;
use std::io::{BufReader, BufWriter};
use glob::glob;
use diacritics::remove_diacritics;

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

#[derive(Serialize, Deserialize, Debug)]
struct Emoji {
    file: OsString,
    emoji: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Emojies {
    // map: IndexMap<OsString, Vec<String>>,
    list: Vec<Emoji>,
}

impl Emojies {

    fn new() -> Self {
        Self { list: Vec::new() }
    }

    fn load_from_cache() -> Self {
        emojies()
    }

    fn store_to_cache(&self) {
        let home_dir = env::var("HOME").expect("HOME variable seems to not be set");
        let cache_dir = PathBuf::from(format!("{}/{}", home_dir, CACHE_DIR));

        if !cache_dir.exists() {
            fs::create_dir_all(&cache_dir).expect("Could not create cache directory");
            println!("Created directory: {}", cache_dir.display());
        }

        let cache_file_path = cache_dir.join("cache.bin");
        let file = File::create(&cache_file_path).expect("Could not create cache file");

        let mut writer = BufWriter::new(file);
        bincode::serialize_into(&mut writer, &self).unwrap();
    }

    /// Returns all emojies from all files.
    fn all(&self) -> Vec<&String> {
        self.list.iter().map(|emoji| &emoji.emoji ).collect()
    }


    /// Returns all emojies that originate from files that contain any word in `keywords` as a substring.
    /// In other words, filter the file names by keywords. Matches, when any keyword matches.
    fn filtered(&self, keywords: Vec<String>) -> Vec<&String> {
        self.list
            .iter()
            .filter(|emoji| {
                let matched = keywords.iter().any(|keyword| emoji.file.to_str().unwrap_or("").contains(keyword));
                // if matched { println!("Selected file {:?}", emoji.file); }
                matched

            })
            .map(|emoji| &emoji.emoji )
            .collect()
    }

    fn push(&mut self, file: OsString, emoji: String) {
        // self.list.entry(file) // Get the entry for the key
        //     .or_insert_with(Vec::new) // If the key doesn't exist, insert an empty Vec
        //     .push(emoji); // Add the value to the Vec
        self.list.push(Emoji{emoji, file});
    }

    fn move_element_to_front(&mut self, emoji: String) {
        if let Some(index) = self.list.iter().position(|x| x.emoji == emoji) {
            let item = self.list.remove(index);
            self.list.insert(0, item);
        }
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

    let mut data = emojies();

    let emojies: Vec<&String> = if let Some(filter_keywords) = args.filter {
        data.filtered(filter_keywords)
    } else {
        data.all()
    };

    let mut rofi_window = rofi::Rofi::new(&emojies);
    rofi_window.pango();
    rofi_window.prompt("😀");
    rofi_window.lines(ROFI_LINES);

    // println!("Starting window");
    match rofi_window.run() {
        Ok(choice) => {

            let (emoji, _) = choice.split_once(" ").expect("Could not extract emoji from selected line");

            println!("Choice: {}", emoji);

            clipboard(emoji);

            data.move_element_to_front(choice);

            data.store_to_cache();

        }
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
                let line = remove_diacritics(&line); // remove diacritics: turn ń into n. Because rofi cant to that while matching, we do it here.
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
    let decoded: Emojies = bincode::deserialize_from(&mut reader).expect("Could not deserialize cache. Try deleting ~/.cache/rustimoji/");

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

/// Copy `text` to clipboard
fn clipboard(text: &str) {
    let mut child = Command::new("xclip")
        .arg("-selection")
        .arg("clipboard")
        .stdin(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to spawn xclip process");

    if let Some(stdin) = &mut child.stdin {
        use std::io::Write;
        stdin.write_all(text.as_bytes()).expect("Failed to write to xclip");
    }

    child.wait().expect("Failed to wait for xclip process");
}
