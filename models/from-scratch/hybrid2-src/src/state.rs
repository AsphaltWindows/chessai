//! Training state beyond what model files hold, for an exact `--resume`.
//!
//! The trees' model files hold their structure and their nodes' models (the
//! delegation trees' in the C text format, with CNBP counts to 6 decimals),
//! but not their split statistics, windows, trials, shadows and error
//! routers, so a reloaded tree predicts (nearly) the same but trains
//! differently. Self-play therefore also writes a tree's whole state beside
//! each model it saves, as `{name}N.state`, in this compact binary format: a
//! header line naming the format and the model kind, then the values in
//! order, little-endian, floats as their bits, so that nothing is rounded.

use std::io;
use std::path::Path;

use crate::textfmt::invalid;

const MAGIC: &str = "chessai-state 1";

pub struct Writer {
    buf: Vec<u8>,
}

impl Writer {
    /// A state of a model of `kind` (bdt, bdd, ht).
    pub fn new(kind: &str) -> Self {
        Writer { buf: format!("{MAGIC} {kind}\n").into_bytes() }
    }

    pub fn u8(&mut self, x: u8) {
        self.buf.push(x);
    }

    pub fn bool(&mut self, x: bool) {
        self.u8(x as u8);
    }

    pub fn u32(&mut self, x: u32) {
        self.buf.extend_from_slice(&x.to_le_bytes());
    }

    pub fn u64(&mut self, x: u64) {
        self.buf.extend_from_slice(&x.to_le_bytes());
    }

    pub fn usize(&mut self, x: usize) {
        self.u64(x as u64);
    }

    pub fn f64(&mut self, x: f64) {
        self.u64(x.to_bits());
    }

    pub fn str(&mut self, s: &str) {
        self.u8s(s.as_bytes());
    }

    pub fn u8s(&mut self, xs: &[u8]) {
        self.usize(xs.len());
        self.buf.extend_from_slice(xs);
    }

    pub fn u16s(&mut self, xs: &[u16]) {
        self.usize(xs.len());
        xs.iter().for_each(|x| self.buf.extend_from_slice(&x.to_le_bytes()));
    }

    pub fn usizes(&mut self, xs: &[usize]) {
        self.usize(xs.len());
        xs.iter().for_each(|&x| self.usize(x));
    }

    pub fn f64s(&mut self, xs: &[f64]) {
        self.usize(xs.len());
        self.buf.reserve(8 * xs.len());
        xs.iter().for_each(|&x| self.f64(x));
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.buf
    }

    /// Writes the state to `path`, through a temporary file, so a run
    /// stopped while writing leaves no partial state.
    pub fn save(self, path: &Path) -> io::Result<()> {
        let mut tmp = path.as_os_str().to_owned();
        tmp.push(".tmp");
        std::fs::write(&tmp, &self.buf)?;
        std::fs::rename(&tmp, path)
    }
}

