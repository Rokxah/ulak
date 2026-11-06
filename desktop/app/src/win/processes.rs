// SPDX-License-Identifier: GPL-3.0-or-later
//! The programs running on this PC, for `pc_apps`.

use std::collections::HashSet;

/// Lowercase executable names of the processes running now.
pub fn running_programs() -> HashSet<String> {
    use windows::Win32::{
        Foundation::CloseHandle,
        System::Diagnostics::ToolHelp::{
            CreateToolhelp32Snapshot, PROCESSENTRY32W, Process32FirstW, Process32NextW, TH32CS_SNAPPROCESS,
        },
    };
    let mut names = HashSet::new();
    // SAFETY: a process snapshot walked with a correctly sized entry; the
    // handle is closed below.
    unsafe {
        let Ok(snapshot) = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) else { return names };
        let mut entry = PROCESSENTRY32W { dwSize: size_of::<PROCESSENTRY32W>() as u32, ..Default::default() };
        let mut ok = Process32FirstW(snapshot, &mut entry).is_ok();
        while ok {
            let len = entry.szExeFile.iter().position(|&c| c == 0).unwrap_or(entry.szExeFile.len());
            names.insert(String::from_utf16_lossy(&entry.szExeFile[..len]).to_lowercase());
            ok = Process32NextW(snapshot, &mut entry).is_ok();
        }
        let _ = CloseHandle(snapshot);
    }
    names
}

/// Waits until process `pid` has ended, or `timeout` passes.
pub fn wait_for_exit(pid: u32, timeout: std::time::Duration) {
    use windows::Win32::{
        Foundation::CloseHandle,
        System::Threading::{OpenProcess, PROCESS_SYNCHRONIZE, WaitForSingleObject},
    };
    // SAFETY: a handle only waited on, then closed.
    unsafe {
        let Ok(process) = OpenProcess(PROCESS_SYNCHRONIZE, false, pid) else { return };
        WaitForSingleObject(process, u32::try_from(timeout.as_millis()).unwrap_or(u32::MAX));
        let _ = CloseHandle(process);
    }
}
