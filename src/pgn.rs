use std::{fs::File, str::Utf8Error};

use chrono::{NaiveDate, ParseError};
use pgn_reader::{BufferedReader, Visitor};

pub fn from_file(path: &str) -> std::io::Result<BufferedReader<File>> {
    let file = std::fs::OpenOptions::new().read(true).open(path)?;
    let reader = pgn_reader::BufferedReader::new(file);
    Ok(reader)
}

#[derive(Debug, Clone)]
pub struct OutcomeResult {
    pub white: String,
    pub black: String,
    pub date: NaiveDate,
    pub outcome: pgn_reader::Outcome,
}

#[derive(Debug, Clone, Default)]
pub struct Outcome {
    pub white: String,
    pub black: String,
    pub date: Option<NaiveDate>,
    pub outcome: Option<pgn_reader::Outcome>,

    error: Option<OutcomeError>,
}

impl Visitor for Outcome {
    type Result = Result<OutcomeResult, OutcomeError>;

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
        if key == "Date".as_bytes() {
            self.date = match value.decode_utf8() {
                Ok(o) => match NaiveDate::parse_from_str(&o, "%Y.%m.%d") {
                    Ok(o) => Some(o),
                    Err(e) => {
                        self.error = Some(e.into());
                        return;
                    }
                },
                Err(e) => {
                    self.error = Some(e.into());
                    return;
                }
            }
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
            None => Ok(OutcomeResult {
                white: self.white.clone(),
                black: self.black.clone(),
                outcome: self.outcome.expect("Unreachable."),
                date: self.date.unwrap(),
            }),
        }
    }
}

#[derive(Debug, Clone, Copy, thiserror::Error)]
pub enum OutcomeError {
    #[error("Error decoding utf8 in header: {0}")]
    HeaderUtf8(#[from] Utf8Error),

    #[error("No outcome found in game")]
    NoOutcome,

    #[error("Date formatting error: {0}")]
    DateFormatting(#[from] ParseError),
}
