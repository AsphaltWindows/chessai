//! Helpers for the whitespace-separated text model formats of the C code.

use std::fmt::Display;
use std::io;
use std::iter::Peekable;
use std::str::{FromStr, SplitWhitespace};

pub struct Tokens<'a>(Peekable<SplitWhitespace<'a>>);

impl<'a> Tokens<'a> {
    pub fn new(text: &'a str) -> Self {
        Tokens(text.split_whitespace().peekable())
    }

    /// The next token, without consuming it.
    pub fn peek(&mut self) -> Option<&'a str> {
        self.0.peek().copied()
    }

    pub fn next<T: FromStr>(&mut self) -> io::Result<T>
    where
        T::Err: Display,
    {
        let tok = self.0.next().ok_or_else(|| invalid("model file ended early"))?;
        tok.parse().map_err(|e: T::Err| invalid(format!("{e}: {tok:?}")))
    }
}

pub fn invalid<E: ToString>(e: E) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, e.to_string())
}
