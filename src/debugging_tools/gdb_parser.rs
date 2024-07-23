//use std::env;

//use std::intrinsics::size_of;
use std::convert::From;
use std::error;
use std::fmt;
use std::io;
use std::io::{BufRead, BufReader, BufWriter, Write};
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;
use std::path::PathBuf;
use std::process;
use std::result;
use std::str;

use std::sync::{Arc, Mutex};
use std::thread;

use regex::Regex;

// ----------------------------------------------------------------------------
#[derive(Default, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct SelectableVariableInfo {
    pub name: String,
    pub types: String,
    pub address: u64,
    pub size: usize,
    pub is_selected: bool,
}

#[derive(Clone, Debug)] // Debugを追加
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct VariableInfo {
    pub name: String,
    pub types: String,
    pub address: u64,
    pub size: usize,
}

use std::cmp::Ordering;
impl PartialEq for VariableInfo {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.types == other.types
            && self.address == other.address
            && self.size == other.size
    }
}

impl Eq for VariableInfo {}

impl PartialOrd for VariableInfo {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.name.partial_cmp(&other.name)
    }
}

impl Ord for VariableInfo {
    fn cmp(&self, other: &Self) -> Ordering {
        self.name.cmp(&other.name)
    }
}
// ----------------------------------------------------------------------------
impl SelectableVariableInfo {
    pub fn generate(list: &Vec<VariableInfo>) -> Vec<SelectableVariableInfo> {
        let mut new_list = Vec::new();

        for vals in list {
            new_list.push(SelectableVariableInfo {
                name: vals.name.clone(),
                types: vals.types.clone(),
                address: vals.address.clone(),
                size: vals.size.clone(),
                is_selected: false,
            });
        }
        new_list
    }

    pub fn fetch(src: &Vec<VariableInfo>, dist: &mut Vec<SelectableVariableInfo>) {
        let mut new_list = Self::generate(src);
        let checked_list = Self::pick_selected(&dist);

        for new_item in &mut new_list {
            if checked_list
                .iter()
                .any(|checked_item| checked_item.name == new_item.name)
            {
                new_item.is_selected = true;
            }
        }
        dist.clear();
        dist.extend(new_list);
    }

    pub fn pick_selected(list: &Vec<SelectableVariableInfo>) -> Vec<VariableInfo> {
        let mut new_list = Vec::new();

        for val in list {
            if val.is_selected == true {
                new_list.push(VariableInfo {
                    name: val.name.clone(),
                    address: val.address.clone(),
                    types: val.types.clone(),
                    size: val.size.clone(),
                });
            }
        }
        new_list
    }
}
// ----------------------------------------------------------------------------
// $env:GDB_BINARY = "C:\ProgramData\chocolatey\bin\arm-none-eabi-gdb.exe"
#[derive(Clone)]
pub struct GdbParser {
    stdin: Arc<Mutex<BufWriter<process::ChildStdin>>>,
    stdout: Arc<Mutex<BufReader<process::ChildStdout>>>,
    variable_list: Arc<Mutex<Vec<VariableInfo>>>,
    scan_prgress: Arc<Mutex<f64>>,
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum Error {
    IOError(std::io::Error),
    ParseError,
    IgnoredOutput,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::IOError(ref err) => write!(f, "{}", err),
            Error::ParseError => write!(f, "cannot parse response from gdb"),
            Error::IgnoredOutput => write!(f, "ignored output"),
        }
    }
}

impl error::Error for Error {
    fn cause(&self) -> Option<&dyn error::Error> {
        match self {
            &Error::IOError(ref err) => Some(err),
            _ => None,
        }
    }
}

pub type Result<T> = result::Result<T, Error>;

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::IOError(err)
    }
}

impl GdbParser {
    pub fn launch(elffile: &PathBuf) -> Result<Self> {
        let name = ::std::env::var("GDB_ARM_BINARY").unwrap_or("arm-none-eabi-gdb".to_string());
        #[cfg(target_os = "windows")]
        let mut child = process::Command::new(name)
            .args(&["--interpreter=mi"])
            .arg("-q")
            .arg(elffile)
            .stdout(process::Stdio::piped())
            .stdin(process::Stdio::piped())
            .stderr(process::Stdio::piped())
            .creation_flags(0x08000000)
            .spawn()?;

        #[cfg(not(target_os = "windows"))]
        let mut child = process::Command::new(name)
            .args(&["--interpreter=mi"])
            .arg("-q")
            .arg(elffile)
            .stdout(process::Stdio::piped())
            .stdin(process::Stdio::piped())
            .stderr(process::Stdio::piped())
            .spawn()?;

        let mut result = GdbParser {
            stdin: Arc::new(Mutex::new(BufWriter::new(
                child.stdin.take().expect("broken stdin"),
            ))),
            stdout: Arc::new(Mutex::new(BufReader::new(
                child.stdout.take().expect("broken stdout"),
            ))),
            variable_list: Arc::new(Mutex::new(Vec::new())),
            scan_prgress: Arc::new(Mutex::new(0.0)),
        };

        result.read_sequence()?;
        result.set_options()?;
        Ok(result)
    }

