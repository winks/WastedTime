extern crate kernel32;
extern crate time;
extern crate user32;
extern crate winapi;

#[macro_use]
extern crate serde_json;

use std::thread;
use std::time as tm;
use kernel32::{CloseHandle, OpenProcess, K32GetModuleFileNameExW};
use user32::{GetForegroundWindow, GetWindowTextW, GetClassNameW};
use user32::{GetWindowThreadProcessId};

const PROCESS_QUERY_INFORMATION: winapi::DWORD = 0x0400;
const PROCESS_VM_READ: winapi::DWORD = 0x0010;

fn main() {
    let mut count = 0;
    let max = 0;
    let sleep_time = tm::Duration::from_millis(1000);

    let mut last = Result{
        timestamp: time::now(),
        title: String::new(),
        class: String::new(),
        pid: 0,
        path: String::new(),
    };
    let mut last_change = 0;

    loop {
        let current = get_info();
        if current.pid != last.pid {
            print_end(&last, last_change);
            last_change = current.timestamp.to_timespec().sec;
            //out(&current, last_change, "S");
        } else {
            //println!("no change since {}", last_change);
        }
        count += 1;
        thread::sleep(sleep_time);
        
        if count >= max && max > 0 {
            print_end(&last, last_change);
            break;
        }
        last = current;
    }
}

#[derive(Debug)]
struct Result {
    timestamp: time::Tm,
    title: String,
    class: String,
    pid: winapi::DWORD,
    path: String,
}

fn print_end(last: &Result, last_change: i64) {
    if last.pid > 0 {
        out(&last, last_change, "E");
    }
}

fn out(r: &Result, last_change: i64, s: &str) {
    let diff = r.timestamp.to_timespec().sec - last_change;
    println!(
        "#{} {}|{}|{}|{}|{}|{}",
        s,
        diff,
        r.timestamp.to_timespec().sec,
        r.pid,
        r.class,
        r.path,
        r.title
    );
    if s == "S" {
        return;
    }
    let out = json!({
        "time": diff,
        "timestamp": r.timestamp.to_timespec().sec,
        "pid": r.pid,
        "class": r.class,
        "path": r.path,
        "title": r.title,
    });
    println!("{}", out.to_string());
}

fn get_info() -> Result {
    unsafe {
        let now = time::now();
        let win = GetForegroundWindow();
        let max_len = winapi::minwindef::MAX_PATH as winapi::INT;

        let mut title = [0 as winapi::WCHAR; winapi::minwindef::MAX_PATH];
        let mut cls = [0 as winapi::WCHAR; winapi::minwindef::MAX_PATH];
        let mut pid: winapi::DWORD = 0;
        let _ = GetWindowTextW(win, title.as_mut_ptr(), max_len);
        let _ = GetClassNameW(win, cls.as_mut_ptr(), max_len);
        let _ = GetWindowThreadProcessId(win, &mut pid as *mut winapi::DWORD);

        let op_flags = PROCESS_QUERY_INFORMATION | PROCESS_VM_READ;
        let ph = OpenProcess(op_flags, 0, pid);

        let mut mod_name = [0 as winapi::WCHAR; winapi::minwindef::MAX_PATH];
        let _ = K32GetModuleFileNameExW(ph, 0 as winapi::HINSTANCE, mod_name.as_mut_ptr(), max_len as winapi::UINT);

        CloseHandle(ph);

        let ret = Result{
            timestamp: now,
            title: from_u16(&title),
            class: from_u16(&cls),
            pid: pid,
            path: from_u16(&mod_name),
        };
        return ret
    }
}

fn from_u16(s: &[u16]) -> String {
  // panic if there's no null terminator
  let pos = s.iter().position(|a| a == &0u16).unwrap();
  use std::ffi::OsString;
  use std::os::windows::ffi::OsStringExt;
  let s2: OsString = OsStringExt::from_wide(&s[..pos]);
  s2.to_string_lossy().to_string()
}