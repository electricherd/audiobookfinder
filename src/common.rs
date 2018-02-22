// common things

use std::thread;

/// This ThreadPool is just a pool of predefined, fixed number
/// of threads. Instead of spawning new, you can just
/// renew it.
/// It is not ideally but a step and nice example of how
/// to pass a closure into a function.
pub struct ThreadPool {
    threads: Vec<thread::JoinHandle<()>>,
}

impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);
        let mut threads = Vec::with_capacity(size);
        for _ in 0..size {
            threads.push(thread::spawn(|| {}));
        }
        ThreadPool { threads }
    }

    // awesome: taking the closure directly from the call!! Works!!
    // but static lifetime seems presumptuous
    pub fn renew<F: 'static + Send>(&mut self, idx: usize, f: F)
    where
        F: FnOnce(),
    {
        assert!(idx < self.threads.len());
        drop(&self.threads[idx]);
        self.threads[idx] = thread::spawn(move || f());
    }
}
