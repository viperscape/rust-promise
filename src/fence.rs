use super::{Promise,Promisee};
use std::thread::{Thread};

pub struct Fence;

impl Fence {
    pub fn new(n:u16) {
        let (it,ir) = Promise::new(); //init fence
        let (ft,fr) = Promise::new(); //commit fence

        Thread::spawn (move || {
            Fence::await(&ir); //wait on init trigger
            ft.wakeup(); //wake others now, fence down
        });

        for n in (0..(n-1)) {
            let _fr = fr.clone();
            Thread::spawn (move || {
                Fence::await(&_fr); //wait on commit trigger
            });
        }

        Thread::spawn (move || {
            it.wakeup(); //wakeup fence, trigger it all
            Fence::await(&fr);
        });
    }

    pub fn await(p:&Promisee<()>) {
        p.with(|_| {()} );
    }
}



#[cfg(test)]
mod tests {
    extern crate test;
    use Fence;
    use std::thread::Thread;

    #[test]
    fn test_fence() {
        Fence::new(5);
        assert!(true); //completed!
    }
}
