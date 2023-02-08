use std::error::Error;
use std::io::Write;
use std::{env, fs, process};
use crate::config::*;

mod config;
mod buffer_manager;
mod data_storage_manager;
mod page;

mod replacer;

fn main() {
    let args: Vec<String> = env::args().collect();

    let config = Config::build(&args).unwrap_or_else(|err| {
        println!("{}", err);
        process::exit(1);
    });

    {
        // write 50000 * PAGE_SIZE bytes to file
        let mut file = std::fs::File::create(DB_FILE_NAME).unwrap();
        let buf = vec![0 as u8; 50000 * PAGE_SIZE];
        file.write_all(&buf).unwrap();
    }

    if let Err(e) = run(config) {
        println!("Application error: {}", e);
        process::exit(1);
    }
}

fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let content = fs::read_to_string(config.file_path)?;

    println!("file content: {}", content);
    
    Ok(())
}
