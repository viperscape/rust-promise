extern crate alloc;
use alloc::arc::strong_count;

use std::sync::{Arc, Mutex, Condvar};
use std::sync::atomic::{Ordering};
use latch::Latch;
 

pub struct Promise<T> {
    pub data: Arc<(Mutex<Option<T>>, Condvar)>,
    pub latch: Latch,
}

impl<T: Sync+Send> Promise<T> {
    pub fn new () -> Promise<T> {
        Promise {data: Arc::new((Mutex::new(None), Condvar::new())), latch: Latch::new()}
    }
    pub fn deliver (&self, d:T) -> bool {
        if self.latch.close() {
            let &(ref lock, ref cond) = &*self.data; 
            let mut data = lock.lock();
            *data = Some(d);
            cond.notify_all(); //wake up others
           // data.downgrade();
            true
        }
        else {false}
    }
 
    //potentially blocking for other readers if fn applied is cpu/disk intensive
    pub fn apply (&self, f: |&T| -> T) -> Result<T,String> {
        let &(ref lock, ref cond) = &*self.data;
        if !self.latch.latched() { 
            if strong_count(&self.data) < 2 { return Err("safety hatch, promise not capable".to_string()) }
            let v = lock.lock(); //lock
            cond.wait(&v);
        }
        
        let v = lock.lock();
        match *v {
            Some(ref r) => Ok(f(r)),
            None => Err("promise signaled early, value not present!".to_string()),
        }
    }
 
    pub fn clone (&self) -> Promise<T> {
        Promise {data: self.data.clone(),
                 latch: self.latch.clone()}
    }
 
    pub fn destroy (&self) -> Result<String,String> {
        if self.latch.close() {
            let &(ref lock, ref cond) = &*self.data;
            let mut data = lock.lock();
            *data = None;
            cond.notify_all(); //wake up others
            //data.downgrade();
            Ok("Promise signaled early".to_string())
        }
        else { Err("promise already delivered".to_string()) }
    }
}

#[unsafe_destructor]
/// Special Drop for Promise
/// we don't want to hang readers on a local panic
impl<T: Sync+Send> Drop for Promise<T> {
    fn drop (&mut self) {
        if strong_count(&self.data) < 3 { self.destroy(); }
    }
}

#[cfg(test)]
mod tests {
    use Promise;
    
    #[test]
    fn test_promise_linear() {
        let p: Promise<int> = Promise::new();
        assert_eq!(p.deliver(1),true);
        assert_eq!(p.deliver(2),false);
        assert_eq!(p.apply(|x| *x).unwrap(),1);
    }

    #[test]
    fn test_promise_threaded() {
        let p: Promise<int> = Promise::new();
        let p2 = p.clone();
        spawn(proc() {
            assert_eq!(p2.deliver(1),true);
        });
        assert_eq!(p.apply(|x| *x).unwrap(),1); //waits on spawned thread
    }

    #[test]
    #[should_fail]
    fn test_promise_threaded_destroyed() {
        let p: Promise<int> = Promise::new();
        let p2 = p.clone();
        spawn(proc() {
            p2.destroy();
        });
        p.apply(|x| *x).unwrap();
    }

    #[test]
    #[should_fail]
    fn test_promise_threaded_panic_safely() {
        let p: Promise<int> = Promise::new();
        let p2 = p.clone();

        spawn (proc () {
            p2.latch.latched(); //moves p2 into proc
            panic!("proc dead"); //destroys promise, triggers wake on main proc
        });
        
        p.apply(|x| *x).unwrap();
    }
}
