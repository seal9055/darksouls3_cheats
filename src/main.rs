use std::path::Path;

use sysinfo::{ProcessExt, System, SystemExt};

const DEBUG: bool = false;

/// retrieves target pid, and uses it to inject selected DLL into target process
fn main() {
    let dll_path = if DEBUG {
        "target\\debug\\darksouls3_cheats.dll"
    } else {
        "target\\release\\darksouls3_cheats.dll"
    };
    let dll = Path::new(dll_path).canonicalize().unwrap().into_os_string().into_string().unwrap();
    let pid = System::new_all().process_by_name("DarkSoulsIII")[0].pid() as u32;

    println!("[+] PID: {}", pid);

    unsafe {
        if dll_injector::inject_dll(pid, &dll).is_none() {
            println!("Error injecting dll {} into process with pid: {}", dll_path, pid);
        }
    }
}
