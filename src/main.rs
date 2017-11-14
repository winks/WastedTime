extern crate time;
extern crate user32;
extern crate winapi;

use std::thread;
use std::time as tm;
use user32::{GetForegroundWindow, GetWindowTextW, GetClassNameW};
use user32::{GetWindowThreadProcessId};

fn main() {
    let mut count = 0;
    let max = 0;
    let sleep_time = tm::Duration::from_millis(1000);

    let mut last = Result{
        timestamp: time::now(),
        title: String::new(),
        class: String::new(),
        pid: 0
    };
    let mut last_change = 0;

    loop {
        let current = get_info();
        if current.pid != last.pid {
            print_end(&current, &last, last_change);
            last_change = current.timestamp.to_timespec().sec;
            out(&current, format!("S"));
        } else {
            //println!("no change since {}", last_change);
        }
        count += 1;
        thread::sleep(sleep_time);
        
        if count >= max && max > 0 {
            print_end(&current, &last, last_change);
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
}

fn print_end(current: &Result, last: &Result, last_change: i64) {
    let diff = current.timestamp.to_timespec().sec - last_change;
    //last_change = current.timestamp.to_timespec().sec;
    if last.pid > 0 {
        out(&last, format!("E {}", diff));
    }
}

fn out(r: &Result, s: String) {
    println!(
        "{}|{}|{}|{}|{}",
        s,
        r.timestamp.to_timespec().sec,
        r.pid,
        r.class,
        r.title
    );
}

fn get_info() -> Result {
    unsafe {
        let win = GetForegroundWindow();
        //let mut len = GetWindowTextLengthW(w);
        //println!("{:?}", len);
        let mut title = [0 as winapi::WCHAR; winapi::minwindef::MAX_PATH];
        let mut cls = [0 as winapi::WCHAR; winapi::minwindef::MAX_PATH];
        let mut pid: winapi::DWORD = 0;
        let _ = GetWindowTextW(win, title.as_mut_ptr(), winapi::minwindef::MAX_PATH as winapi::INT);
        let _ = GetClassNameW(win, cls.as_mut_ptr(), winapi::minwindef::MAX_PATH as winapi::INT);
        let _ = GetWindowThreadProcessId(win, &mut pid as *mut winapi::DWORD);
        //println!("{:?}", t);
        //println!("{:?}", w);
        let now = time::now();
        //println!("{} {:?} {:?} {} {:?}", now.to_timespec().sec, from_u16(&title), from_u16(&cls), pid, win);
        let ret = Result{
            timestamp: now,
            title: from_u16(&title),
            class: from_u16(&cls),
            pid: pid,
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