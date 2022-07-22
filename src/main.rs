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
  pub awd: AppointmentWindowData,
  pub awd_open: bool,
  pub awd_editing_idx: Option<usize>,//Index to appointments
  //Main window appointments
  pub appointments: Vec<Appointment>,
  pub tasks: Vec<Task>,
  pub done_tasks: Vec<Task>,
}

#[derive(Default)]
struct AppointmentWindowData {
  pub month: u32,
  pub year: i32,
  pub selected_y: i32,
  pub selected_m: u32,
  pub selected_d: u32,
  pub hour: u32,
  pub minute: u32,
  pub d_hour: u32,
  pub d_minute: u32,
  pub text: String,
  //These are task specific fields
  pub force_regularity: bool,//if we force everything to regular, disable years and force months to be 30 days
  pub is_task: bool,
  pub repeat_y:  i64,
  pub repeat_mo: i64,
  pub repeat_w:  i64,
  pub repeat_d:  i64,
  pub repeat_h:  i64,
  pub repeat_mi: i64,
  pub repeat_s:  i64,
}

enum AppointmentWindowResult {
  Open,
  Closed,
  SavedAppointment(Appointment),
  SavedTask(Task),
}

#[derive(Default)]
struct Appointment {
  pub init: i64,//Timestamp
  pub end: i64,//Timestamp
  pub text: String,
}

#[derive(Default)]
struct Task {
  pub task: Appointment,//A task is conceptually the same except we repeat it
  pub repeat_period: RepeatPeriod,
}

impl CalendarAutomergeApp {
  fn new(_cc: &eframe::CreationContext<'_>) -> Self {
    // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
    // Restore app state using cc.storage (requires the "persistence" feature).
    // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
    // for e.g. egui::PaintCallback.
    return Self::default();
  }
  fn appointment_window_set_data(&mut self,a_idx: Option<usize>,is_task: bool){
    self.awd_editing_idx = a_idx;
    
    let init: i64;
    let end: i64;
    let text: String;
    let repeat_period: RepeatPeriod;
    
    match a_idx {
      None => {
        init  = Local::now().timestamp();
        end   = init + 1*60*60;
        text  = String::new();
        repeat_period = RepeatPeriod::default();
      }
      Some(idx) => {
        if is_task {
          init  = self.tasks[idx].task.init;
          end   = self.tasks[idx].task.end;
          text  = String::from(self.tasks[idx].task.text.as_str());
          repeat_period = self.tasks[idx].repeat_period;
        }
        else{
          init  = self.appointments[idx].init;
          end   = self.appointments[idx].end;
          text  = String::from(self.appointments[idx].text.as_str());
          repeat_period = RepeatPeriod::default();
        }
      }
    }
  
    let date = chrono::NaiveDateTime::from_timestamp(init,0);
    let awd = &mut self.awd;//alias
    awd.text   = text;
    awd.month  = date.month();
    awd.year   = date.year();
    awd.hour   = date.hour();
    awd.minute = date.minute();
    let diff = (end - init).abs();
    awd.d_minute   = (diff/60) as u32;
    awd.d_hour     = awd.d_minute/60;
    awd.d_minute  -= awd.d_hour*60;
    awd.selected_y = awd.year;
    awd.selected_m = awd.month;
    awd.selected_d = date.day();
    
    awd.is_task = is_task;
    awd.repeat_s  = repeat_period.regular.seconds;
    awd.repeat_mi = awd.repeat_s/60;
    awd.repeat_h  = awd.repeat_mi/60;
    awd.repeat_d  = awd.repeat_h/24;
    awd.repeat_w  = awd.repeat_d/7;
    awd.repeat_d  %=7;
    awd.repeat_h  %=24;
    awd.repeat_mi %=60;
    awd.repeat_s  %=60;
    
    awd.repeat_mo = repeat_period.irregular.months;
    awd.repeat_y  = repeat_period.irregular.months/12;
    awd.repeat_mo %= 12;
    
    awd.force_regularity = awd.repeat_mo == 0 && awd.repeat_y == 0 && awd.repeat_w > 3;
    if awd.force_regularity {
      awd.repeat_mo = awd.repeat_w/4;
      awd.repeat_y  = awd.repeat_mo/12;
      awd.repeat_w  %= 4;
      awd.repeat_mo %= 12;
    }
  }
}

