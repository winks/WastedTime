extern crate ctrlc;
extern crate directories;
extern crate kernel32;
extern crate rand;
extern crate regex;
extern crate rusqlite;
extern crate time;
extern crate toml;
extern crate user32;
extern crate winapi;

#[macro_use]
extern crate serde_json;

use directories::ProjectDirs;
use regex::Regex;
use rusqlite::{params, Connection};
use std::fs::{File, create_dir_all};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time as tm;
use toml::Value;

#[cfg(not(target_family = "windows"))]
use rand::Rng;
#[cfg(target_family = "windows")]
use kernel32::{CloseHandle, OpenProcess, K32GetModuleFileNameExW};
#[cfg(target_family = "windows")]
use user32::{GetForegroundWindow, GetWindowTextW, GetClassNameW, GetWindowThreadProcessId};

const PROCESS_QUERY_INFORMATION: u32 = 0x0400;
const PROCESS_VM_READ: u32 = 0x0010;

#[derive(Debug)]
struct Result {
    timestamp: time::Tm,
    title: String,
    class: String,
    pid: u32,
    path: String,
}

impl Result {
    pub fn new(title: String, class: String, path: String, pid: u32) -> Result{
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
            _ => name.to_string(),
        };
        Item{
            title: Regex::new(_title).unwrap(),
            class: Regex::new(_class).unwrap(),
            path: Regex::new(_path).unwrap(),
            name: _name,
        }
    }
}

fn out(r: &Result, last_change: i64, s: &str, is_debug: bool) {
    let diff = r.timestamp.to_timespec().sec - last_change;
    if is_debug {
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
    }
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

#[cfg(not(target_family = "windows"))]
fn get_info(ignorelist: &Vec<Item>, grouplist: &Vec<Item>, is_debug: bool) -> Result {
    let mut rnd = rand::thread_rng();
    let pid = &rnd.gen_range(0..10);
    Result::new("Title".to_string(), "Class".to_string(), "Path".to_string(), *pid)
}

#[cfg(target_family = "windows")]
fn get_info(ignorelist: &Vec<Item>, grouplist: &Vec<Item>, is_debug: bool) -> Result {
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
                if is_debug {
                    println!("#X IGNORELIST {}", ret.title);
                }
                return empty;
            }
        }
        for item in grouplist.iter() {
            if item.title.is_match(&ret.title) && item.class.is_match(&ret.class) && item.path.is_match(&ret.path) {
                if is_debug {
                    println!("#X GROUPLIST {}", ret.title);
                }
                return Result::new(item.name.clone(), String::new(), from_u16(&mod_name), pid);
            }
        }

        return ret
    }
}

#[cfg(not(target_family = "windows"))]
fn from_u16(s: &[u16]) -> String {
    "NYI".to_string()
}

#[cfg(target_family = "windows")]
fn from_u16(s: &[u16]) -> String {
    // panic if there's no null terminator
    let pos = s.iter().position(|a| a == &0u16).unwrap();
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;
    let s2: OsString = OsStringExt::from_wide(&s[..pos]);
    s2.to_string_lossy().to_string()
}

fn read_config_file(path: &Path) -> Value {
    let mut file = match File::open(&path) {
        Err(why) => {
            println!("ERROR: Couldn't open {}: {}", path.display(), &why);
            std::process::exit(1)
        },
        Ok(file) => file,
    };

    let mut contents = String::new();
    let _ = file.read_to_string(&mut contents);

    contents.parse::<Value>().unwrap()
}

fn parse_bool(val: &Value, key: &str) -> bool {
    let v1 = match val.get("WastedTime") {
        Some(v) => v,
        _ => {
            println!("Config file malformed.");
            std::process::exit(2)
        }
    };
    let v2 = match v1.get(key) {
        Some(v) => v,
        _ => {
            return false
        }
    };
    match v2.as_str() {
        Some(s) => {
            match s.to_lowercase().as_str() {
                "true" => true,
                _ => false
            }
        },
        None => false
    }
}

fn parse_toml(val: &Value, section: &str) -> Vec<Item> {
    let e = &std::vec::Vec::new();
    let v1 = match val.get("WastedTime") {
        Some(v) => v,
        _ => {
            println!("Config file malformed.");
            std::process::exit(2)
        }
    };
    let v2 = match v1.get(section) {
        Some(v) => v,
        _ => {
            println!("Config file malformed3.");
            std::process::exit(3)
        }
    };
    if !v2.is_array() {
        println!("Config file malformed.");
        std::process::exit(4)
    }
    let entries = match v2.as_array() {
         Some(s) => s,
         _ => e,
    };
    let mut itemlist = Vec::new();
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
        if m.len() < 4 && section == "grouplist" {
            continue;
        }
        let name = match m[3].as_str() {
            Some(s) => s,
            _ => "",
        };
        let item = Item::new(title, class, path, name);
        itemlist.push(item);
    }
    return itemlist
}

fn main() {
    let running = Arc::new(AtomicBool::new(true));
    let run = running.clone();
    ctrlc::set_handler(move || {
        run.store(false, Ordering::SeqCst);
    }).expect("Error setting Ctrl-C handler");

    let mut count = 0;
    let max = 0;
    let sleep_time = tm::Duration::from_millis(1000);

    let mut last = Result::empty();
    let mut last_change = 0;

    let cfg_path : PathBuf;
    let db_path : PathBuf;
    if let Some(proj_dirs) = ProjectDirs::from("org", "art-core",  "WastedTime") {
        let mut config_filename = "WastedTime.toml";
        let mut db_filename     = "WastedTime.sqlite";
        if cfg!(debug_assertions) {
            config_filename = "WastedTime.dev.toml";
            db_filename     = "WastedTime.dev.sqlite";
        }

        let config_dir = proj_dirs.config_dir();
        create_dir_all(config_dir).unwrap();
        cfg_path = config_dir.join(config_filename);

        let data_dir = proj_dirs.data_dir();
        create_dir_all(data_dir).unwrap();
        db_path = data_dir.join(db_filename);
    } else {
        panic!("foo");
    }

    let cfg = read_config_file(&cfg_path);
    let ignorelist = parse_toml(&cfg, "ignorelist");
    let grouplist = parse_toml(&cfg, "grouplist");
    let is_debug = parse_bool(&cfg, "debug");

    let conn = Connection::open(db_path).unwrap();

    while running.load(Ordering::SeqCst) {
        thread::sleep(sleep_time);
        let current = get_info(&ignorelist, &grouplist, is_debug);
        if current.pid == 0 {
            continue;
        }
        if current.pid != last.pid {
            if last.pid > 0 {
                out(&last, last_change, "E", is_debug);
            }
            let diff = last.timestamp.to_timespec().sec - last_change;
            conn.execute(
                "INSERT INTO log (time, timestamp, pid, class, path, title)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![diff, last.timestamp.to_timespec().sec, last.pid,
                        last.class, last.path, last.title],
            ).unwrap();
            last_change = current.timestamp.to_timespec().sec;
            out(&current, last_change, "S", is_debug);
        }
        count += 1;

        if count >= max && max > 0 {
            out(&last, last_change, "E", is_debug);
            break;
        }
        last = current;
    }
    println!("The end");
    let diff = last.timestamp.to_timespec().sec - last_change;
    conn.execute(
        "INSERT INTO log (time, timestamp, pid, class, path, title)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![diff, last.timestamp.to_timespec().sec, last.pid,
                last.class, last.path, last.title],
    ).unwrap();
}
