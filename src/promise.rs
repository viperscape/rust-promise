use std::sync::{Arc, RWLock};

use latch::Latch;
 
pub struct Promise<T> {
    pub data: Arc<RWLock<Option<T>>>,
    pub latch: Latch,
}

impl<T: Sync+Send> Promise<T> {
    pub fn new () -> Promise<T> {
        Promise {data: Arc::new(RWLock::new(None)), latch: Latch::new()}
    }
    pub fn deliver (&self, d:T) -> bool {
        if self.latch.close() {
            let mut data = self.data.write();
            *data = Some(d);
            data.cond.broadcast(); //wake up others
            true
        }
        else {false}
    }
 
    pub fn apply (&self, f: |&T| -> T) -> Result<T,String> {
        if !self.latch.latched() { 
            let vw = self.data.write();
            vw.cond.wait();
            vw.downgrade(); //do this?
        }
        
        let v = self.data.read();
        match *v {
            Some(ref r) => Ok(f(r)),
            None => Err("promise signaled early, value not present!".to_string()),
        }
    }
 
    pub fn clone (&self) -> Promise<T> {
        Promise {data: self.data.clone(),
                 latch: self.latch.clone()}
    }
 
    pub fn destroy (self) -> Result<String,String> { //promise is moved
        if self.latch.close() {
            let mut data = self.data.write();
            *data = None;
            data.cond.broadcast(); //wake up others
            Ok("Promise signaled early".to_string())
        }
        else { Err("promise already delivered".to_string()) }
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
}
