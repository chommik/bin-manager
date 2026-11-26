use sha2::{Digest, Sha256};
use std::io::BufReader;
use std::path::Path;
use std::{fs, io};
use digest_io::IoWrapper;

pub fn calculate_checksum(path: &Path) -> Result<Vec<u8>, String> {
    let mut digest = IoWrapper(Sha256::new());

    let file = fs::File::open(path).map_err(|e| e.to_string())?;
    let mut reader = BufReader::with_capacity(32768usize, file);
    io::copy(&mut reader, &mut digest).map_err(|e| e.to_string())?;

    let result = digest.0.finalize().to_vec();
    Ok(result)
}
