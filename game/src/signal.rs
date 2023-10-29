use anyhow::Result;
use base64::{engine::general_purpose, Engine};

/// must_read_stdin blocks until input is received from stdin
pub fn must_read_stdin() -> Result<String> {
    let mut line = String::new();

    std::io::stdin().read_line(&mut line)?;
    line = line.trim().to_owned();

    Ok(line)
}

// Allows compressing offer/answer to bypass terminal input limits.
// const COMPRESS: bool = false;

/// encode encodes the input in base64
/// It can optionally zip the input before encoding
pub fn encode(b: &str) -> String {
    general_purpose::STANDARD.encode(b.as_bytes())
}

/// decode decodes the input from base64
/// It can optionally unzip the input after decoding
pub fn decode(s: &str) -> Result<String> {
    let b = general_purpose::STANDARD.decode(s)?;

    let s = String::from_utf8(b)?;
    Ok(s)
}
