// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use libfprint_rs::FpContext;
use tauri::{App, AppHandle, Manager, Window};
use std::thread;
use butterfingers_tauri::butterfingersd_enroll::enroll as enroll;
use butterfingers_tauri::butterfingersd_verify::verify as verify;
// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}
#[derive(Clone, serde::Serialize)]
struct Payload {
    message: String,
}
#[tauri::command]
fn start_identify(window: Window) {    
        // thread code
        //;
        println!("starting verify!");
        let _ = window.emit("identify-messages", "starting verify!");
        //println!("{}", result1.is_ok());
        // tauri::async_runtime::spawn(async move {
            
        // });
        // tokio::spawn(async {
        //     let _ = verify();
        // });
        // let id = window.listen("manual-attendance", |event| {
        //     //println!("got event-name with payload {:?}", event.payload());
        //     println!("value: {}", event.payload().unwrap());
        //     if event.payload().eq(&Some("\"1234\"")){
        //         println!("right value!"); 
        //         window.unlisten(handler_id);
        //     } else {
        //         println!("wrong value!");
        //     }
        // });
        
        // println!("entering verify mode!");
        // window.emit("identify-messages","entering verify mode!");
        // //Get FpContext to get devices
        // println!("Before context");
        // window.emit("identify-messages","before context");
        // let context = FpContext::new();
        // //Use FpContext to get devices (returns a vector/array of devices)
        // println!("before devices");
        // window.emit("identify-messages","before devices");
        // let mut devices = context.devices();
        // //Get the first device (which, in this case, is the only device, and it is the fingerprint scanner)
        // println!("before scanner");
        // window.emit("identify-messages","before scanner");
        // //let fp_scanner = devices.first().expect("Devices could not be retrieved");
        // // let fp_scanner = devices.get(0).expect("Devices could not be retrieved");
        // let fp_scanner = devices.remove(0);
        // fp_scanner.open_sync(None).expect("Device could not be opened");

        // tauri::SyncTask::current().spawn( || {
        //     verify(window, fp_scanner);
        // });

        // thread::spawn( || {
        //     verify(window, fp_scanner);
        // });
        // let rt = Runtime::new().expect("Failed to create Tokio runtime");
        // rt.spawn(async {
        //     verify(window, fp_scanner).await;
        // });
        tauri::async_runtime::spawn(async {
            verify(window).await;
        });
} 

#[tokio::main]
async fn main() {
    tauri::Builder::default().setup(|app| {
        let main_window = app.get_window("main").unwrap();
        // tauri::async_runtime::spawn(async move {
            //verify(main_window.clone()).await;
        // });
    	Ok(())
    })
        .invoke_handler(tauri::generate_handler![greet,start_identify])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
        
}
