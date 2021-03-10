extern crate kernel32;
extern crate regex;
extern crate time;
extern crate toml;
extern crate user32;
extern crate winapi;

#[macro_use]
extern crate serde_json;

use regex::Regex;
use std::thread;
use std::time as tm;
use std::io::Read;
use std::path::Path;
use std::fs::File;
use toml::Value;
use kernel32::{CloseHandle, OpenProcess, K32GetModuleFileNameExW};
use user32::{GetForegroundWindow, GetWindowTextW, GetClassNameW};
use user32::{GetWindowThreadProcessId};

const PROCESS_QUERY_INFORMATION: winapi::DWORD = 0x0400;
const PROCESS_VM_READ: winapi::DWORD = 0x0010;

#[derive(Debug)]
struct Result {
    timestamp: time::Tm,
    title: String,
    class: String,
    pid: winapi::DWORD,
    path: String,
}

impl Result {
    pub fn new(title: String, class: String, path: String, pid: winapi::DWORD) -> Result{
        Result{
            timestamp: time::now(),
            title: title,
            class: class,
            pid: pid,
            path: path,
        }
    }
    pub fn empty() -> Result{
        Result{
            timestamp: time::now(),
            title: String::new(),
            class: String::new(),
            pid: 0,
            path: String::new(),
        }
    }
}

struct Item {
    title: Regex,
    class: Regex,
    path: Regex,
    name: String,
}

impl Item {
    pub fn new(title: &str, class: &str, path: &str, name: &str) -> Item{
        //println!("#XX {}|{}|{}", title, class, path);
        let _title = match title {
            "" => r".",
            _ => title,
        };
        let _class = match class {
            "" => r".",
            _ => class,
        };
        let _path = match path {
            "" => r".",
            _ => path,
        };
        let _name = match name {
            "" => String::new(),
            _ => name,
        };
        Item{
            title: Regex::new(_title).unwrap(),
            class: Regex::new(_class).unwrap(),
            path: Regex::new(_path).unwrap(),
            name: _name,
        }
    }
}

fn main() {
    let mut count = 0;
    let max = 0;
    let sleep_time = tm::Duration::from_millis(1000);

    let mut last = Result::empty();
    let mut last_change = 0;

    let config_filename = "./config/config.toml".to_string();
    let cfg = read_config_file(config_filename);
    let ignorelist = parse_toml(&cfg);

    loop {
        thread::sleep(sleep_time);
        let current = get_info(&ignorelist);
        if current.pid == 0 {
            continue;
        }
        if current.pid != last.pid {
            print_end(&last, last_change);
            last_change = current.timestamp.to_timespec().sec;
            //out(&current, last_change, "S");
        } else {
            //println!("no change since {}", last_change);
        }
        count += 1;

        if count >= max && max > 0 {
            print_end(&last, last_change);
            break;
        }
        last = current;
    }
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

fn get_info(ignorelist: &Vec<Item>) -> Result {
    unsafe {
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

        let ret = Result::new(from_u16(&title), from_u16(&cls), from_u16(&mod_name), pid);
        let empty = Result::empty();

        for item in ignorelist.iter() {
            if item.title.is_match(&ret.title) && item.class.is_match(&ret.class) && item.path.is_match(&ret.path) {
                //println!("# XX IGNORELIST {}", ret.title);
                return empty;
            }
        }

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

fn read_config_file(filename: String) -> Value {
    // check and read config file
    let path = Path::new(&filename);
    let mut file = match File::open(&path) {
        Err(why) => panic!("ERROR: Couldn't open {}: {}", path.display(), &why),
        Ok(file) => file,
    };

    let mut contents = String::new();
    let _ = file.read_to_string(&mut contents);

    contents.parse::<Value>().unwrap()
}

fn parse_toml(val: &Value) -> Vec<Item> {
    let e = &std::vec::Vec::new();
    let entries = match val["WastedTime"]["ignorelist"].as_array() {
         Some(s) => s,
         _ => e,
    };
    let mut ignorelist = Vec::new();
    for entry in entries.iter() {
        let m = match entry.as_array() {
            Some(s) => s,
            _ => e,
        };
        if m.len() < 3 {
            continue;
        }
        let title = match m[0].as_str() {
            Some(s) => s,
            _ => "",
        };
        let class = match m[1].as_str() {
            Some(s) => s,
            _ => "",
        };
        let path = match m[2].as_str() {
            Some(s) => s,
            _ => "",
        };
        let name = "";
        let item = Item::new(title, class, path, name);
        ignorelist.push(item);
    }
    return ignorelist;
}