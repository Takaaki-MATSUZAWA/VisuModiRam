use crate::debugging_tools::{ProbeInterface, VariableInfo};

#[derive(Default)]
pub struct ComResource {
    pub probe: Option<ProbeInterface>,
    pub target_mcu: String,
    pub watch_list: Vec<VariableInfo>,
}

impl ComResource {
    //pub fn setProbe(&mut self, )
}
