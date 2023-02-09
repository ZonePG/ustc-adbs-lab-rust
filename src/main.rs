use std::error::Error;
use std::io::{Write, BufReader, BufRead};
use std::{env, fs, process};
use crate::config::*;

mod config;
mod buffer_manager;
mod data_storage_manager;
mod page;

mod replacer;

fn main() {
    let run_time = std::time::Instant::now();
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
    println!("procee run time: {} ms", run_time.elapsed().as_millis());
}

fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let content = fs::read_to_string(&config.file_path)?;

    let mut buffer_manager = buffer_manager::BMgr::new(DB_FILE_NAME, config.policy, FRAME_NUM);
    let data_file = std::fs::File::open(&config.file_path).unwrap();
    let reader = BufReader::new(data_file);

    let trace_time = std::time::Instant::now();
    for line in reader.lines() {
        let line = line.unwrap();
        let parts: Vec<&str> = line.split(",").collect();
        
        let is_dirty = parts[0].parse::<u8>().unwrap() != 0;
        let page_id = parts[1].parse::<PageId>().unwrap() - 1;
        buffer_manager.fix_page(page_id, is_dirty);
        buffer_manager.unfix_page(page_id);
    }
    println!("read io: {}", buffer_manager.get_read_io_num());
    println!("write io: {}", buffer_manager.get_write_io_num());
    println!("total io: {}", buffer_manager.get_io_num());
    println!("hit number: {}", buffer_manager.get_hit_num());
    println!("hit rate: {}%", buffer_manager.get_hit_num() as f64 / content.lines().count() as f64 * 100.0);
    println!("trace time: {} ms", trace_time.elapsed().as_millis());

    Ok(())
}
