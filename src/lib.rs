
#![feature(alloc)]

#![feature(arc_counts)]
pub use latch::Latch;
pub use promise::{Promise,Promisee,Promiser};

pub mod latch;
pub mod promise;
