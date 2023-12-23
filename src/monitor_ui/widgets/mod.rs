mod probe_if_test;
mod graph_test;

use std::vec;

pub use probe_if_test::ProbeIfTest;
pub use graph_test::GraphTest;


mod widget_test;
pub use widget_test::widgetTest;
// ----------------------------------------------------------------------------
use eframe::egui::{self, Color32};
use egui_extras::{Column, TableBuilder};
use std::sync::Arc;

use crate::debugging_tools::VariableInfo;

pub struct Widget<'a>{
    pub id: u32,
    pub name: String,
    watch_list: Option<Arc<Vec<VariableInfo>>>,
    pub wiget_ui: Box<dyn WidgetApp<'a>>,
}

impl<'a> Widget<'a>{
    pub fn new(id:u32, name:String, wiget_ui: Box<dyn WidgetApp<'a>>) -> Self{
        Self{
            id,
            name,
            watch_list: None,
            wiget_ui,
        }
    }

    pub fn set_watch_list_ptr(&mut self, watch_list_ptr: &Vec<VariableInfo>){
        self.watch_list = Some(Arc::new(watch_list_ptr.clone()));
    }


    pub fn ui(&mut self, ctx: &egui::Context, ui: &mut egui::Ui){
        let mut _in_watch_list = Vec::new();

        if let Some(watch_list) = &mut self.watch_list {
            self.wiget_ui.fetch_watch_list(watch_list);

            for val in watch_list.iter(){
                _in_watch_list.push(val);
            }
        }
        
        egui::Window::new(&self.name).show(ctx, |ui| {
            self.wiget_ui.ui(ui);

            ui.separator();
            TableBuilder::new(ui)
                    .striped(true)
                    .resizable(true)
                    .vscroll(true)
                    .column(Column::auto().resizable(true))
                    .column(Column::auto().resizable(true))
                    .header(20.0, |mut header| {
                        header.col(|ui| {
                            ui.heading("Address");
                            ui.set_width(80.0);
                        });
                        header.col(|ui| {
                            ui.heading("Symbol");
                        });
                    })
                    .body(|mut body| {
                        for vals in &_in_watch_list {
                            body.row(18.0, |mut row| {
                                    row.col(|ui| {
                                        ui.label(&vals.address);
                                    });
                                    row.col(|ui| {
                                        ui.label(&vals.name);
                                    });
                                
                            });
                        }
                    });
            
        });
    }
}

pub trait WidgetApp<'a> {
    fn ui(&mut self, ui: &mut egui::Ui);
    fn fetch_watch_list(&mut self, watch_list: &Vec<crate::debugging_tools::VariableInfo>);
}