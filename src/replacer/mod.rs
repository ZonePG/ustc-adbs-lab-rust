mod replacer;
mod lru_replacer;
mod clock_replacer;

pub use {
    replacer::Replacer,
    lru_replacer::LruReplacer,
    clock_replacer::ClockReplacer,
};