use std::fs;
use std::io;
use std::io::prelude::*;
use std::path::Path;
use std::result::Result;
use std::string::String;

#[derive(Debug)]
pub enum WriteStrToFileError {
    OpenFailed(io::Error),
    WriteFailed(io::Error),
}

impl std::fmt::Display for WriteStrToFileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OpenFailed(e) => write!(f, "Open file for writing failed {}", e),
            Self::WriteFailed(e) => write!(f, "Failed to write file {}", e),
        }
    }
}

impl std::error::Error for WriteStrToFileError {}

pub fn write_str_to_file<P: AsRef<Path>>(
    content: &str,
    path: P,
) -> Result<(), WriteStrToFileError> {
    let mut file = fs::File::create(path).map_err(WriteStrToFileError::OpenFailed)?;
    file.write_all(content.as_bytes())
        .map_err(WriteStrToFileError::WriteFailed)?;
    Ok(())
}

#[derive(Debug)]
pub enum ReadStringFromFileError {
    OpenFailed(io::Error),
    ReadFailed(io::Error),
}

impl std::fmt::Display for ReadStringFromFileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OpenFailed(e) => write!(f, "Open file for reading failed {}", e),
            Self::ReadFailed(e) => write!(f, "Failed to read file {}", e),
        }
    }
}

impl std::error::Error for ReadStringFromFileError {}

pub fn read_str_from_file<P: AsRef<Path>>(path: P) -> Result<String, ReadStringFromFileError> {
    let mut file = fs::File::open(path).map_err(ReadStringFromFileError::OpenFailed)?;
    let mut s = String::new();
    file.read_to_string(&mut s)
        .map_err(ReadStringFromFileError::ReadFailed)?;
    Ok(s)
}
