use std::{env, ffi::OsString, fs::File, io::{self, BufRead}, path::{Path, PathBuf}, process::{exit, Command}};
use rofi;
use serde::{Deserialize, Serialize};
use std::fs;
use bincode;
use std::io::{BufReader, BufWriter};
use glob::glob;
use diacritics::remove_diacritics;

use clap::Parser;

const EMOJI_FILES_DIR: &str = "./src/picker/data/";
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

    #[arg(long)]
    rescan: bool,
}

#[derive(Serialize, Deserialize, Debug)]
struct Emoji {
    origin_file: OsString,
    emoji_line: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Emojies {
    // map: IndexMap<OsString, Vec<String>>,
    items: Vec<Emoji>,
}

impl Emojies {

    /// Returns a list of emojies.
    ///
    /// - If the cache is already built, returns the content of the cache
    /// - Otherwse, fill cache and return emojies afterwards
    fn load() -> Self {
        println!("Loading from cache");

        let home_dir = env::var("HOME").expect("HOME variable seems to not be set");
        let cache_dir = PathBuf::from(format!("{}/{}", home_dir, CACHE_DIR));
        let cache_file_path = cache_dir.join("cache.bin");

        if !cache_dir.exists() {
            fs::create_dir_all(&cache_dir).expect("Could not create cache directory");
            println!("Created directory: {}", cache_dir.display());
        }

        let file = File::open(&cache_file_path).expect("Could not read cache file");
        let mut reader = BufReader::new(file);
        let mut emojies: Emojies = bincode::deserialize_from(&mut reader).expect("Could not deserialize cache. Try deleting ~/.cache/rustimoji/");

        if emojies.items.is_empty() {
            println!("Cache is empty");
            emojies.scan()
        }

        emojies
    }

    fn store_to_cache(&self) {
        let home_dir = env::var("HOME").expect("HOME variable seems to not be set");
        let cache_dir = PathBuf::from(format!("{}/{}", home_dir, CACHE_DIR));
        let cache_file_path = cache_dir.join("cache.bin");

        let file = File::create(&cache_file_path).expect("Could not create cache file");

        let mut writer = BufWriter::new(file);
        bincode::serialize_into(&mut writer, &self).unwrap();
    }

    /// Scan emoji directory and merge new items with already existing ones
    fn scan(&mut self) {
        println!("Scanning files");
        // read emoji csv files

        let path = Path::new(EMOJI_FILES_DIR);

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
                let emoji_line = remove_diacritics(&line); // remove diacritics: turn ń into n. Because rofi cant to that while matching, we do it here.
                let file_name: OsString = file_path.file_name().unwrap().to_os_string();

                if !self.contains(&emoji_line) {
                    self.push(file_name, emoji_line)
                }
            }
        }

        self.store_to_cache()
    }

    /// Returns all emojies from all files.
    fn all(&self) -> Vec<&String> {
        self.items.iter().map(|emoji| &emoji.emoji_line ).collect()
    }


    /// Returns all emojies that originate from files that contain any word in `keywords` as a substring.
    /// In other words, filter the file names by keywords. Matches, when any keyword matches.
    fn filtered(&self, keywords: Vec<String>) -> Vec<&String> {
        self.items
            .iter()
            .filter(|emoji| {
                let matched = keywords.iter().any(|keyword| emoji.origin_file.to_str().unwrap_or("").contains(keyword));
                // if matched { println!("Selected file {:?}", emoji.file); }
                matched

            })
            .map(|emoji| &emoji.emoji_line )
            .collect()
    }

    fn push(&mut self, origin_file: OsString, emoji_line: String) {
        self.items.push(Emoji{emoji_line, origin_file});
    }

    fn contains(&self, emoji: &String) -> bool {
        for item in self.items.iter() {
            if &item.emoji_line == emoji {
                return true
            }
        }

        false
    }

    fn move_element_to_front(&mut self, emoji_line: String) {
        if let Some(index) = self.items.iter().position(|x| x.emoji_line == emoji_line) {
            let item = self.items.remove(index);
            self.items.insert(0, item);
        }
    }

}

fn main() {

    let args = Cli::parse();

    let mut emojies = Emojies::load();

    if args.rescan {
        println!("Extra file scan requested");
        emojies.scan()
    }

    let filtered_emojies: Vec<&String> = if let Some(filter_keywords) = args.filter {
        println!("Applying filter");
        emojies.filtered(filter_keywords)
    } else {
        emojies.all()
    };

    println!("Showing rofi");
    let mut rofi_window = rofi::Rofi::new(&filtered_emojies);
    rofi_window.pango();
    rofi_window.prompt("😀");
    rofi_window.lines(ROFI_LINES);

    // println!("Starting window");
    match rofi_window.run() {
        Ok(choice) => {

            let (emoji, _) = choice.split_once(" ").expect("Could not extract emoji from selected line");

            println!("Choice: {}", emoji);

            clipboard(emoji);

            emojies.move_element_to_front(choice);

            emojies.store_to_cache();

        }
        Err(rofi::Error::Interrupted) => println!("Interrupted"),
        Err(e) => println!("Error: {}", e)
    }
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