impl eframe::App for CalendarAutomergeApp {
  fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
    let app = self;
    //Main window
    egui::CentralPanel::default().show(ctx, |ui| {
      let w_width  = ui.available_size().x;
      let w_height = ui.available_size().y;
      let vpadding = 10.0;
      let column_size = 2.0*w_width/5.0 - vpadding;
      ui.horizontal(|ui|{
        ui.set_max_height(w_height);//Why do I have to do this
        ui.vertical(|ui|{
          ui.set_width(w_width - 2.0*column_size);
          if ui.button_enabled(!app.awd_open,"Add appointment").clicked(){
            app.appointment_window_set_data(None,false);
            app.awd_open = true;
          }
          if ui.button_enabled(!app.awd_open,"Add task").clicked(){
            app.appointment_window_set_data(None,true);
            app.awd_open = true;
          }
          ui.heading("0.0.1 prealpha");
        });
        ui.separator();
        ui.vertical(|ui|{
          ui.set_width(column_size);
          ui.label("Day/Week/Month viewer");
        });
        ui.separator();
        ui.vertical(|ui|{
          ui.vertical(|ui|{
            ui.set_height(w_height/2.0);
            ui.set_width(column_size);
            egui::ScrollArea::vertical().id_source("Appointments").show(ui,|ui|{
              appointments_list_ui(ui,app);
            });
          });
          ui.separator_spacing(0.5);
          ui.separator_spacing(0.5);
          ui.vertical(|ui|{
            ui.set_height(w_height/2.0);
            ui.set_width(column_size);
            egui::ScrollArea::vertical().id_source("Tasks").show(ui,|ui|{
              tasks_list_ui(ui,app);
            });
          });
        });
      });
    });
    
