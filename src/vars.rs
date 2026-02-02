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
            "linux" => "linux-gnu",
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

    ret.insert(
        String::from("arch"),
        uname.machine().to_str().unwrap().to_string(),
    );

    ret
}

pub fn find_templated_checksum(checksum: &ChecksumDefinition) -> String {
    let checksum_idx = strfmt::strfmt(&checksum.key, &current_machine_vars()).unwrap();
    debug!("looking for checksum with key: {}", checksum_idx);

    checksum
        .values
        .get(&checksum_idx)
        .expect("expects checksum machine-dependent variables in definition")
        .clone()
}
