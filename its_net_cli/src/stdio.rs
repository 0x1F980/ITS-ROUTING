//! `-` as stdin/stdout for CLI file paths (bash / zsh / fish compatible).

use std::io::{self, Read, Write};
use std::path::Path;

pub fn is_stdio(path: &Path) -> bool {
    path.as_os_str() == "-"
}

pub fn read_bytes(path: &Path) -> io::Result<Vec<u8>> {
    if is_stdio(path) {
        let mut buf = Vec::new();
        io::stdin().read_to_end(&mut buf)?;
        Ok(buf)
    } else {
        std::fs::read(path)
    }
}

pub fn read_text(path: &Path) -> io::Result<String> {
    let bytes = read_bytes(path)?;
    String::from_utf8(bytes).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

pub fn write_bytes(path: &Path, data: &[u8]) -> io::Result<()> {
    if is_stdio(path) {
        io::stdout().write_all(data)?;
        io::stdout().flush()
    } else {
        std::fs::write(path, data)
    }
}

pub fn log_status(quiet_stdout: bool, msg: &str) {
    if quiet_stdout {
        eprintln!("{msg}");
    } else {
        println!("{msg}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stdio_path_detection() {
        assert!(is_stdio(Path::new("-")));
        assert!(!is_stdio(Path::new("/tmp/x")));
    }
}
