use super::elf_parser::VariableInfo;
use probe_rs::{Core, MemoryInterface};

pub struct MCUMemory {}

impl MCUMemory {
    pub fn write(
        core: &mut Core,
        symbol: &VariableInfo,
        value_str: &str,
    ) -> Result<(), probe_rs::Error> {
        let c_type = symbol.types.as_str();
        let c_type = c_type.strip_prefix("volatile ").unwrap_or(&c_type);
        let c_type = c_type.strip_suffix(" [").unwrap_or(c_type);
        match c_type {
            "signed char" | "char" => match value_str.parse::<i8>() {
                Ok(val) => core.write_word_8(symbol.address, val as u8),
                Err(_) => Err(probe_rs::Error::Other(anyhow::anyhow!(
                    "Parse error for signed char"
                ))),
            },
            "unsigned char" | "bool" | "_Bool" => match value_str.parse::<u8>() {
                Ok(val) => core.write_word_8(symbol.address, val),
                Err(_) => Err(probe_rs::Error::Other(anyhow::anyhow!(
                    "Parse error for unsigned char"
                ))),
            },
            "short int" => match value_str.parse::<i16>() {
                Ok(val) => {
                    let block = val.to_le_bytes();
                    core.write_8(symbol.address, &block).map_err(|e| e.into())
                }
                Err(_) => Err(probe_rs::Error::Other(anyhow::anyhow!(
                    "Parse error for short"
                ))),
            },
            "short unsigned int" => match value_str.parse::<u16>() {
                Ok(val) => {
                    let block = val.to_le_bytes();
                    core.write_8(symbol.address, &block).map_err(|e| e.into())
                }
                Err(_) => Err(probe_rs::Error::Other(anyhow::anyhow!(
                    "Parse error for unsigned short"
                ))),
            },
            "int" | "long int" => match value_str.parse::<i32>() {
                Ok(val) => core.write_word_32(symbol.address, val as u32),
                Err(_) => Err(probe_rs::Error::Other(anyhow::anyhow!(
                    "Parse error for int/long"
                ))),
            },
            "unsigned int" | "long unsigned int" => match value_str.parse::<u32>() {
                Ok(val) => core.write_word_32(symbol.address, val),
                Err(_) => Err(probe_rs::Error::Other(anyhow::anyhow!(
                    "Parse error for unsigned int/long"
                ))),
            },
            "long long int" => match value_str.parse::<i64>() {
                Ok(val) => {
                    let block = val.to_le_bytes();
                    let block_u32 = [
                        u32::from_le_bytes([block[0], block[1], block[2], block[3]]),
                        u32::from_le_bytes([block[4], block[5], block[6], block[7]]),
                    ];
                    core.write_32(symbol.address, &block_u32)
                        .map_err(|e| e.into())
                }
                Err(_) => Err(probe_rs::Error::Other(anyhow::anyhow!(
                    "Parse error for long long"
                ))),
            },
            "long long unsigned int" => match value_str.parse::<u64>() {
                Ok(val) => {
                    let block = val.to_le_bytes();
                    let block_u32 = [
                        u32::from_le_bytes([block[0], block[1], block[2], block[3]]),
                        u32::from_le_bytes([block[4], block[5], block[6], block[7]]),
                    ];
                    core.write_32(symbol.address, &block_u32)
                        .map_err(|e| e.into())
                }
                Err(_) => Err(probe_rs::Error::Other(anyhow::anyhow!(
                    "Parse error for unsigned long long"
                ))),
            },
            "float" => match value_str.parse::<f32>() {
                Ok(val) => core.write_word_32(symbol.address, val.to_bits()),
                Err(_) => Err(probe_rs::Error::Other(anyhow::anyhow!(
                    "Parse error for float"
                ))),
            },
            "double" | "long double" => match value_str.parse::<f64>() {
                Ok(val) => core.write_word_64(symbol.address, val.to_bits()),
                Err(_) => Err(probe_rs::Error::Other(anyhow::anyhow!(
                    "Parse error for double/long double"
                ))),
            },
            _ => Err(probe_rs::Error::Other(anyhow::anyhow!("Unsupported type"))),
        }
    }

    pub fn read(core: &mut Core, symbol: &VariableInfo) -> String {
        let c_type = symbol.types.as_str();
        let c_type = c_type.strip_prefix("volatile ").unwrap_or(&c_type);
        let c_type = c_type.strip_suffix(" [").unwrap_or(c_type);
        let val_str = {
            match c_type {
                "char" | "signed char" => {
                    let val_bits = core
                        .read_word_8(symbol.address)
                        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
                        .unwrap();
                    format!("{}", val_bits as i8)
                }
                "unsigned char" | "bool" | "_Bool" => {
                    let val_bits = core
                        .read_word_8(symbol.address)
                        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
                        .unwrap();
                    format!("{}", val_bits)
                }
                "short int" => {
                    let mut buff = [0u8; 2];
                    core.read_8(symbol.address, &mut buff)
                        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
                        .unwrap();
                    let val_bits = u16::from_le_bytes(buff);
                    format!("{}", val_bits as i16)
                }
                "short unsigned int" => {
                    let mut buff = [0u8; 2];
                    core.read_8(symbol.address, &mut buff)
                        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
                        .unwrap();
                    let val_bits = u16::from_le_bytes(buff);
                    format!("{}", val_bits)
                }
                "int" | "long int" => {
                    let val_bits = core
                        .read_word_32(symbol.address)
                        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
                        .unwrap();
                    format!("{}", val_bits as i32)
                }
                "unsigned int" | "long unsigned int" => {
                    let val_bits = core
                        .read_word_32(symbol.address)
                        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
                        .unwrap();
                    format!("{}", val_bits)
                }
                "long long int" => {
                    let val_bits = core
                        .read_word_64(symbol.address)
                        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
                        .unwrap();
                    format!("{}", val_bits as i64)
                }
                "long long unsigned int" => {
                    let val_bits = core
                        .read_word_64(symbol.address)
                        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
                        .unwrap();
                    format!("{}", val_bits)
                }
                "float" => {
                    let val_bits = core
                        .read_word_32(symbol.address)
                        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
                        .unwrap();
                    format!("{:?}", f32::from_bits(val_bits))
                }
                "double" | "long double" => {
                    // long double cast to double
                    let val_bits = core
                        .read_word_64(symbol.address)
                        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
                        .unwrap();
                    format!("{:?}", f64::from_bits(val_bits))
                }
                _ => format!(""),
            }
        };
        val_str
    }
}
