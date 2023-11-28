//use std::env;

//use std::intrinsics::size_of;
use std::process;
use std::path::PathBuf;
use std::io::{Write, BufReader, BufWriter, BufRead};
use std::io;
use std::error;
use std::fmt;
use std::convert::From;
use std::result;
use std::str;

use std:: thread;
use std::sync::{Arc, Mutex};

use regex::Regex;
//use futures::prelude::*;

//use dirs;

// $env:GDB_BINARY = "C:\ProgramData\chocolatey\bin\arm-none-eabi-gdb.exe"
//use gdb;
#[derive(Clone)]
pub struct GdbParser {
    stdin: Arc<Mutex<BufWriter<process::ChildStdin>>>,
    stdout: Arc<Mutex<BufReader<process::ChildStdout>>>,
    variable_list: Arc<Mutex<Vec<VariableList>>>,
    scan_prgress: Arc<Mutex<f64>>,
}

#[derive(Clone, Debug)] // Debugを追加
pub struct  VariableList{
    pub name :String,
    pub types:String,
    pub address:String,
}

#[derive(Debug)]
pub enum Error {
    IOError(std::io::Error),
    ParseError,
    IgnoredOutput
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
            _ => None
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
    pub fn launch(elffile: &PathBuf)-> Result<Self>{
        let name = ::std::env::var("GDB_ARM_BINARY").unwrap_or("arm-none-eabi-gdb".to_string());
        let mut child = process::Command::new(name)
            .args(&["--interpreter=mi"])
            .arg("-q")
            .arg(elffile)
            .stdout(process::Stdio::piped())
            .stdin(process::Stdio::piped())
            .stderr(process::Stdio::piped())
            .spawn()?;

            let mut result = GdbParser {
                stdin: Arc::new(Mutex::new(BufWriter::new(child.stdin.take().expect("broken stdin")))),
                stdout: Arc::new(Mutex::new(BufReader::new(child.stdout.take().expect("broken stdout")))),
                variable_list: Arc::new(Mutex::new(Vec::new())),
                scan_prgress: Arc::new(Mutex::new(0.0)),
            };
            
        result.read_sequence()?;
        result.set_options()?;
        Ok(result)
    }

    pub fn set_options(&mut self) -> Result<()>{
        let opt1 = "set print type methods off";
        let opt2 = "set print type typedefs off";
        
        self.send_cmd_raw(opt1)?;
        self.send_cmd_raw(opt2)?;
        Ok(())
    }

    pub fn scan_variables(&mut self) -> Result<Vec<VariableList>> {
        let mut _variable_list = Vec::new();
        let mut _vari_list = self.get_variable_list()?;

        let expanded_list = self.expand_symbol(_vari_list);
        for var in expanded_list {
            let var_type = self.get_variable_types(&var)?;

            if var_type.is_empty(){
                continue;
            }

            let var_address = self.get_variable_address(&var)?;
            _variable_list.push(VariableList {
                name: var,
                types: var_type.get(0).cloned().unwrap_or_default(),
                address: var_address.unwrap_or_default(),
            });
        }

        //self.variable_list = _variable_list.clone();
        let mut variable_list_guard = self.variable_list.lock().unwrap();
        *variable_list_guard = _variable_list.clone();
        Ok(_variable_list)
    }

    pub fn scan_variables_none_blocking_start(&mut self) -> std::thread::JoinHandle<Result<()>> {
        let mut _variable_list = Vec::new();
        let mut _vari_list = self.get_variable_list().unwrap();
    
        let expanded_list = self.expand_symbol(_vari_list);
        let mut now_num_of_list = 0;
    
        let expanded_list_len = expanded_list.len() as f64;
        let progress_clone = Arc::clone(&self.scan_prgress);
    
        let self_arc = Arc::new(Mutex::new(self.clone())); // ここを変更
    
        thread::spawn(move || {
            let mut self_lock = self_arc.lock().unwrap();
    
            for var in expanded_list {
                now_num_of_list += 1;
    
                let mut progress = progress_clone.lock().unwrap();
                *progress = now_num_of_list as f64 /expanded_list_len ;
                
                let var_type = self_lock.get_variable_types(&var).unwrap();
    
                if var_type.is_empty(){
                    continue;
                }
    
                let var_address = self_lock.get_variable_address(&var).unwrap();
                _variable_list.push(VariableList {
                    name: var,
                    types: var_type.get(0).cloned().unwrap_or_default(),
                    address: var_address.unwrap_or_default(),
                });
            }
            
            // self_lock.variable_list = Arc::new(Mutex::new(_variable_list.clone()));
            let mut variable_list_guard = self_lock.variable_list.lock().unwrap();
            *variable_list_guard = _variable_list.clone();
            Ok(())
        })
    }