    fn set_options(&mut self) -> Result<()> {
        let opt1 = "set print type methods off";
        let opt2 = "set print type typedefs off";

        self.send_cmd_raw(opt1)?;
        self.send_cmd_raw(opt2)?;
        Ok(())
    }

    pub fn scan_variables_none_blocking_start(&mut self) -> std::thread::JoinHandle<Result<()>> {
        let mut _variable_list = Vec::new();
        let mut _vari_list = self.get_variable_list().unwrap();

        let mut now_num_of_list = 0;

        let progress_clone = Arc::clone(&self.scan_prgress);

        let self_arc = Arc::new(Mutex::new(self.clone())); // ここを変更

        thread::spawn(move || {
            let mut self_lock = self_arc.lock().unwrap();

            let expanded_list = self_lock.expand_symbol(_vari_list);
            let expanded_list_len = expanded_list.len() as f64;

            for var in expanded_list {
                now_num_of_list += 1;

                let mut progress = progress_clone.lock().unwrap();
                *progress = now_num_of_list as f64 / expanded_list_len;
                drop(progress);

                let var_type = self_lock.get_variable_types(&var).unwrap();

                if var_type.is_empty() {
                    continue;
                }

                let var_size = self_lock.get_variable_size(&var).unwrap();

                let var_address = self_lock.get_variable_address(&var).unwrap();
                _variable_list.push(VariableInfo {
                    name: var,
                    types: var_type.get(0).cloned().unwrap_or_default(),
                    address: var_address.unwrap_or_default(),
                    size: var_size,
                });
            }

            // self_lock.variable_list = Arc::new(Mutex::new(_variable_list.clone()));
            let mut variable_list_guard = self_lock.variable_list.lock().unwrap();
            *variable_list_guard = _variable_list.clone();

            Ok(())
        })
    }

    fn get_variable_size(&mut self, var: &String) -> Result<usize> {
        let cmd = format!("p sizeof {}", var);
        let output = self.send_cmd_raw(&cmd)?;
        let size = self.extract_size(output);
        Ok(size)
    }

    fn extract_size(&mut self, input: Vec<String>) -> usize {
        let size_str = input[1]
            .split("= ")
            .last()
            .unwrap_or_default()
            .split("\\n")
            .next()
            .unwrap_or_default();
        //println!("{} --> {}", input[1], size_str);
        size_str.trim().parse::<usize>().unwrap_or_default()
    }

    pub fn load_variable_list(&mut self) -> Vec<VariableInfo> {
        let variable_list_guard = self.variable_list.lock().unwrap();
        variable_list_guard.clone()
    }

    pub fn get_scan_progress(&mut self) -> f32 {
        let progress = self.scan_prgress.lock().unwrap();
        *progress as f32
    }

    fn expand_symbol(&mut self, vari_list: Vec<String>) -> Vec<String> {
        let mut _new_vari_list = Vec::new();
        let mut false_cnt = 0;

        for val in vari_list {
            if self.check_expanded(val.as_str()) {
                if let Ok(mut child_symbols) = self.get_variable_types(&val) {
                    let val_parts: Vec<&str> = val.split('.').collect();
                    child_symbols.retain(|child| !val_parts.contains(&child.as_str()));

                    let child_symbols = child_symbols
                        .iter()
                        .map(|chid| format!("{}.{}", val, chid))
                        .collect::<Vec<String>>();
                    _new_vari_list.append(&mut child_symbols.clone());
                    //println!("ex {}", &val);

                    false_cnt += 1;
                } else {
                    continue;
                }
            } else {
                _new_vari_list.push(val);
            }
        }

        if false_cnt != 0 {
            _new_vari_list = self.expand_symbol(_new_vari_list);
        }

        _new_vari_list
    }

    fn get_variable_list(&mut self) -> Result<Vec<String>> {
        let cmd = "info variables";

        let output_vec_str = self.send_cmd_raw(&cmd)?;
        let vari_list = self.extract_variable_names(output_vec_str);
        Ok(vari_list)
    }

    fn extract_variable_names(&mut self, input: Vec<String>) -> Vec<String> {
        //let mut results = HashSet::new();
        let mut results = Vec::new();
        let re = Regex::new(r"(\w+)\[?\d*\]?;").unwrap();

        for line in input {
            if let Some(caps) = re.captures(&line) {
                if let Some(name) = caps.get(1) {
                    //results.insert(name.as_str().to_string());
                    results.push(name.as_str().to_string());
                    //println!("{}", name.as_str().to_string());
                }
            }
        }

        results.into_iter().collect()
    }

    fn check_expanded(&mut self, symbol_name: &str) -> bool {
        let cmd = format!("ptype {}", symbol_name);

        let output_vec_str = self.send_cmd_raw(&cmd);
        match output_vec_str {
            Ok(vec) => vec.len() != 3,
            Err(_) => return true,
        }
    }

