use std::io::prelude::*;

use std::cmp;
use std::fmt;
use std::io::{self, IoSliceMut};

pub struct BufReader<'r, R> {
    inner: &'r mut R,
    buf: Box<[u8]>,
    pos: usize,
    cap: usize,
}

impl<'r, R: Read> BufReader<'r, R> {
    pub fn new(inner: &'r mut R) -> BufReader<R> {
        BufReader::with_capacity(4 << 10, inner)
    }

    pub fn with_capacity(capacity: usize, inner: &'r mut R) -> BufReader<R> {
        let buffer = vec![0; capacity];
        BufReader {
            inner,
            buf: buffer.into_boxed_slice(),
            pos: 0,
            cap: 0,
        }
    }
}

impl<'r, R> BufReader<'r, R> {
    pub fn get_ref(&self) -> &R { &self.inner }

    pub fn get_mut(&mut self) -> &mut R { &mut self.inner }

    pub fn buffer(&self) -> &[u8] {
        &self.buf[self.pos..self.cap]
    }

    pub fn into_inner(self) -> &'r R { self.inner }

    #[inline]
    fn discard_buffer(&mut self) {
        self.pos = 0;
        self.cap = 0;
    }
}

impl<'r, R: Read> Read for BufReader<'r, R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.pos == self.cap && buf.len() >= self.buf.len() {
            self.discard_buffer();
            return self.inner.read(buf);
        }
        let nread = {
            let mut rem = self.fill_buf()?;
            rem.read(buf)?
        };
        self.consume(nread);
        Ok(nread)
    }

    fn read_vectored(&mut self, bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        let total_len = bufs.iter().map(|b| b.len()).sum::<usize>();
        if self.pos == self.cap && total_len >= self.buf.len() {
            self.discard_buffer();
            return self.inner.read_vectored(bufs);
        }
        let nread = {
            let mut rem = self.fill_buf()?;
            rem.read_vectored(bufs)?
        };
        self.consume(nread);
        Ok(nread)
    }
}

impl<'r, R: Read> BufRead for BufReader<'r, R> {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        if self.pos >= self.cap {
            debug_assert!(self.pos == self.cap);
            self.cap = self.inner.read(&mut self.buf)?;
            self.pos = 0;
        }
        Ok(&self.buf[self.pos..self.cap])
    }

    fn consume(&mut self, amt: usize) {
        self.pos = cmp::min(self.pos + amt, self.cap);
    }
}

impl<'r, R> fmt::Debug for BufReader<'r, R> where R: fmt::Debug {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("BufReader")
            .field("reader", &self.inner)
            .field("buffer", &format_args!("{}/{}", self.cap - self.pos, self.buf.len()))
            .finish()
    }
}