    pub fn load_variable_list(&mut self) -> Vec<VariableList>{
        let variable_list_guard = self.variable_list.lock().unwrap();
        variable_list_guard.clone()
    }

    pub fn get_scan_progress(&mut self) -> f32{
        let progress = self.scan_prgress.lock().unwrap();
        *progress as f32
    }

    pub fn expand_symbol(&mut self, vari_list: Vec<String>) ->  Vec<String>{
        let mut _new_vari_list = Vec::new();
        let mut false_cnt = 0;

        for val in vari_list {
            if self.check_expanded(val.as_str()) {
                if let Ok(mut child_symbols) = self.get_variable_types(&val) {
                    let val_parts: Vec<&str> = val.split('.').collect();
                    child_symbols.retain(|child| !val_parts.contains(&child.as_str()));

                    let child_symbols = child_symbols.iter().map(|chid| format!("{}.{}", val, chid)).collect::<Vec<String>>();
                    _new_vari_list.append(&mut child_symbols.clone());
                    //println!("ex {}", &val);

                    false_cnt += 1;
                }else{
                    continue;
                }
            } else {
                _new_vari_list.push(val);
            }
        }

        if false_cnt != 0{
            _new_vari_list = self.expand_symbol(_new_vari_list);
        }

        _new_vari_list
    }

    pub fn get_variable_list(&mut self) -> Result<Vec<String>>{
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

    pub fn check_expanded(&mut self, symbol_name: &str) -> bool{
        let cmd = format!("ptype {}", symbol_name);

        let output_vec_str = self.send_cmd_raw(&cmd);
        match output_vec_str {
            Ok(vec) => vec.len() != 3,
            Err(_) => return true,
        }
    }
    
    // 変数名からアドレスを取得する
    pub fn get_variable_address(&mut self, symbol_name: &str) -> Result<Option<String>> {
        let cmd = format!("print /x &({})", symbol_name);

        let output_vec_str = self.send_cmd_raw(&cmd)?;
        let address = self.extract_variable_address(output_vec_str);
        Ok(address)
    }

    fn extract_variable_address(&mut self, input: Vec<String>) -> Option<String> {
        let re = Regex::new(r#"~"\$[0-9]+ = (0x[0-9a-fA-F]+)\\n"#).unwrap();
        
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

    
    pub fn get_variable_types(&mut self, symbol_name: &str) -> Result<Vec<String>>{
        let cmd = format!("ptype {}", symbol_name);

        let output_vec_str = self.send_cmd_raw(&cmd)?;
        if output_vec_str.is_empty() {
            return Ok(Vec::new());
        }
        let second_last_element = output_vec_str[output_vec_str.len() - 2].clone();
        if second_last_element.contains("*") {
            //println!("   {} is pointor !!", symbol_name);
            return Ok(Vec::new())
        }

        let mut vari_list = self.extract_variable_names(output_vec_str.clone());

        if vari_list.len() == 0{
            //vari_list.push(output_vec_str.clone());
            //print!("{:?}", output_vec_str.clone()[1]);

            let valtype = self.extract_variable_type(output_vec_str.clone());
            //print!("  --> {:?}", valtype);
            if let Some(valtype) = valtype {
                vari_list.push(valtype);
            }
        }
        Ok(vari_list)
    }
     

    fn extract_variable_type(&mut self, input: Vec<String>) -> Option<String>{
        let re = Regex::new(r#"~"type = ([^"]+)\\n"#).unwrap();

        for line in input {
            if let Some(caps) = re.captures(&line){
                match caps.get(1) {
                    Some(res) => return Some(res.as_str().to_string()),
                    None => continue,
                }
            }
        }
        None
    }

    fn send_cmd_raw(&mut self, cmd: &str) -> Result<Vec<String>>{
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
        while line != "(gdb) \n" && line != "(gdb) \r\n"{
            /*
            match parser::parse_line(line.as_str()) {
                Ok(resp) => result.push(resp),
                Err(err) => return Err(err),
            }
             */
            result.push(line.clone());

            line.clear();
            let _ = self.stdout.lock().unwrap().read_line(&mut line);
            //print!("read_sequence : {}", &line);
            //print!("{}", &line);
        }
        Ok(result)
    }
    
}

impl Drop for GdbParser {
    fn drop(&mut self) {
        let _ = self.stdin.lock().unwrap().write_all(b"-gdb-exit\n");
    }
}
