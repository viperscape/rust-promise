#![feature(unsafe_destructor)]
#![feature(alloc)]

pub use latch::Latch;
pub use promise::Promise;

pub mod latch;
pub mod promise;
