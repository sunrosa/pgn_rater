use std::{borrow::Cow, fs::File, io::Read, str::Utf8Error};

use pgn_reader::{BufferedReader, Visitor};

pub fn from_file(path: &str) -> std::io::Result<BufferedReader<File>> {
    let file = std::fs::OpenOptions::new().read(true).open(path)?;
    let reader = pgn_reader::BufferedReader::new(file);
    Ok(reader)
}

#[derive(Debug, Clone, Default)]
pub struct Outcome {
    pub white: String,
    pub black: String,
    pub outcome: Option<pgn_reader::Outcome>,

    error: Option<OutcomeError>,
}

impl Visitor for Outcome {
    type Result = Result<(String, String, pgn_reader::Outcome), OutcomeError>;

    fn header(&mut self, key: &[u8], value: pgn_reader::RawHeader<'_>) {
        if key == "White".as_bytes() {
            self.white = match value.decode_utf8() {
                Ok(o) => o.to_string(),
                Err(e) => {
                    self.error = Some(e.into());
                    return;
                }
            };
        }
        if key == "Black".as_bytes() {
            self.black = match value.decode_utf8() {
                Ok(o) => o.to_string(),
                Err(e) => {
                    self.error = Some(e.into());
                    return;
                }
            };
        }
    }

    fn outcome(&mut self, outcome: Option<pgn_reader::Outcome>) {
        self.outcome = match outcome {
            Some(s) => Some(s),
            None => {
                self.error = Some(OutcomeError::NoOutcome);
                return;
            }
        };
    }

    fn end_game(&mut self) -> Self::Result {
        match self.error {
            Some(s) => Err(s),
            None => Ok((
                self.white.clone(),
                self.black.clone(),
                self.outcome.expect("Unreachable."),
            )),
        }
    }
}

#[derive(Debug, Clone, Copy, thiserror::Error)]
pub enum OutcomeError {
    #[error("Error decoding utf8 in header")]
    HeaderUtf8(#[from] Utf8Error),

    #[error("No outcome found in game")]
    NoOutcome,
}
