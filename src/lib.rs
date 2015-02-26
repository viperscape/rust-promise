#![feature(unsafe_destructor)]
#![feature(alloc)]

pub use latch::Latch;
pub use promise::{Promise,Promisee,Promiser};
pub use fence::Fence;

pub mod latch;
pub mod promise;
pub mod fence;
