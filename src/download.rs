use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use indicatif::{ProgressBar, ProgressStyle};
use log::{debug, info};
use data_encoding::HEXLOWER;
use crate::config::ChecksumDefinition;
use crate::{utils, vars};

pub fn download_file<'a>(url: &str, download_dest: &'a Path) -> Result<&'a Path, String> {
    let client = reqwest::blocking::Client::new();

    let download_temp = tempfile::Builder::new()
        .prefix(".tmp-")
        .tempfile_in(download_dest.parent().unwrap())
        .map_err(|e| e.to_string())?;

    let mut response = client.get(url).send().map_err(|e| e.to_string())?;

    let pb = match response.content_length() {
        None => ProgressBar::no_length(),
        Some(len) => ProgressBar::new(len),
    }
        .with_style(
            ProgressStyle::with_template(
                "[{elapsed_precise}] [{wide_bar:.cyan/blue}] {decimal_bytes}/{decimal_total_bytes} @ {decimal_bytes_per_sec} ({eta})",
            )
                .unwrap()
                .progress_chars("████████▏▎▍▌▋▊▉█        "),
        );

    let mut buffer = [0u8; 8192];
    let mut out_file = download_temp.as_file();

    loop {
        let bytes_read = match response.read(&mut buffer) {
            Ok(0) => break,
            Ok(n) => Ok(n),
            Err(e) => Err(e.to_string()),
        }?;
        out_file
            .write(&buffer[..bytes_read])
            .map_err(|e| e.to_string())?;
        pb.inc(bytes_read as u64);
    }

    fs::rename(download_temp, download_dest).map_err(|e| e.to_string())?;

    Ok(download_dest)
}

pub fn is_download_necessary(
    dest_path: &PathBuf,
    checksum: &ChecksumDefinition,
) -> Result<bool, String> {
    if !fs::exists(dest_path).map_err(|e| e.to_string())? {
        debug!("target doesn't exist");
        return Ok(true);
    }

    let expected_checksum = vars::find_templated_checksum(checksum);
    let actual_checksum = HEXLOWER.encode(utils::calculate_checksum(dest_path)?.as_slice());

    debug!(
        "Expected checksum: {}, found: {}",
        expected_checksum, actual_checksum
    );

    if expected_checksum == actual_checksum {
        info!("Actual checksum matches expected, skipping download");
        return Ok(false);
    }

    Ok(true)
}