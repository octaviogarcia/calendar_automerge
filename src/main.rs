extern crate automerge;
extern crate num_traits;
extern crate num_integer;

use eframe::egui;
use eframe::egui::RichText;
use chrono::prelude::*;

/*
use crate::automerge::transaction::Transactable;
use std::path::Path;
const DOC_FILE: &str = "doc.am";*/

fn main() {
  /*let mut doc;
  if Path::new(DOC_FILE).exists(){
    println!("File exists!");
    doc = automerge::AutoCommit::load(&std::fs::read(DOC_FILE).unwrap()).unwrap();
  }
  else{
    println!("Making new file");
    doc = automerge::AutoCommit::new();
    doc.set_actor(automerge::ActorId::from(0u128.to_be_bytes()));
    doc.put(&automerge::ROOT,"test_key","test_value").unwrap();
    let test_key_id = doc.put_object(&automerge::ROOT,"test_key",automerge::ObjType::Map).unwrap();
    doc.put(&test_key_id,"hello","world").unwrap();

    let mut another_doc = automerge::AutoCommit::new();
    another_doc.set_actor(automerge::ActorId::from(1u128.to_be_bytes()));
    another_doc.put_object(&automerge::ROOT,"test_list",automerge::ObjType::List).unwrap();
    doc.merge(&mut another_doc).unwrap();
    std::fs::write(DOC_FILE,doc.save()).unwrap();
  }*/
  //println!("doc {:#?}",doc);
  let native_options = eframe::NativeOptions::default();
  eframe::run_native("Calendar-Automerge", native_options, Box::new(|cc| Box::new(CalendarAutomergeApp::new(cc))));
}

#[derive(Default)]
struct CalendarAutomergeApp {
  pub appointment_window_data: AppointmentWindowData,
  //Main window appointments
  pub appointments: Vec<Appointment>,
}
#[derive(Default)]
struct AppointmentWindowData {
  pub open: bool,
  pub editing: Option<usize>,
  pub month: u32,
  pub year: i32,
  pub selected_y: i32,
  pub selected_m: u32,
  pub selected_d: u32,
  pub hour: u32,
  pub minute: u32,
  pub d_hour: u32,
  pub d_minute: u32,
  pub new_text: String,
}

struct Appointment {
  pub init: i64,//Timestamp
  pub end: i64,//Timestamp
  pub text: String,
}

impl CalendarAutomergeApp {
  fn new(_cc: &eframe::CreationContext<'_>) -> Self {
    // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
    // Restore app state using cc.storage (requires the "persistence" feature).
    // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
    // for e.g. egui::PaintCallback.
    let mut app = Self::default();
    app.appointment_window_set_data(None);
    app.appointment_window_data.open = false;
    return app;
  }
  fn appointment_window_set_data(&mut self,a_idx: Option<usize>){
    let init: i64;
    let end: i64;
    let text: String;
    
    match a_idx {
      None => {
        init  = Local::now().timestamp();
        end   = init + 1*60*60;
        text  = String::new();
      }
      Some(idx) => {
        init  = self.appointments[idx].init;
        end   = self.appointments[idx].end;
        text  = String::from(self.appointments[idx].text.as_str());
      }
    }
  
    let date = chrono::NaiveDateTime::from_timestamp(init,0);
    let awd = &mut self.appointment_window_data;//alias
    awd.editing  = a_idx;
    awd.new_text = text;
    awd.month    = date.month();
    awd.year     = date.year();
    awd.hour     = date.hour();
    awd.minute   = date.minute();
    let diff = (end - init).abs();
    awd.d_minute   = (diff/60) as u32;
    awd.d_hour     = awd.d_minute/60;
    awd.d_minute  -= awd.d_hour*60;
    awd.selected_y = awd.year;
    awd.selected_m = awd.month;
    awd.selected_d = date.day();
  }
}

impl eframe::App for CalendarAutomergeApp {
  fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
    let app = self;
    if app.appointment_window_data.open { 
      egui::Window::new(
        if app.appointment_window_data.editing.is_none() { "New Appointment" } else { "Editing Appointment" }
      ).show(ctx,|ui| {
        ui.vertical(|ui|{
          appointment_window_ui(ui,app);
        });
      });
    }
    egui::CentralPanel::default().show(ctx, |ui| {
      let height = ui.available_size().y;
      ui.horizontal(|ui|{
        ui.set_max_height(height);//Why do I have to do this
        ui.vertical(|ui|{
          if ui.button("Add appointment").clicked(){
            app.appointment_window_set_data(None);
            app.appointment_window_data.open = true;
          }
          ui.heading("0.0.1 prealpha");
        });
        ui.vertical(|ui|{
          egui::ScrollArea::vertical().show(ui,|ui|{
            appointments_list_ui(ui,app);
          });
        });
      });
   });
  }
}

