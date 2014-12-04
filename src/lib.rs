#![feature(unsafe_destructor)]

extern crate alloc;
use alloc::arc::strong_count;

pub use latch::Latch;
pub use promise::Promise;

pub mod latch;
pub mod promise;


#[test]
fn it_works() {
}
