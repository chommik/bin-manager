use std::{fs, io};
use std::io::BufReader;
use std::path::Path;
use sha2::{Digest, Sha256};
pub fn calculate_checksum(path: &Path) -> Result<Vec<u8>, String> {
    let mut digest = Sha256::new();

    let file = fs::File::open(path).map_err(|e| e.to_string())?;
    let mut reader = BufReader::with_capacity(32768usize, file);
    io::copy(&mut reader, &mut digest).map_err(|e| e.to_string())?;

    let result = digest.finalize().to_vec();
    Ok(result)
}
