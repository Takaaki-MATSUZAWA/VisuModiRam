use ddbug_parser::{File, FileHash, Member, TypeKind, TypeModifierKind, TypeOffset, Variable};
use std::convert::From;
use std::error;
use std::fmt;
use std::io;
use std::path::PathBuf;
use std::result;
use std::str;
use std::sync::{Arc, Mutex};
use std::thread;

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

#[derive(Clone)]
pub struct ELFParser {
    elf_path: String,
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
            Error::ParseError => write!(f, "cannot parse response from ELF"),
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

// ----------------------------------------------------------------------------

impl ELFParser {
    pub fn launch(elffile: &PathBuf) -> Result<Self> {
        Ok(ELFParser {
            elf_path: elffile.to_str().unwrap().to_string(),
            variable_list: Arc::new(Mutex::new(Vec::new())),
            scan_prgress: Arc::new(Mutex::new(0.0)),
        })
    }

    pub fn scan_variables_none_blocking_start(&mut self) -> std::thread::JoinHandle<Result<()>> {
        let mut _variable_list = self.variable_list.clone();
        let elf_path = self.elf_path.to_string().clone();
        let progress_clone = Arc::clone(&self.scan_prgress);

        thread::spawn(move || {
            let variable_vec = File::parse(elf_path).and_then(|ctx| {
                let hash = FileHash::new(ctx.file()); // 検索用のハッシュを用意
                let mut variable_info_vec = Vec::new(); // 保存用のVec

                let prgress_step = 1.0 / ctx.file().units().len() as f64;

                // ユニットを取り出す
                for unit in ctx.file().units().iter() {
                    // 変数を取り出す
                    for var in unit.variables() {
                        // Addressが存在するものだけに絞る
                        if let Some(var_address) = var.address() {
                            // アドレスが0でなければ継続
                            if var_address != 0 {
                                // 変数の中身を再帰的に調べる（メンバー変数などもリストアップ）
                                variable_info_vec.extend(get_variable_info(var, &hash));
                            }
                        }
                    }
                    let mut progress = progress_clone.lock().unwrap();
                    *progress += prgress_step;
                    drop(progress);
                }
                Ok(variable_info_vec)
            });

            if let Ok(vars) = variable_vec {
                *_variable_list.lock().unwrap() = vars.clone();
                let mut progress = progress_clone.lock().unwrap();
                *progress = 1.0;
            }

            Ok(())
        })
    }

    pub fn load_variable_list(&mut self) -> Vec<VariableInfo> {
        let variable_list_guard = self.variable_list.lock().unwrap();
        variable_list_guard.clone()
    }

