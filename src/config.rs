pub const FRAME_NUM: usize = 1024;
pub const PAGE_SIZE: usize = 4096;
pub const DB_FILE_NAME : &str = "./target/data.dbf";

pub type FrameId = usize;
pub type PageId = usize;
pub type Data = [u8; PAGE_SIZE];

pub enum ReplacePolicy {
    Lru,
    Clock,
}

pub struct Config {
    pub policy: ReplacePolicy,
    pub file_path: String,
}

impl Config {
    pub fn build(args: &[String]) -> Result<Config, &'static str> {
        if args.len() != 3 {
            return Err("Usage: cargo run --release -- [lru|clock] [file_path]");
        }

        let policy = match args[1].as_str() {
            "lru" => ReplacePolicy::Lru,
            "clock" => ReplacePolicy::Clock,
            _ => {
                return Err("Usage: cargo run --release -- [lru|clock] [file_path]");
            }
        };
        let file_path = args[2].clone();

        Ok(Config { policy, file_path })
    }
}