    //Appointment/Task window
    if app.awd_open { 
      let window_title = if app.awd_editing_idx.is_none() { "New Appointment/Task" } else { "Editing Appointment/Task" };
      egui::Window::new(window_title).show(ctx,|ui| {
        ui.vertical(|ui|{
          let result = appointment_window_ui(ui,&mut app.awd);
          match result {
            AppointmentWindowResult::Open => {},
            AppointmentWindowResult::Closed => {
              app.awd_open        = false;
            },
            AppointmentWindowResult::SavedAppointment(appointment) => {
              match app.awd_editing_idx {
                None => {
                  app.appointments.push(appointment);
                }
                Some(idx) => {
                  app.appointments[idx] = appointment
                }
              }
              app.appointments.sort_by(|a,b| a.init.partial_cmp(&b.init).unwrap());
              app.awd_open        = false;
            },
            AppointmentWindowResult::SavedTask(task) => {
              match app.awd_editing_idx {
                None => {
                  app.tasks.push(task);
                }
                Some(idx) => {
                  app.tasks[idx] = task;
                }
              }
              app.tasks.sort_by(|a,b| a.task.init.partial_cmp(&b.task.init).unwrap());
              app.awd_open        = false;
            },
          }
        });
      });
    }
  }
}
fn appointment_window_ui(ui: &mut egui::Ui,awd: &mut AppointmentWindowData) -> AppointmentWindowResult{
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
  
  if awd.is_task{
    ui.heading("Repeat");
    ui.checkbox(&mut awd.force_regularity,"Normalized (1 year = 365 days, 1 month = 30 days)");
    ui.horizontal(|ui|{
      awd.repeat_y  = ui_counter(ui,"Years:",awd.repeat_y ,0,10,true);
      awd.repeat_mo = ui_counter(ui,"Months:",awd.repeat_mo,0,12,true);
    });
    ui.horizontal(|ui|{
      awd.repeat_w  = ui_counter(ui,"Weeks:",awd.repeat_w ,0,3,true);
      awd.repeat_d  = ui_counter(ui,"Days:",awd.repeat_d ,0,6,true);
      awd.repeat_h  = ui_counter(ui,"Hours:",awd.repeat_h,0,23,true);
      awd.repeat_mi = ui_counter(ui,"Minutes:",awd.repeat_mi,0,59,true);
      awd.repeat_s  = ui_counter(ui,"Seconds:",awd.repeat_s ,0,59,true);
    });
  }
  
  let mut note_size = ui.available_size();
  note_size.y = 200.;//Pixels, make this font size dependent?
  ui.add_sized(note_size,egui::TextEdit::multiline(&mut awd.text));
  
  let mut result = AppointmentWindowResult::Open;
  ui.horizontal(|ui|{
    if ui.button("Save").clicked(){
      let init = chrono::Utc.ymd(
        awd.selected_y,
        awd.selected_m,
        awd.selected_d
      ).and_hms_milli(awd.hour,awd.minute,0,0).timestamp();
      let end  = init + ((awd.d_minute + awd.d_hour * 60) * 60) as i64;
      let text = String::from(awd.text.as_str()); 
      let appointment = Appointment{init: init,end: end,text: text};
      result = if awd.is_task {
        let repeat_period = RepeatPeriod::new(awd.force_regularity,awd.repeat_y,awd.repeat_mo,awd.repeat_w,awd.repeat_d,awd.repeat_h,awd.repeat_mi,awd.repeat_s);
        AppointmentWindowResult::SavedTask(Task{task: appointment,repeat_period: repeat_period})
      }
      else{
        AppointmentWindowResult::SavedAppointment(appointment)
      };
    }
    if ui.button("Close").clicked() {
      result = AppointmentWindowResult::Closed;
    }
  });
  
  return result;
}

fn appointments_list_ui(ui: &mut egui::Ui,app: &mut CalendarAutomergeApp){
  ui.heading("Appointments");
  ui.separator();
  let mut for_deletion = Vec::<usize>::with_capacity(app.appointments.len());
  let mut edit_appointment: Option<usize> = None;
  for (idx,a) in app.appointments.iter_mut().enumerate(){
    let init_appointment = chrono::NaiveDateTime::from_timestamp(a.init,0);//@SPEED Save this to avoid recreating in each frame
    let end_appointment = chrono::NaiveDateTime::from_timestamp(a.end,0);
    ui.horizontal_wrapped(|ui|{
      ui.add(egui::Label::new(init_appointment.to_string()+" - "+&end_appointment.to_string()).wrap(true));
      if ui.button_enabled(!app.awd_open,"Edit").clicked(){
        edit_appointment = Some(idx);
      }
      if ui.button_enabled(!app.awd_open,"Delete").clicked(){
        for_deletion.push(idx);
      }
    });
    ui.label(&a.text);
    ui.separator();
  }
  if edit_appointment.is_some() {
    app.appointment_window_set_data(edit_appointment,false);
    app.awd_open = true;
  }
  for idx in for_deletion.iter().rev(){//@SLOW Delete by reverse to mitigate O(n) remove
    app.appointments.remove(*idx);
  }
}

