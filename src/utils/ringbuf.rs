/// Fixed-size ring buffer for streaming computations.
#[derive(Debug, Clone)]
pub struct RingBuf<T: Clone> {
    buf: Vec<T>,
    cap: usize,
    head: usize,
    len: usize,
}

impl<T: Clone> RingBuf<T> {
    pub fn new(cap: usize, initial: T) -> Self {
        assert!(cap > 0, "RingBuf capacity must be > 0");
        Self {
            buf: vec![initial; cap],
            cap,
            head: 0,
            len: 0,
        }
    }

    /// Push a value; returns the overwritten value if buffer was full.
    pub fn push(&mut self, value: T) -> Option<T> {
        let overwritten = if self.is_full() {
            Some(self.buf[self.head].clone())
        } else {
            None
        };

        self.buf[self.head] = value;
        self.head = (self.head + 1) % self.cap;
        self.len = self.len.saturating_add(1).min(self.cap);
        overwritten
    }

    /// Current number of valid elements.
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_full(&self) -> bool {
        self.len == self.cap
    }

    /// Iterate from oldest to newest.
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        (0..self.len).map(move |i| &self.buf[(self.head + self.cap - self.len + i) % self.cap])
    }

    /// Get the most recent element (if any).
    pub fn latest(&self) -> Option<&T> {
        if self.len == 0 {
            None
        } else {
            let idx = (self.head + self.cap - 1) % self.cap;
            self.buf.get(idx)
        }
    }
}
