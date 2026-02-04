use crate::config::ChecksumDefinition;
use crate::{utils, vars};
use data_encoding::HEXLOWER;
use indicatif::{ProgressBar, ProgressStyle};
use log::{debug, info};
use reqwest::StatusCode;
use std::fs;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

pub fn download_to_stream(url: &str, dest_file: &mut File) -> Result<usize, String> {
    let client = reqwest::blocking::Client::new();
    let mut response = client
        .get(url)
        .send()
        .and_then(|r| r.error_for_status())
        .map_err(|e| e.to_string())?;

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
    let mut total_read = 0usize;

    loop {
        let bytes_read = match response.read(&mut buffer) {
            Ok(0) => return Ok(total_read),
            Ok(n) => Ok(n),
            Err(e) => Err(e.to_string()),
        }?;
        dest_file
            .write_all(&buffer[..bytes_read])
            .map_err(|e| e.to_string())?;

        total_read += bytes_read;
        pb.inc(bytes_read as u64);
    }
}

pub fn download_to_destination<'a>(url: &str, download_dest: &'a Path) -> Result<&'a Path, String> {
    let mut download_temp = tempfile::Builder::new()
        .prefix(".tmp-")
        .tempfile_in(download_dest.parent().unwrap())
        .map_err(|e| e.to_string())?;

    download_to_stream(url, download_temp.as_file_mut())?;
    fs::rename(download_temp, download_dest).map_err(|e| e.to_string())?;

    Ok(download_dest)
}

pub fn download_to_tempfile(url: &str, temp_dir: &Path) -> Result<File, String> {
    let temp_dir = if temp_dir.is_dir() {
        temp_dir
    } else {
        temp_dir.parent().unwrap()
    };
    let mut temp_file = tempfile::Builder::new()
        .prefix(".tmp-")
        .tempfile_in(temp_dir)
        .map_err(|e| e.to_string())?;

    let fetched_bytes = download_to_stream(url, temp_file.as_file_mut())?;
    debug!("Fetched {fetched_bytes} bytes");

    temp_file
        .seek(SeekFrom::Start(0))
        .map_err(|e| e.to_string())?;
    Ok(temp_file.into_file())
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
