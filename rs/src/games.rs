//! Reading the `.games` training files written by the self-play scripts:
//! one encoded position per line, values separated by spaces.

use std::io;
use std::path::Path;

use crate::encoding::{Encoded, FEATURES};

/// Outcome files in a games directory and the one-hot label of each.
pub const OUTCOME_FILES: [(&str, [f64; 3]); 3] = [
    ("white_wins.games", [1.0, 0.0, 0.0]),
    ("black_wins.games", [0.0, 1.0, 0.0]),
    ("draws.games", [0.0, 0.0, 1.0]),
];

pub fn read_positions(path: &Path) -> io::Result<Vec<Encoded>> {
    std::fs::read_to_string(path)?
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(parse_position)
        .collect()
}

pub fn parse_position(line: &str) -> io::Result<Encoded> {
    let mut enc = [0u8; FEATURES];
    let mut n = 0;
    for tok in line.split_whitespace() {
        if n == FEATURES {
            return Err(invalid(format!("more than {FEATURES} values: {line}")));
        }
        enc[n] = tok.parse().map_err(|e| invalid(format!("{e}: {line}")))?;
        n += 1;
    }
    if n != FEATURES {
        return Err(invalid(format!("expected {FEATURES} values, got {n}: {line}")));
    }
    Ok(enc)
}

/// All positions in a games directory, labelled by game outcome, in the
/// same order the C harness trains on them.
pub fn read_labelled(dir: &Path) -> io::Result<Vec<(Encoded, [f64; 3])>> {
    let mut data = Vec::new();
    for (file, label) in OUTCOME_FILES {
        data.extend(read_positions(&dir.join(file))?.into_iter().map(|p| (p, label)));
    }
    Ok(data)
}

fn invalid(msg: String) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, msg)
}
