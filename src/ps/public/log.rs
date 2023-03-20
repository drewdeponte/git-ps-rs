use super::super::private::utils;
use std::result::Result;

#[derive(Debug)]
pub enum LogError {
    LogFailed(utils::ExecuteError),
}

pub fn log() -> Result<(), LogError> {
    utils::execute("git", &["log", "--graph", "--abbrev-commit", "--date=relative", "--pretty=format:\"%C(yellow)%h%Creset - %G? -%C(red)%d%Creset %s %Cgreen(%ar) %C(bold blue)<%an>%Creset\"", "--topo-order"]).map_err(LogError::LogFailed)
}
