// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
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
fn start_identify(window: Window){    
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
        
        thread::spawn( || {
            let _ = verify(window);
        });
} 
fn main() {
    tauri::Builder::default().setup(|app| {
    	 Ok(())
    })
        .invoke_handler(tauri::generate_handler![greet,start_identify])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
        
}
