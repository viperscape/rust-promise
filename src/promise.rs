extern crate alloc;

use self::alloc::arc::strong_count;
use std::sync::{Arc};
//use std::sync::atomic::{AtomicPtr,Ordering};
use latch::Latch;
use std::sync::mpsc::{channel,Sender,Receiver};
use std::thread::{Thread};
use std::thread;
use std::cell::{UnsafeCell};

#[derive(Clone)]
pub struct Promise<T> {
    pub data: Arc<UnsafeCell<Option<T>>>,
    pub init: Latch,
    pub commit: Latch,
}

#[derive(Clone)]
pub struct Promisee<T> {
    pub p: Promise<T>,
    sink: Sender<Thread>,
}

pub struct Promiser<T> {
    p: Promise<T>,
    sink: Receiver<Thread>,
}

unsafe impl<T: Send> Send for Promise<T> {}
unsafe impl<T: Sync> Sync for Promise<T> {}
impl<T: Send+'static> Promise<T> {
    pub fn new () -> (Promiser<T>,Promisee<T>) {
        let (t,r) = channel();
        let d: Option<T> = None;

        let p = Promise { data: Arc::new(UnsafeCell::new(d)),
                          init: Latch::new(),
                          commit: Latch::new()};

        let p2 = p.clone();
        let pt = Promiser { p: p,
                            sink: r };
        let pr = Promisee { p: p2,
                            sink: t };

        (pt,pr)
    }

    pub fn clone (&self) -> Promise<T> {
        Promise { data: self.data.clone(),
                  init: self.init.clone(),
                  commit: self.commit.clone(),}
    }

    fn _deliver (&self, d:Option<T>) -> bool {
        if self.init.close() {
            let w = self.data.get();
            unsafe{ *w = d; }
            self.commit.close();
            return true
        }
        
        return false
    }

    pub fn deliver (&self, d:T) -> bool {
        self._deliver(Some(d))
    }

    ///should be called only from promiser/promisee-- public for now tho
    pub fn _with<W,F:FnMut(&T)->W> (&self, mut f:F) -> Result<W,String> {
        let v = self.data.get();

        unsafe {
            match *v {
                Some(ref r) => Ok(f(&*r)),
                None => Err("promise signaled early, value not present!".to_string()),
            }
        }
    }

 
    pub fn destroy (&self) -> Result<String,String> {
        if self._deliver(None) {
            Ok("Promise signaled early".to_string())
        }
        else { Err("promise already delivered".to_string()) }
    }
}

#[unsafe_destructor]
/// Special Drop for Promise
/// we don't want to hang readers on a local panic
impl<T: Sync+Send+'static> Drop for Promise<T> {
    fn drop (&mut self) {
        if strong_count(&self.data) < 3 { self.destroy(); }
    }
}


impl<T: Send+'static> Promiser<T> {
    pub fn deliver (&self, d:T) -> bool {
        let r  = self.p.deliver(d);

        self.wakeup();

        r
    }

    /// only call manually if you intend to destroy the promise
    pub fn wakeup (&self) {
        //let's wake everyone up!
        let mut s = self.sink.try_recv();
        while s.is_ok() {
            s.unwrap().unpark();
            s = self.sink.try_recv();
        }
    }
}

#[unsafe_destructor]
impl<T: Send+'static> Drop for Promiser<T> {
    fn drop (&mut self) {
        self.p.destroy();

        self.wakeup();
    }
}


impl<T: Send+'static> Promisee<T> {
    pub fn with<W,F:FnMut(&T)->W> (&self,f:F) -> Result<W,String> {
        if !self.p.commit.latched() { //not finalized?
            if !self.p.init.latched() { //has it been locked?
                if strong_count(&self.p.data) < 2 { 
                    return Err("safety hatch, promise not capable".to_string());
                }

                //todo: consider removing below ifstatement, atomicbool should take care of above logic
                //might need to change latch to seqcst tho
                self.sink.send(thread::current()); //signal promiser
                if !self.p.commit.latched() { //check again!
                    thread::park();
                }
            }
        }

        self.p._with(f)
    }

    pub fn get(&self) -> &Option<T> {
        let d = self.p.data.get();
        unsafe { &*d }
    }

    pub fn clone(&self) -> Promisee<T> {
        Promisee { p: self.p.clone(),
                   sink: self.sink.clone(), }
    }
}


#[cfg(test)]
mod tests {
    extern crate test;
    use Promise;
    use std::sync::mpsc::channel;
    use std::thread::Thread;
    use std::rand;

    #[test]
    fn test_promise_linear() {
        let (pt,pr) = Promise::new();
        assert_eq!(pt.deliver(1),true);
        assert_eq!(pt.deliver(2),false);
        assert_eq!(pr.with(|x| *x).unwrap(),1);
        let pr2 = pr.clone();
        assert_eq!(pr2.with(|x| *x).unwrap(),1);
    }

    #[test]
    fn test_promise_threaded() {
        let (pt,pr) = Promise::new();
        Thread::spawn(move || {
            assert_eq!(pt.deliver(1),true);
        });
        assert_eq!(pr.with(|x| *x).unwrap(),1); //waits on spawned thread
    }

    #[test]
    #[should_fail]
    fn test_promise_threaded_panic_safely() {
        let (pt,pr) = Promise::new();
        Thread::spawn (move || {
            panic!("proc dead"); //destroys promise, triggers wake on main proc
            pt.deliver(1);
        });
        
        pr.with(|x| *x).unwrap();
    }

    #[bench]
    fn bench_promise_build(b: &mut test::Bencher) {
        b.iter(|| {
            let (pt,pr) = Promise::<u8>::new();
        });
    }

    #[bench]
    fn bench_promise_clone(b: &mut test::Bencher) {
        let (pt,pr) = Promise::<u8>::new();
        b.iter(|| {
            pr.clone();
        });
    }

    #[bench]
    fn bench_promise(b: &mut test::Bencher) {
        let (pt,pr) = Promise::new();
        let bd = vec![rand::random::<u64>();1000];
        pt.deliver(bd); //delivery is a one shot deal

        b.iter(|| {
            pr.with(|x| x[999]);
        });
    }
    #[bench]
    fn bench_channel(b: &mut test::Bencher) {
        let (cs,cr) = channel::<Vec<u64>>();
        let bd = vec![rand::random::<u64>();1000];

        b.iter(|| {
            cs.send(bd.clone()); //must send each time w/ chan
            cr.recv().unwrap()[999];
        });
    }

    #[bench]
    fn bench_promise_multi(b: &mut test::Bencher) {
        b.iter(|| {
            let (pt,pr) = Promise::new();
            let mut vpr = vec![pr.clone();10];
            let bd = vec![rand::random::<u64>();1000];
            pt.deliver(bd);
            for n in vpr.drain() {
                n.with(|x| x[999]);
            }
        });
    }
    #[bench]
    fn bench_channel_multi(b: &mut test::Bencher) {
        let mut vcs = vec!();
        let mut vcr = vec!();
        for n in (0..10) {
            let (cs,cr) = channel::<Vec<u64>>();
            vcs.push(cs);
            vcr.push(cr);
        }
        b.iter(|| {
            let bd = vec![rand::random::<u64>();1000];
            for cs in vcs.iter() {
                cs.send(bd.clone());
            }
            for cr in vcr.iter(){
                cr.recv().unwrap()[999];
            }
        });
    }
}


