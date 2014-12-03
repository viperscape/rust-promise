use std::sync::atomic::{AtomicBool,Ordering};
use std::sync::{Arc};

pub struct Latch {
    latch:Arc<AtomicBool>,
}

impl Latch {
    pub fn new () -> Latch {
        Latch {latch:Arc::new(AtomicBool::new(false))}
    }
    pub fn close (&self) -> bool {
        if !self.latch.compare_and_swap(false,true,Ordering::SeqCst) {true}
        else {false}
    }
    pub fn latched (&self) -> bool {
        self.latch.load(Ordering::SeqCst)
    }
    pub fn clone (&self) -> Latch {
        Latch {latch:self.latch.clone()}
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
    }
}
