use super::super::super::utils;
use super::signer_error::SignerError;

pub fn gpg_signer(
    signing_key: String,
    program: Option<String>,
) -> impl Fn(String) -> Result<String, SignerError> {
    move |commit_string: String| {
        gpg_sign_string(commit_string, signing_key.clone(), program.clone())
            .map_err(|e| SignerError::Signing(e.into()))
    }
}

#[derive(Debug)]
enum GpgSignStringError {
    Unhandled(Box<dyn std::error::Error>),
}

impl std::fmt::Display for GpgSignStringError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GpgSignStringError::Unhandled(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for GpgSignStringError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Unhandled(e) => Some(e.as_ref()),
        }
    }
}

fn gpg_sign_string(
    commit: String,
    signing_key: String,
    program: Option<String>,
) -> Result<String, GpgSignStringError> {
    let prog = program.unwrap_or("gpg".to_string());
    let output = utils::execute_with_input_and_output(
        &commit,
        &prog,
        &[
            "--local-user",
            &signing_key,
            "--sign",
            "--armor",
            "--detach-sig",
        ],
    )
    .map_err(|e| GpgSignStringError::Unhandled(e.into()))?;

    String::from_utf8(output.stdout).map_err(|e| GpgSignStringError::Unhandled(e.into()))
}
