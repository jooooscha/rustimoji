use std::{env, fs::File, io::{self, BufRead}, path::{Path, PathBuf}};
use rofi;
use std::fs;
use bincode;
use std::io::{BufReader, BufWriter};
use glob::glob;

use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)] // Optional metadata
struct Cli {
    /// The path to the file
    #[arg(long)]
    path_from: PathBuf,

    // TODO:
    // #[arg(long)]
    // invalidate: bool
}

fn main() {

    let args = Cli::parse();
    println!("args: {:?}", args.path_from);

    let (_, created) = get_cache_file_path().unwrap();
    // println!("cache_dir: {:?}", cache_dir);

    if created {
        // println!("Setting up cache");

        // let path = Path::new("/nix/store/2d2sqja29sf9zk2rrnn41hrq0i5zljly-rofimoji-6.3.1/lib/python3.11/site-packages/picker/data/emojis_smileys_emotion.csv");
        // let path = Path::new("/nix/store/2d2sqja29sf9zk2rrnn41hrq0i5zljly-rofimoji-6.3.1/lib/python3.11/site-packages/picker/data/");
        let path = Path::new(&args.path_from);

        let mut vec = Vec::new();

        for file in glob(path.join("kaomoji.csv").to_str().unwrap()).expect("Failed to read glob pattern") {

            let file = file.unwrap();

            let path = file.as_path();
            if !path.is_file() {
                continue
            }

            let file = File::open(&path).unwrap();

            let reader = io::BufReader::new(file);

            for line in reader.lines() {
                let line = line.unwrap();
                // let (first, second) = line.split_once(" ").unwrap();
                vec.push(line);
            }
        }

        write_cache(vec).unwrap();
    }

    let data = read_cache().unwrap();

    // println!("Creating window");
    let mut rofi_window = rofi::Rofi::new(&data);
    rofi_window.lines(10);

    // println!("Starting window");
    match rofi_window.run() {
        Ok(choice) => println!("Choice: {}", choice),
        Err(rofi::Error::Interrupted) => println!("Interrupted"),
        Err(e) => println!("Error: {}", e)
    }
}

fn get_cache_file_path() -> Result<(PathBuf, bool), std::io::Error> {
    // Get the home directory
    let home_dir = env::var("HOME").expect("Could not get home directory");

    // Define the path for the cache directory
    let cache_dir = PathBuf::from(format!("{}/.cache/rustimoji", home_dir));

    let mut created = false;

    // Create the directory if it doesn't exist
    if !cache_dir.exists() {
        created = true;
        fs::create_dir_all(&cache_dir)?;
        println!("Created directory: {}", cache_dir.display());
    }

    // Define the cache file path
    let cache_file_path = cache_dir.join("cache.bin");

    Ok((cache_file_path, created))
}


fn write_cache(data: Vec<String>) -> io::Result<()> {

    println!("Writing cache file");

    let (cache_file, _) = get_cache_file_path()?;

    // Write to file
    let file = File::create(cache_file)?;
    let mut writer = BufWriter::new(file);
    bincode::serialize_into(&mut writer, &data).unwrap();

    Ok(())
}

fn read_cache() -> io::Result<Vec<String>> {
    // println!("Reading cache");

    let (cache_file, _) = get_cache_file_path()?;

    // Read from file
    let file = File::open(cache_file)?;
    let mut reader = BufReader::new(file);
    let decoded: Vec<String> = bincode::deserialize_from(&mut reader).unwrap();

    Ok(decoded)
}

