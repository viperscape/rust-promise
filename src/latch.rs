use std::sync::atomic::{AtomicBool,Ordering};
use std::sync::{Arc};

#[derive(Clone)]
pub struct Latch {
    latch: Arc<AtomicBool>,
}

impl Latch {
    pub fn new () -> Latch {
        Latch { latch: Arc::new(AtomicBool::new(false)) }
    }

    /// close latch
    pub fn close (&self) -> bool {
        if !self.latch.compare_and_swap(false,true,Ordering::Release) {true}
        else {false}
    }

    /// is latch closed?
    pub fn latched (&self) -> bool {
        self.latch.load(Ordering::Acquire)
    }

    /// reopens latch, if necessary; do not use for stateful checks
    pub fn open (&self) -> bool {
        if self.latched() {
            self.latch.compare_and_swap(true,false,Ordering::Release)
        }
        else { true }
    }
}


#[cfg(test)]
mod tests {
    use Latch;
    
    #[test]
    fn test_latch() {
        let l = Latch::new();
        assert_eq!(l.latched(),false);
        assert_eq!(l.close(),true);
        assert_eq!(l.close(),false); //subsequent latching fails, already latched
        assert_eq!(l.latched(),true);
        assert_eq!(l.open(),true);
        assert_eq!(l.latched(),false);
    }
}