fn appointment_window_ui(ui: &mut egui::Ui,app: &mut CalendarAutomergeApp){
  let awd = &mut app.appointment_window_data;//alias
  awd.year  = ui_counter(ui,"Year",awd.year,0,i32::MAX-1,true);
  awd.month = ui_counter(ui,"Month",awd.month,1,12,true);
  ui.heading(format!("Selected {:0>4}-{:0>2}-{:0>2}",
    awd.selected_y,
    awd.selected_m,
    awd.selected_d
  ));
  for w in 0..5{//Weeks
    ui.horizontal(|ui| {
      for wd in 1..=7{//Week day
        let day = w*7+wd;
        if day > days_from_month(awd.year,awd.month){
          ui.label("");
        }
        else{
          let lpad = if day < 10 { " " } else { "" };
          let daystr = lpad.to_owned() + &day.to_string();
          let is_selected_label = 
             awd.selected_y == awd.year
          && awd.selected_m == awd.month
          && awd.selected_d == day;
          let dayclicked = ui.selectable_label(is_selected_label,RichText::new(daystr).monospace()).clicked();
          if dayclicked {
            awd.selected_y = awd.year;
            awd.selected_m = awd.month;
            awd.selected_d = day;
          }
        }
      }
    });
  }
  ui.horizontal(|ui| {
    awd.hour   = ui_counter(ui,"h:",awd.hour,0,23,true);
    awd.minute = ui_counter(ui,"m:",awd.minute,0,59,true);
  });
  ui.heading("Duration");
  ui.horizontal(|ui| {
    awd.d_hour   = ui_counter(ui,"h:",awd.d_hour,0,23,true);
    awd.d_minute = ui_counter(ui,"m:",awd.d_minute,0,59,true);
  });
  ui.horizontal(|ui|{
    match awd.editing {
      None => {
        if ui.button("Add").clicked(){
          let init = chrono::Utc.ymd(
            awd.selected_y,
            awd.selected_m,
            awd.selected_d
          ).and_hms_milli(awd.hour,awd.minute,0,0).timestamp();
          let end  = init + ((awd.d_minute + awd.d_hour * 60) * 60) as i64;
          //@Slow, if we keep the vec sorted from the start we can push and swap() to the correct position unless its equal to another value
          app.appointments.push(Appointment{init: init,end: end,text: String::from(awd.new_text.as_str())});
          app.appointments.sort_by(|a,b| a.init.partial_cmp(&b.init).unwrap());
          awd.open = false;
        }
      }
      Some(idx) => {
        if ui.button("Save").clicked(){
          let init = chrono::Utc.ymd(
            awd.selected_y,
            awd.selected_m,
            awd.selected_d
          ).and_hms_milli(awd.hour,awd.minute,0,0).timestamp();
          let end  = init + ((awd.d_minute + awd.d_hour * 60) * 60) as i64;
          app.appointments[idx].init = init;
          app.appointments[idx].end  = end;
          app.appointments[idx].text = String::from(awd.new_text.as_str());
          app.appointments.sort_by(|a,b| a.init.partial_cmp(&b.init).unwrap());
          awd.editing = None;
          awd.open = false;
        }
      }
    }
    if ui.button("Close").clicked() {
      awd.open = false;
    }
  });
  let mut note_size = ui.available_size();
  note_size.y = 200.;
  ui.add_sized(note_size,egui::TextEdit::multiline(&mut awd.new_text));
}

fn appointments_list_ui(ui: &mut egui::Ui,app: &mut CalendarAutomergeApp){
  ui.heading("Appointments");
  let mut for_deletion = Vec::<usize>::with_capacity(app.appointments.len());
  let mut edit_appointment: Option<usize> = None;
  for (idx,a) in app.appointments.iter_mut().enumerate(){
    let init_appointment = chrono::NaiveDateTime::from_timestamp(a.init,0);//@SPEED Save this to avoid recreating in each frame
    let end_appointment = chrono::NaiveDateTime::from_timestamp(a.end,0);
    ui.horizontal(|ui|{
      ui.heading(init_appointment.to_string()+" - "+&end_appointment.to_string());
      if ui.button("Edit").clicked(){
        edit_appointment = Some(idx);
      }
      if ui.button("Delete").clicked(){
        for_deletion.push(idx);
      }
    });
    ui.label(&a.text);
  }
  if edit_appointment.is_some() {
    app.appointment_window_set_data(edit_appointment);
    app.appointment_window_data.open = true;
  }
  for idx in for_deletion.iter().rev(){//@SLOW Delete by reverse to mitigate O(n) remove
    app.appointments.remove(*idx);
  }
}

fn ui_counter<T: ToString + num_integer::Integer + num_traits::Euclid + Copy>(ui: &mut egui::Ui,label: &str, counter: T, min: T, max: T,cycle: bool) -> T {
  return ui.horizontal(|ui| {
    ui.heading(label);
    let mut ret = counter;
    if ui.button("-").clicked() && (counter > min || cycle){
      ret = if ret == min {
        max
      }
      else {
        ret - num_traits::One::one()
      }
    }
    ui.label(counter.to_string());
    if ui.button("+").clicked() && (counter < max || cycle){
      ret = if ret == max {
        min
      }
      else {
        ret + num_traits::One::one()
      }
    }
    return ret;
  }).inner;
}

fn days_from_month(year: i32,month: u32) -> u32 {
  let firstday = chrono::Utc.ymd(year,month,1).and_hms_milli(0,0,0,0);
  let is_december = month == 12;
  let year = if is_december { year+1 } else { year };
  let month = if is_december { 1 } else { month+1 };
  let firstday_nextmonth = chrono::Utc.ymd(year,month,1).and_hms_milli(0,0,0,0);
  return firstday_nextmonth.signed_duration_since(firstday).num_days().abs() as u32;//cast i32 to u32
}