#![feature(unsafe_destructor)]
#![feature(alloc)]
#![feature(test)]

pub use latch::Latch;
pub use promise::{Promise,Promisee,Promiser};

pub mod latch;
pub mod promise;
