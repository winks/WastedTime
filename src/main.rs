extern crate time;
extern crate user32;
extern crate winapi;

use std::thread;
use std::time as tm;
//use time::Duration;
use user32::{GetForegroundWindow, GetWindowTextW, GetClassNameW};
use user32::{GetWindowThreadProcessId};
 //, GetWindowTextLengthW, SetWinEventHook};

//use winapi::EVENT_SYSTEM_FOREGROUND;

//fn main1() {
    //unsafe {
        //let hook = SetWinEventHook(0x0003, 0x0003,NULL, cb, 0, 0,  0x0000 | 0x0002);
    //}
//}
// https://msdn.microsoft.com/en-us/library/windows/desktop/dd373640(v=vs.85).aspx
// http://www.pinvoke.net/default.aspx/user32.setwineventhook
// https://stackoverflow.com/questions/4407631/is-there-windows-system-event-on-active-window-changed
// https://gist.github.com/pfn/80b7b5f081594c1d935d6f23f6c66b40
// https://stackoverflow.com/questions/1933113/c-windows-how-to-get-process-path-from-its-pid
// https://retep998.github.io/doc/kernel32/fn.OpenProcess.html https://retep998.github.io/doc/kernel32/fn.GetModuleFileNameW.html

fn main() {
    let mut count = 0;
    let max = 0;
    //let sleep_time = std::time::Duration::milliseconds(1000);
    let sleep_time = tm::Duration::from_millis(1000);

    loop {
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
            println!("{} {:?} {:?} {} {:?}", now.to_timespec().sec, from_u16(&title), from_u16(&cls), pid, win);
            count += 1;
            thread::sleep(sleep_time);
        }
        if count >= max && max > 0 {
            break;
        }
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