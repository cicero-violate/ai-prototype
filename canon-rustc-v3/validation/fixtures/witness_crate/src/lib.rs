pub trait Worker {
    fn run(&self) -> usize;
}

pub struct Counter(pub usize);

impl Worker for Counter {
    fn run(&self) -> usize {
        self.0
    }
}

pub fn caller(worker: &impl Worker) -> usize {
    worker.run()
}

pub fn mutates(value: &mut usize) {
    *value += 1;
}

pub fn does_io(path: &std::path::Path) -> std::io::Result<String> {
    std::fs::read_to_string(path)
}

pub fn panics(value: Option<usize>) -> usize {
    value.expect("witness panic fact")
}

pub fn allocates() -> Vec<usize> {
    vec![1, 2, 3]
}

pub unsafe fn unsafe_boundary(ptr: *const usize) -> usize {
    unsafe { *ptr }
}