    pub fn get_scan_progress(&mut self) -> f32 {
        let progress = self.scan_prgress.lock().unwrap();
        *progress as f32
    }
}

// ----------------------------------------------------------------------

pub fn get_variable_info(var: &Variable, hash: &FileHash) -> Vec<VariableInfo> {
    let mut vars_vec = Vec::new();
    let typedata = var.ty(hash);
    match typedata.clone().unwrap().kind() {
        // ベース型の場合はそのままVecに追加
        TypeKind::Base(typeinfo) => {
            let info = VariableInfo {
                name: var.name().unwrap().to_string(),
                address: var.address().unwrap(),
                types: typeinfo.name().unwrap().to_string(),
                size: typeinfo.byte_size().unwrap() as usize,
            };
            vars_vec.push(info);
        }

        // メンバー変数を再帰的に探す
        TypeKind::Def(typeinfo) => {
            for member in typeinfo.ty(hash).clone().unwrap().members() {
                vars_vec.extend(get_member(
                    member,
                    hash,
                    var.name().unwrap().to_string(),
                    var.address().unwrap(),
                ));
            }
        }

        // メンバー変数を再帰的に探す
        TypeKind::Struct(typeinfo) => {
            for member in typeinfo.members() {
                vars_vec.extend(get_member(
                    member,
                    hash,
                    var.name().unwrap().to_string(),
                    var.address().unwrap(),
                ));
            }
        }
        _ => {} // 配列は後ほど実装を考える
    }

    vars_vec
}

pub fn get_member(
    member: &Member,
    hash: &FileHash,
    parent_name: String,
    parent_address: u64,
) -> Vec<VariableInfo> {
    let mut vars_vec = Vec::new();

    match member.ty(hash).unwrap().kind() {
        // ベース型の場合はそのままVecに追加
        TypeKind::Base(typeinfo) => {
            let member_name = format!("{}.{}", parent_name, member.name().unwrap());
            let member_address = parent_address + (member.bit_offset() / 8);
            let info = VariableInfo {
                name: member_name,
                address: member_address,
                types: typeinfo.name().unwrap().to_string(),
                size: typeinfo.byte_size().unwrap() as usize,
            };
            vars_vec.push(info);
        }

        TypeKind::Def(typeinfo) => {
            // メンバー変数がこれ以上ない場合はベース型が存在するかチェックして、あればVecに追加
            if typeinfo.ty(hash).unwrap().members().is_empty() {
                match typeinfo.ty(hash).unwrap().kind() {
                    TypeKind::Def(typeinfo) => {
                        let member_name = format!("{}.{}", parent_name, member.name().unwrap());
                        let member_address = parent_address + (member.bit_offset() / 8);
                        let info = VariableInfo {
                            name: member_name,
                            address: member_address,
                            types: get_base_type(typeinfo.ty(hash).unwrap().offset(), hash)
                                .unwrap(),
                            size: typeinfo.byte_size(hash).unwrap() as usize,
                        };
                        vars_vec.push(info);
                    }
                    _ => {}
                }
            } else {
                // メンバー変数が存在する場合、再帰的に探す
                let member_address = parent_address + (member.bit_offset() / 8);
                for nest_member in typeinfo.ty(hash).unwrap().members() {
                    vars_vec.extend(get_member(
                        nest_member,
                        &hash,
                        format!("{}.{}", parent_name, member.name().unwrap().to_string()),
                        member_address,
                    ));
                }
            }
        }
        // 構造体なので、メンバー変数をさらに調べる
        TypeKind::Struct(typeinfo) => {
            let member_address = parent_address + (member.bit_offset() / 8);
            for nest_member in typeinfo.members() {
                vars_vec.extend(get_member(
                    nest_member,
                    &hash,
                    format!("{}.{}", parent_name, member.name().unwrap().to_string()),
                    member_address,
                ));
            }
        }

        // 修飾子のある変数の場合は、型名に修飾子を付与する
        TypeKind::Modifier(typeinfo) => {
            let member_name = format!("{}.{}", parent_name, member.name().unwrap());
            let member_address = parent_address + (member.bit_offset() / 8);

            // ベース型がある場合
            if let Some(typename) = get_base_type(typeinfo.ty(hash).unwrap().offset(), hash) {
                // 修飾子の種類と表示名をマッピング
                let modifier = match typeinfo.kind() {
                    TypeModifierKind::Const => Some("const "),
                    TypeModifierKind::Volatile => Some("volatile "),
                    _ => None,
                };

                if let Some(modif) = modifier {
                    let info = VariableInfo {
                        name: member_name.clone(),
                        address: member_address,
                        types: format!("{}{}", modif, typename.clone()),
                        size: typeinfo.byte_size(hash).unwrap() as usize,
                    };

                    vars_vec.push(info);
                }
            }
        }
        _ => {}
    }

    vars_vec
}

// ベース型を調べる関数
pub fn get_base_type(type_offset: TypeOffset, hash: &FileHash) -> Option<String> {
    if let Some(typedata) = hash.types.get(&type_offset) {
        match typedata.kind().clone() {
            TypeKind::Base(typeinfo) => Some(typeinfo.name().unwrap().to_string()),
            TypeKind::Def(typeinfo) => get_base_type(typeinfo.ty(hash).unwrap().offset(), hash),
            _ => None, // その他の形式は必要に応じて実装していく予定
        }
    } else {
        None
    }
}

// ----------------------------------------------------------------------

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
                    if start_index >= name.len() {
                        return None;
                    }
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