    // 変数名からアドレスを取得する
    fn get_variable_address(&mut self, symbol_name: &str) -> Result<Option<u64>> {
        let cmd = format!("print /x &({})", symbol_name);

        let output_vec_str = self.send_cmd_raw(&cmd)?;
        let address = self.extract_variable_address(output_vec_str);
        Ok(address)
    }

    fn extract_variable_address(&mut self, input: Vec<String>) -> Option<u64> {
        let re = Regex::new(r#"~"\$[0-9]+ = (0x[0-9a-fA-F]+)\\n"#).unwrap();

        for line in input {
            if let Some(caps) = re.captures(&line) {
                match caps.get(1) {
                    Some(res) => {
                        let address_str = res.as_str().to_string();
                        let address = if address_str.starts_with("0x") {
                            u64::from_str_radix(&address_str[2..], 16)
                        } else {
                            address_str.parse::<u64>()
                        }
                        .expect("failed to parse watchlist variable address");
                        return Some(address);
                    }
                    None => continue,
                }
            }
        }
        None
    }

    fn get_variable_types(&mut self, symbol_name: &str) -> Result<Vec<String>> {
        let cmd = format!("ptype {}", symbol_name);

        let output_vec_str = self.send_cmd_raw(&cmd)?;
        if output_vec_str.is_empty() {
            return Ok(Vec::new());
        }

        // skip pointor
        if output_vec_str[output_vec_str.len() - 2].contains("*") {
            return Ok(Vec::new());
        }

        // skip enum
        if output_vec_str[1].contains(" enum ") {
            return Ok(Vec::new());
        }

        // skip const
        if output_vec_str[1].contains("const") {
            return Ok(Vec::new());
        }

        let mut vari_list = self.extract_variable_names(output_vec_str.clone());
        if vari_list.len() == 0 {
            if let Some(valtype) = self.extract_variable_type(output_vec_str.clone()) {
                vari_list.push(valtype);
            }
        }
        Ok(vari_list)
    }

    fn extract_variable_type(&mut self, input: Vec<String>) -> Option<String> {
        let re = Regex::new(r#"~"type = ([^"]+)\\n"#).unwrap();

        for line in input {
            if let Some(caps) = re.captures(&line) {
                match caps.get(1) {
                    Some(res) => return Some(res.as_str().to_string()),
                    None => continue,
                }
            }
        }
        None
    }

    fn send_cmd_raw(&mut self, cmd: &str) -> Result<Vec<String>> {
        if cmd.ends_with("\n") {
            write!(self.stdin.lock().unwrap(), "{}", cmd)?;
        } else {
            writeln!(self.stdin.lock().unwrap(), "{}", cmd)?;
        }
        self.stdin.lock().unwrap().flush()?;
        self.read_sequence()
    }

    fn read_sequence(&mut self) -> Result<Vec<String>> {
        let mut result = Vec::new();
        let mut line = String::new();
        self.stdout.lock().unwrap().read_line(&mut line)?;
        while line != "(gdb) \n" && line != "(gdb) \r\n" {
            result.push(line.clone());
            //print!("--read_sequence--  {}", line);
            if line == "" {
                break;
            }

            line.clear();
            self.stdout.lock().unwrap().read_line(&mut line)?;
        }
        Ok(result)
    }
}

impl Drop for GdbParser {
    fn drop(&mut self) {
        let _ = self.stdin.lock().unwrap().write_all(b"-gdb-exit\n");
    }
}

pub fn search_target_mcu_name(elf_file_path: &PathBuf) -> Option<String> {
    let project_name = elf_file_path.file_stem()?.to_str()?.to_string();
    let mut project_dir = elf_file_path.parent();

    // ELFを解析してMCUの名前を特定
    if let Some(mcu_id) = ddbug_parser::File::parse(elf_file_path.to_str().unwrap().to_string())
        .ok()
        .and_then(|ctx| {
            ctx.file()
                .units()
                .iter()
                .filter_map(|unit| {
                    let name = unit.name().unwrap();
                    let start_index = name.find("startup_").unwrap_or(0) + "startup_".len();
                    let trimmed_name = &name[start_index..];
                    let mcu_name = trimmed_name.replace(".s", "").to_uppercase();
                    mcu_name.into()
                })
                .find(|name| name.starts_with("STM32"))
        })
    {
        return Some(mcu_id);
    }

    while let Some(path) = project_dir {
        if path.file_name()?.to_str()? == project_name {
            break;
        }
        project_dir = path.parent();
    }

    // STM32Cube用のiocファイルからDeviceIdを特定
    let ioc_file_path = project_dir?.join(format!("{}.ioc", &project_name));
    if ioc_file_path.is_file() {
        let content = std::fs::read_to_string(&ioc_file_path).ok()?;
        for line in content.lines() {
            if line.starts_with("ProjectManager.DeviceId=") {
                return Some(line["ProjectManager.DeviceId=".len()..].to_string());
            }
        }
    }

    None
}
