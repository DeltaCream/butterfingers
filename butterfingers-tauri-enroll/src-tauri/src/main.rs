// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
 use std::{
        env, 
        fs::OpenOptions,
        io::{self, Write}, 
        sync::{Arc, Mutex},
    };
   use libfprint_rs::{
        FpPrint, 
        FpDevice,
    };
      use sqlx::{
        MySqlPool,
        Row,
    };
    
    use uuid::Uuid;
// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}
#[tauri::command]
async fn enumerate_unenrolled_employees() -> anyhow::Result<()>{
 	dotenvy::dotenv()?;
        let pool = MySqlPool::connect(&env::var("DATABASE_URL")?).await?; 
        let result = sqlx::query!("CALL enumerate_unenrolled_employees_json")
            .fetch_all(&pool)
            .await?;
       if result.is_empty() {
            println!("No unenrolled employees found");
       }
       for row in result.iter().enumerate() {
       		println!("{}", row.get::<String, usize>(0));
       }
       Ok(())
}
fn main() {
    tauri::Builder::default()
    	.plugin(tauri_plugin_sql::Builder::default().build())
        .invoke_handler(tauri::generate_handler![greet, enumerate_unenrolled_employees])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