fn tasks_list_ui(ui: &mut egui::Ui,app: &mut CalendarAutomergeApp){
  ui.heading("Tasks");
  ui.separator();
  let mut for_deletion = Vec::<usize>::with_capacity(app.tasks.len());
  let mut edit_task: Option<usize> = None;
  for (idx,t) in app.tasks.iter_mut().enumerate(){
    let init = chrono::NaiveDateTime::from_timestamp(t.task.init.max(i32::MIN as i64).min(i32::MAX as i64),0);//@SPEED Save this to avoid recreating in each frame
    let end  = chrono::NaiveDateTime::from_timestamp(t.task.end.max(i32::MIN as i64).min(i32::MAX as i64),0);
    ui.horizontal_wrapped(|ui|{
      ui.add(egui::Label::new(init.to_string()+" - "+&end.to_string()+" - "+&t.repeat_period.to_string()).wrap(true));
      if ui.button_enabled(false,"Edit").clicked(){
        edit_task = Some(idx);
      }
      if ui.button_enabled(false,"Delete").clicked(){
        for_deletion.push(idx);
      }
    });
    ui.label(&t.task.text);
    ui.separator();
  }
  if edit_task.is_some() {
    app.appointment_window_set_data(edit_task,true);
    app.awd_open = true;
  }
  for idx in for_deletion.iter().rev(){//@SLOW Delete by reverse to mitigate O(n) remove
    app.tasks.remove(*idx);
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

trait CustomUiShortcuts {
  fn button_enabled(&mut self,enabled: bool,text: impl Into<egui::WidgetText>) -> egui::Response;
  fn separator_spacing(&mut self,spacing: f32) -> egui::Response;
}
impl CustomUiShortcuts for egui::Ui {
  fn button_enabled(&mut self,enabled: bool,text: impl Into<egui::WidgetText>) -> egui::Response {
    self.add_enabled(enabled,egui::Button::new(text))
  }
  fn separator_spacing(&mut self,spacing: f32) -> egui::Response {
    self.add(egui::Separator::default().spacing(spacing))
  }
}

#[derive(Default,Copy,Clone)]
struct RepeatPeriod{
  //These 2 "add up" to make a full repaet period, i.e. 1 years, 3 months, 4 weeks. Can be expresed as months + seconds
  pub regular: RegularRepeatPeriod,
  pub irregular: IrregularRepeatPeriod,
}
#[derive(Default,Copy,Clone)]
struct RegularRepeatPeriod{
  seconds: i64,
}
#[derive(Default,Copy,Clone)]
struct IrregularRepeatPeriod{
  months: i64,
}
impl ToString for RegularRepeatPeriod {
  fn to_string(&self) -> String {
    let mut m = self.seconds/60;
    let mut h = m/60;
    let mut d = h/24;
    let w = d/7;
    let s = self.seconds % 60;
    d %= 7;
    h %= 24;
    m %= 60;
    return format!("{} weeks {} days {} hours {} minutes {} seconds",w,d,h,m,s);
  }
}
impl ToString for IrregularRepeatPeriod {
  fn to_string(&self) -> String {
    let y = self.months/12;
    let m = self.months % 12;
    return format!("{} years {} months",y,m);
  }
}
impl ToString for RepeatPeriod{
  fn to_string(&self) -> String {
    return format!("{} {}",self.irregular.to_string(),self.regular.to_string());
  }
}
impl RegularRepeatPeriod{
  #[allow(dead_code)]
  pub fn new(w: i64,d: i64,h: i64,m: i64,s: i64) -> Self {
    Self{seconds: (((w*7 + d)*24 + h)*60 + m)*60 + s}
  }
}
impl IrregularRepeatPeriod {
  #[allow(dead_code)]
  pub fn new(y: i64,m: i64) -> Self {
    Self{months: 12*y+m}
  }
}
impl RepeatPeriod {
  #[allow(dead_code)]
  pub fn new(force_regular: bool,y: i64,mo: i64,w: i64,d: i64,h: i64,mi: i64,s: i64) -> Self {
    let d  = if force_regular { d+mo*30+y*364 } else { d  };
    let y  = if force_regular {       0       } else { y  };
    let mo = if force_regular {       0       } else { mo };
    Self{irregular: IrregularRepeatPeriod::new(y,mo),regular: RegularRepeatPeriod::new(w,d,h,mi,s)}
  }
}
