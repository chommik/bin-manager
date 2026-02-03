use crate::config::ChecksumDefinition;
use log::debug;
use std::collections::HashMap;

pub fn current_machine_vars() -> HashMap<String, String> {
    let mut ret = HashMap::new();
    let uname = nix::sys::utsname::uname().unwrap();

    let uname_os = uname.sysname().to_str().unwrap().to_lowercase();
    ret.insert(String::from("os"), uname_os.clone());

    ret.insert(
        String::from("os_libc"),
        String::from(match uname_os.as_str() {
            "darwin" => "darwin",
            "linux" => "linux-gnu", // TODO: detect correct libc
            s => s,
        }),
    );

    ret.insert(
        String::from("os_musl"),
        String::from(match uname_os.as_str() {
            "darwin" => "darwin",
            "linux" => "linux-musl",
            s => s,
        }),
    );

    ret.insert(
        String::from("vendor"),
        String::from(match uname_os.as_str() {
            "darwin" => "apple",
            "linux" => "unknown",
            _ => "unknown",
        }),
    );

    let os_arch = uname.machine().to_str().unwrap().to_string();
    ret.insert(String::from("arch"), os_arch.clone());
    ret.insert(
        String::from("cpu"),
        String::from(match os_arch.as_str() {
            "x86_64" => "amd64",
            a => a,
        }),
    );

    ret.insert("triplet".to_string(), current_platform::CURRENT_PLATFORM.to_string());

    ret
}

pub fn find_templated_checksum(checksum: &ChecksumDefinition) -> String {
    let checksum_key = match &checksum.key {
        Some(ck) => ck,
        None => current_platform::CURRENT_PLATFORM
    };
    let checksum_idx = strfmt::strfmt(checksum_key, &current_machine_vars()).unwrap();
    debug!("looking for checksum with key: {}", checksum_idx);

    checksum
        .values
        .get(&checksum_idx)
        .expect("expects checksum machine-dependent variables in definition")
        .clone()
}
