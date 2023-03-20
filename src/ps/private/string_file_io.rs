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

pub fn read_str_from_file<P: AsRef<Path>>(path: P) -> Result<String, ReadStringFromFileError> {
    let mut file = fs::File::open(path).map_err(ReadStringFromFileError::OpenFailed)?;
    let mut s = String::new();
    file.read_to_string(&mut s)
        .map_err(ReadStringFromFileError::ReadFailed)?;
    Ok(s)
}