pub struct Reader<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    /// Reads a state written by `Writer::new(kind)`.
    pub fn new(data: &'a [u8], kind: &str) -> io::Result<Self> {
        let header = format!("{MAGIC} {kind}\n");
        if !data.starts_with(header.as_bytes()) {
            let line = data.split(|&b| b == b'\n').next().unwrap_or_default();
            return Err(invalid(format!(
                "not a {kind} state file (header {:?})",
                String::from_utf8_lossy(&line[..line.len().min(40)])
            )));
        }
        Ok(Reader { data, pos: header.len() })
    }

    fn take(&mut self, n: usize) -> io::Result<&'a [u8]> {
        let end = self.pos.checked_add(n).filter(|&end| end <= self.data.len());
        let end = end.ok_or_else(|| invalid("state file ended early"))?;
        let bytes = &self.data[self.pos..end];
        self.pos = end;
        Ok(bytes)
    }

    pub fn u8(&mut self) -> io::Result<u8> {
        Ok(self.take(1)?[0])
    }

    pub fn bool(&mut self) -> io::Result<bool> {
        match self.u8()? {
            0 => Ok(false),
            1 => Ok(true),
            b => Err(invalid(format!("bad bool {b} in state file"))),
        }
    }

    pub fn u32(&mut self) -> io::Result<u32> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }

    pub fn u64(&mut self) -> io::Result<u64> {
        Ok(u64::from_le_bytes(self.take(8)?.try_into().unwrap()))
    }

    pub fn usize(&mut self) -> io::Result<usize> {
        usize::try_from(self.u64()?).map_err(|_| invalid("size out of range in state file"))
    }

    pub fn f64(&mut self) -> io::Result<f64> {
        Ok(f64::from_bits(self.u64()?))
    }

    pub fn str(&mut self) -> io::Result<String> {
        String::from_utf8(self.u8s()?).map_err(|_| invalid("bad string in state file"))
    }

    /// The length of a sequence of `size`-byte items, checked against the
    /// bytes left.
    fn len(&mut self, size: usize) -> io::Result<usize> {
        let n = self.usize()?;
        if n.checked_mul(size).is_none_or(|bytes| bytes > self.data.len() - self.pos) {
            return Err(invalid("state file ended early"));
        }
        Ok(n)
    }

    pub fn u8s(&mut self) -> io::Result<Vec<u8>> {
        let n = self.len(1)?;
        Ok(self.take(n)?.to_vec())
    }

    pub fn u16s(&mut self) -> io::Result<Vec<u16>> {
        let n = self.len(2)?;
        Ok(self.take(2 * n)?.chunks_exact(2).map(|b| u16::from_le_bytes([b[0], b[1]])).collect())
    }

    pub fn usizes(&mut self) -> io::Result<Vec<usize>> {
        let n = self.len(8)?;
        (0..n).map(|_| self.usize()).collect()
    }

    pub fn f64s(&mut self) -> io::Result<Vec<f64>> {
        let n = self.len(8)?;
        Ok(self
            .take(8 * n)?
            .chunks_exact(8)
            .map(|b| f64::from_bits(u64::from_le_bytes(b.try_into().unwrap())))
            .collect())
    }

    /// Checks that a value written as `what` matches the one expected, such
    /// as the options a model was trained with.
    pub fn expect_str(&mut self, what: &str, expected: &str) -> io::Result<()> {
        let saved = self.str()?;
        if saved != expected {
            return Err(invalid(format!(
                "the state's {what} differ from this run's (give the options it was trained with):\n  saved: {saved}\n  now:   {expected}"
            )));
        }
        Ok(())
    }

    /// Checks that the whole state was read.
    pub fn finish(self) -> io::Result<()> {
        if self.pos != self.data.len() {
            return Err(invalid(format!("{} bytes left over in state file", self.data.len() - self.pos)));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn values_round_trip_exactly() {
        let mut w = Writer::new("test");
        w.u8(7);
        w.bool(true);
        w.u32(u32::MAX);
        w.u64(u64::MAX - 1);
        w.usize(12345);
        for x in [0.1, -0.0, f64::INFINITY, f64::NEG_INFINITY, f64::MIN_POSITIVE, 1.0 / 3.0] {
            w.f64(x);
        }
        w.str("options ü");
        w.u8s(&[1, 2, 3]);
        w.u16s(&[0, 65535, 9]);
        w.usizes(&[4, 0, usize::MAX >> 1]);
        w.f64s(&[std::f64::consts::PI, f64::NAN]);
        let bytes = w.into_bytes();

        let mut r = Reader::new(&bytes, "test").unwrap();
        assert_eq!(r.u8().unwrap(), 7);
        assert!(r.bool().unwrap());
        assert_eq!(r.u32().unwrap(), u32::MAX);
        assert_eq!(r.u64().unwrap(), u64::MAX - 1);
        assert_eq!(r.usize().unwrap(), 12345);
        for x in [0.1, -0.0, f64::INFINITY, f64::NEG_INFINITY, f64::MIN_POSITIVE, 1.0 / 3.0] {
            assert_eq!(r.f64().unwrap().to_bits(), f64::to_bits(x));
        }
        assert_eq!(r.str().unwrap(), "options ü");
        assert_eq!(r.u8s().unwrap(), [1, 2, 3]);
        assert_eq!(r.u16s().unwrap(), [0, 65535, 9]);
        assert_eq!(r.usizes().unwrap(), [4, 0, usize::MAX >> 1]);
        let fs = r.f64s().unwrap();
        assert_eq!(fs[0], std::f64::consts::PI);
        assert!(fs[1].is_nan());
        r.finish().unwrap();
    }

    #[test]
    fn rejects_other_kinds_and_truncation() {
        let mut w = Writer::new("bdt");
        w.f64s(&[1.0, 2.0]);
        let bytes = w.into_bytes();
        assert!(Reader::new(&bytes, "ht").is_err());
        let mut r = Reader::new(&bytes[..bytes.len() - 1], "bdt").unwrap();
        assert!(r.f64s().is_err());
        let mut r = Reader::new(&bytes, "bdt").unwrap();
        r.u64().unwrap();
        assert!(r.finish().is_err());
    }
}
