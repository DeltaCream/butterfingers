// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde_json::json;
use tauri::State;

use std::{
    env,
    fs::{self, OpenOptions},
    io::{BufReader, Read},
    sync::Mutex,
};

use libfprint_rs::{FpContext, FpDevice, FpPrint};

use sqlx::Row;

use sqlx::{mysql::MySqlRow, types::time, MySqlPool};

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
// #[tauri::command]
// fn greet(name: &str) -> String {
//     format!("Hello, {}! You've been greeted from Rust!", name)
// }

#[derive(Clone, serde::Serialize)]
struct Payload {
    message: String,
}

// #[tauri::command]
// fn test_function(emp: u64) -> String {
//     println!("Entering test function");
//     json!({
//         "responsecode" : "success",
//         "body" : emp,
//     })
//     .to_string()
// }

#[tauri::command]
fn manual_attendance(emp: String) -> String {
    println!("Entering manual attendance");
    println!("Emp: {}", emp);

    let emp_num = match emp.trim().parse::<u64>() {
        Ok(num) => num,
        Err(_) => {
            return json!({
                "responsecode" : "failure",
                "body" : "Invalid employee ID",
            })
            .to_string()
        }
    };

    let row = futures::executor::block_on(async { query_record_attendance(&emp_num).await });

    let output = if row.is_ok() {
        let row = row.unwrap();
        let row_emp_id = row.get::<u64, usize>(0);
        println!("Emp ID: {}", row_emp_id);
        let row_fname = row.get::<String, usize>(1);
        println!("Fname: {}", row_fname);
        let row_lname = row.get::<String, usize>(2);
        println!("Lname: {}", row_lname);
        //let row_date = row.get::<time::Date, usize>(3).to_string();
        let row_date = match row.try_get::<time::Date, usize>(3) {
            Ok(date) => date.to_string(),
            Err(e) => match e {
                sqlx::Error::ColumnNotFound(_) => {
                    println!("Column not found");
                    "error".to_string()
                }
                sqlx::Error::ColumnDecode { index, source } => {
                    println!("Column decode error: {} at index {}", source, index);
                    "error".to_string()
                }
                _ => {
                    println!("Unknown error: {}", e);
                    "error".to_string()
                }
            },
        };
        println!("Date: {}", row_date);

        //let row_time = row.get::<time::Time, usize>(4).to_string();
        let row_time = match row.try_get::<time::Time, usize>(4) {
            Ok(date) => date.to_string(),
            Err(e) => match e {
                sqlx::Error::ColumnNotFound(_) => {
                    println!("Column not found");
                    "error".to_string()
                }
                sqlx::Error::ColumnDecode { index, source } => {
                    println!("Column decode error: {} at index {}", source, index);
                    "error".to_string()
                }
                _ => {
                    println!("Unknown error: {}", e);
                    "error".to_string()
                }
            },
        };
        println!("Time: {}", row_time);

        //let row_attendance_status = row.get::<u16, usize>(5);
        let row_attendance_status = match row.try_get::<u64, usize>(5) {
            Ok(status) => status.to_string(),
            Err(e) => match e {
                sqlx::Error::ColumnNotFound(_) => {
                    println!("Column not found");
                    "error".to_string()
                }
                sqlx::Error::ColumnDecode { index, source } => {
                    println!("Column decode error: {} at index {}", source, index);
                    "error".to_string()
                }
                _ => {
                    println!("Unknown error: {}", e);
                    "error".to_string()
                }
            },
        };
        println!("Attendance Status: {}", row_attendance_status);

        json!({
           "responsecode" : "success",
           "body" : [
                row_emp_id,
                row_fname,
                row_lname,
                row_time,
                row_date,
                row_attendance_status,
           ]
        })
    } else {
        json!({
            "responsecode" : "failure",
            "body" : row.err().unwrap().to_string(),
        })
    };

    output.to_string()
}

/*
{
    "responsecode": "success",
    "body": [
        row_emp_id,
        row_fname,
        row_lname,
        row_time,
        row_date,
        row_attendance_status,
    ]
}
*/

/*
{
    "responsecode": "failure",
    "body": "thingy",
}
*/

#[tauri::command]
fn start_identify(device: State<ManagedFpDevice>, fingerprints: State<ManagedFprintList>) -> String {
    println!("entering verify mode!");

    if device.0.is_none() {
        return json!({
            "responsecode": "failure",
            "body": "Device could not be opened. Please try plugging in your fingerprint scanner and restarting the app.",
        }).to_string();
    }

    //let mut fun_result: Option<String> = Some(String::from(""));
    //let mut fun_result: Value = Default::default();

    // Get a list of all entries in the folder
    // let entries = match fs::read_dir(
    //     dirs::home_dir()
    //         .expect("Home directory could not be found")
    //         .join("print/"),
    // ) {
    //     Ok(entries) => entries,
    //     Err(e) => {
    //         return json!({
    //             "responsecode": "failure",
    //             "body": format!("Could not read the directory to retrieve files pertaining to the stored fingerprints. Error: {}", e.to_string())
    //         }).to_string();
    //     }
    // };

    // Extract the filenames from the directory entries and store them in a vector
    // let file_names: Vec<String> = entries
    //     .filter_map(|entry| {
    //         let path = entry.ok()?.path();
    //         if path.is_file() {
    //             path.file_name()?.to_str().map(|s| s.to_owned())
    //         } else {
    //             None
    //         }
    //     })
    //     .collect();

    // Iterate over the file names
    // let fingerprints = file_names
    //     .iter()
    //     .map(|filename| {
    //         //for every file name
    //         //go to the directory where the file will be placed
    //         let fpprint_file = OpenOptions::new()
    //             .read(true)
    //             .open(
    //                 dirs::home_dir()
    //                     .expect("Home directory could not be found")
    //                     .join(format!("print/{}", filename)),
    //             )
    //             .expect("Could not read the fingerprint file");

    //         //create a buffer for the files
    //         let mut reader = BufReader::new(fpprint_file);
    //         let mut buffer = Vec::new();

    //         //read file into buffer vector
    //         reader
    //             .read_to_end(&mut buffer)
    //             .expect("Could not retrieve contents of file");

    //         //deserialize the fingerprint stored in the file
    //         let deserialized_print = FpPrint::deserialize(&buffer);

    //         //retrieve the enrolled print from deserialized_print
    //         deserialized_print.expect("Could not unwrap the deserialized print")
    //         //let enrolled_print = deserialized_print.expect("Could not unwrap the deserialized print");
    //     })
    //     .collect::<Vec<FpPrint>>();

    // for (i, fingerprint) in fingerprints.iter().enumerate() {
    //     //print the fingeprint number (on the array of fingerprints returned) and its corresponding username
    //     println!(
    //         "Username for Fingerprint #{}: {:?}",
    //         i,
    //         fingerprint
    //             .username()
    //             .expect("Fingerprint username could not be retrieved")
    //     );
    // }

    //print that the fingerprints are retrieved (for debugging purposes, commented out on production)
    println!("Fingerprints retrieved");

    //
    let fp_scanner = match device.0.as_ref().unwrap().lock() {
        Ok(fp_scanner) => fp_scanner,
        Err(e) => {
            return json!({
                "responsecode": "failure",
                "body": format!("Could not open fingerprint scanner. Error: {}", e.to_string()),
            })
            .to_string();
        }
    };

    match fp_scanner.open_sync(None) {
        Ok(()) => {
            println!("Fingerprint scanner opened!");
        }
        Err(e) => {
            return json!({
                "responsecode": "failure",
                "body": format!("Could not open fingerprint scanner. Error: {}", e.to_string()),
            })
            .to_string();
        }
    };

    let mut new_print = FpPrint::new(&fp_scanner);
    println!("Please scan your fingerprint");
    let fprint_list = match fingerprints.0.as_ref().unwrap().lock() {
        Ok(fprint_list) => fprint_list,
        Err(e) => {
            return json!({
                "responsecode": "failure",
                "body": format!("Could not parse list of fingerprints. Error: {}", e.to_string()),
            })
            .to_string();
        }
    };
    let print_identified = match fp_scanner.identify_sync(
        &fprint_list,
        None,
        Some(match_cb),
        None,
        Some(&mut new_print),
    ) {
        Ok(print) => print,
        Err(e) => {
            fp_scanner
                .close_sync(None)
                .expect("Could not close the fingerprint scanner");
            return json!({
                    "responsecode": "failure",
                    "body": format!("Could not identify fingerprint due to an error: {}", e.to_string()),
                }).to_string();
        }
    };
    //.expect("Fingerprint could not be identified due to an error");

    match fp_scanner.close_sync(None) {
        Ok(()) => (),
        Err(e) => {
            return json!({
                "responsecode": "failure",
                "body": format!("Could not close the fingerprint scanner. Error: {}",&e.to_string()),
            }).to_string();
        }
    }

    if print_identified.is_some() { //put another check sa db side if the preloaded fprint is in the db
        let fprint = print_identified.expect("Print should be able to be unwrapped here");
        let emp_id = fprint.username();
        match emp_id {
            Some(emp_id) => {
                futures::executor::block_on(async {
                    println!("emp_id of the fingerprint: {}", emp_id);
                    println!("Before recording attendance");
                    let result = record_attendance(&emp_id).await;
                    if result.is_ok() {
                        let row = result.unwrap();
                        let row_emp_id = row.get::<u64, usize>(0);
                        let row_fname = row.get::<String, usize>(1);
                        let row_lname = row.get::<String, usize>(2);
                        let row_date = row.get::<time::Date, usize>(3).to_string();
                        let row_time = row.get::<time::Time, usize>(4).to_string();
                        let row_attendance_status = row.get::<u16, usize>(5);

                        let msg =
                            format!("\nAttendance recorded for {} {}\n", row_fname, row_lname);

                        println!("{}", msg);

                        json!({
                            "responsecode": "success",
                            "body": [
                                row_emp_id,
                                row_fname,
                                row_lname,
                                row_time,
                                row_date,
                                row_attendance_status,
                            ]
                        })
                        .to_string()
                    } else {
                        //show that attendance could not be recorded
                        println!("Attendance could not be recorded\n");
                        json!({
                            "responsecode": "failure",
                            "body": result.err().unwrap().to_string(),
                        })
                        .to_string()
                    }
                })
            }
            None => {
                println!("No user associated with fingerprint"); //uuid did not contain a string (essentially None acts as a null value)
                json!({
                    "responsecode": "failure",
                    "body": "No user associated with fingerprint. Please try scanning again, or enroll first.",
                }).to_string()
            }
        }
    } else {
        println!("No matching fingerprint could be found.");
        json!({
            "responsecode": "failure",
            "body": "No matching fingerprint could be found.",
        })
        .to_string()
    }
}
/*
{
    "responsecode": "success",
    "body": [
        "emp_id",
        "fname",
        "lname",
        "time"
    ]
}
*/

/*
{
    "responsecode": "failure",
    "body": "thingy",
}
*/

async fn query_record_attendance(emp_id: &u64) -> Result<MySqlRow, String> {
    let database_url = match db_url() {
        Ok(url) => url,
        Err(e) => return Err(format!("DATABASE_URL not set: {}", e)),
    };

    //connect to the database
    let pool = match MySqlPool::connect(&database_url).await {
        Ok(pool) => pool,
        Err(e) => return Err(e.to_string()),
    };

    //query the record_attendance_by_empid stored procedure (manual attendance)
    let result = match sqlx::query!("CALL record_attendance_by_empid(?)", emp_id)
        //.execute(&pool)
        .fetch_one(&pool)
        .await
    {
        Ok(result) => {
            println!("Attendance recorded successfully");
            result
        }
        Err(e) => match e {
            sqlx::Error::Database(e) => {
                return Err(e.message().to_string());
            }
            _ => {
                return Err(e.to_string());
            }
        },
    };

    // let uuid_query = match sqlx::query!("SELECT fprint_uuid from enrolled_fingerprints where emp_id = ?", emp_id)
    //     .fetch_one(&pool)
    //     .await {
    //         Ok(uuid) => uuid,
    //         Err(e) => return Err(e.to_string()),
    //     };
    //     //.expect("Could not retrieve uuid");

    // let uuid = uuid_query.fprint_uuid;   //.get::<String, usize>(0);

    // println!("UUID: {}", uuid);

    // let row = match sqlx::query!("CALL get_latest_attendance_record(?)", uuid)
    //     .fetch_one(&pool)
    //     .await {
    //         Ok(row) => row,
    //         Err(e) => return Err(e.to_string()),
    //     };
    //.expect("Could not retrieve latest attendance record");

    pool.close().await; //close connection to database
    Ok(result) //return from the function with no errors
}

// async fn employee_name_from_uuid(uuid: &str) -> String {
//     dotenvy::dotenv().unwrap();
//     let pool = MySqlPool::connect(&env::var("DATABASE_URL").unwrap())
//         .await
//         .unwrap();
//     let result = sqlx::query!(r#"SELECT enrolled_fingerprints.fprint_uuid AS "uuid", employee.fname AS "fname", employee.mname AS "mname", employee.lname AS "lname"
//     FROM enrolled_fingerprints JOIN employee USING(emp_id) WHERE fprint_uuid = ?"#, uuid)
//         .fetch_one(&pool)
//         .await
//         .expect("Could not retrieve employee name from uuid");
//     pool.close().await;
//     match (result.fname, result.mname, result.lname) {
//         (fname, Some(mname), lname) => format!("{} {} {}", fname, mname, lname),
//         (fname, None, lname) => format!("{} {}", fname, lname),
//     }
// }

// async fn employee_name_from_empid(emp_id: &u64) -> String {
//     dotenvy::dotenv().unwrap();
//     let pool = MySqlPool::connect(&env::var("DATABASE_URL").unwrap())
//         .await
//         .unwrap();
//     let result = sqlx::query!(//r#"SELECT production_staff.emp_id AS "emp_id",
//     //employee.fname AS "fname", employee.mname AS "mname", employee.lname AS "lname" FROM production_staff JOIN employee USING(emp_id) WHERE emp_id = ?"#, emp_id)
//         r#"SELECT employee.emp_id AS "emp_id", employee.fname AS "fname", employee.mname AS "mname", employee.lname AS "lname" FROM employee WHERE role_code = 2 AND emp_id = ?"#, emp_id)
//         .fetch_one(&pool)
//         .await
//         .expect("Could not retrieve employee name from employee id");
//     pool.close().await;
//     match (result.fname, result.mname, result.lname) {
//         (fname, Some(mname), lname) => format!("{} {} {}", fname, mname, lname),
//         (fname, None, lname) => format!("{} {}", fname, lname),
//     }
// }

//function below is a callback function that is called when a scanned fingerprint is matched with previously enrolled fingerprints
pub fn match_cb(
    _device: &FpDevice,
    matched_print: Option<FpPrint>,
    print: FpPrint,
    _error: Option<libfprint_rs::GError>,
    _data: &Option<()>,
) {
    if let Some(matched_print) = &matched_print {
        //set the scanned fingerprint's username to the matched print's username
        //(because the scanned fingerprint was matched with the previously enrolled fingerprint,
        //and currently, the scanned fingerprint has no username)
        print.set_username(
            &matched_print
                .username()
                .expect("Username could not be retrieved"),
        );
        println!("Matched");
    } else {
        //if matched_print is None (null value)
        //print that no fingerprint was matched with the scanned fingerprint
        println!("Not matched");
    }
}

async fn record_attendance(emp_id: &str) -> Result<MySqlRow, String> {
    println!("recording attendance");
    //setup involving the .env file

    let database_url = match db_url() {
        Ok(url) => url,
        Err(e) => return Err(format!("DATABASE_URL not set: {}", e)),
    };

    //connect to the database
    let pool = match MySqlPool::connect(&database_url).await {
        Ok(pool) => pool,
        Err(e) => return Err(e.to_string()),
    };

    //query the record_attendance stored procedure (non-manual attendance)
    match sqlx::query!("CALL record_attendance_by_empid(?)", emp_id)
        .execute(&pool)
        .await
    {
        Ok(_) => (),
        Err(e) => {
            return Err(e.to_string());
        }
    };

    let row = sqlx::query!("CALL get_latest_attendance_record(?)", emp_id)
        .fetch_one(&pool)
        .await;

    pool.close().await; //close connection to database

    if row.is_err() {
        return Err(row.err().unwrap().to_string());
    }

    Ok(row.ok().unwrap())
}

async fn obtain_fingerprints_from_db() -> Result<Vec<FpPrint>, String> {
    println!("recording attendance");
    //setup involving the .env file

    let database_url = match db_url() {
        Ok(url) => url,
        Err(e) => return Err(format!("DATABASE_URL not set: {}", e)),
    };

    //connect to the database
    let pool = match MySqlPool::connect(&database_url).await {
        Ok(pool) => pool,
        Err(e) => return Err(e.to_string()),
    };

    let row = sqlx::query!("SELECT fprint FROM enrolled_fingerprints")
        .fetch_all(&pool)
        .await;

    pool.close().await; //close connection to database
    if row.is_err() {
        return Err(row.err().unwrap().to_string());
    }
    
    let raw_fprints = row.ok().unwrap();

    let fprint_list = raw_fprints.iter().map(|fprint_file|{
        let deserialized_print = FpPrint::deserialize(&fprint_file.fprint);
        deserialized_print.unwrap()
    }).collect::<Vec<FpPrint>>();
    Ok(fprint_list)
}

fn db_url() -> Result<String, String> {
    match dotenvy::dotenv() {
        Ok(_) => (),
        Err(e) => return Err(format!("Failed to load .env file: {}", e)),
    }

    let db_type = match env::var("DB_TYPE") {
        Ok(db_type) => db_type,
        Err(_) => return Err("DB_TYPE not set".to_string()),
    };

    let db_username = match env::var("DB_USERNAME") {
        Ok(username) => username,
        Err(_) => return Err("DB_USERNAME not set".to_string()),
    };

    let db_password = match env::var("DB_PASSWORD") {
        Ok(password) => password,
        Err(_) => return Err("DB_PASSWORD not set".to_string()),
    };

    let hostname = match env::var("HOSTNAME") {
        Ok(name) => name,
        Err(_) => return Err("HOSTNAME not set".to_string()),
    };

    let db_port = match env::var("DB_PORT") {
        Ok(port) => port,
        Err(_) => return Err("DB_PORT not set".to_string()),
    };

    let db_name = match env::var("DB_NAME") {
        Ok(name) => name,
        Err(_) => return Err("DB_NAME not set".to_string()),
    };

    let db_params = match env::var("DB_PARAMS") {
        Ok(params) => params,
        Err(_) => return Err("DB_PARAMS not set".to_string()),
    };

    let database_url = format!(
        "{}://{}:{}@{}:{}/{}?{}",
        db_type, db_username, db_password, hostname, db_port, db_name, db_params
    );
    Ok(database_url)
}

// struct AttRecord {
//     @emp_id: u64, //bigint
//     fname: String, //varchar
//     lname: String, //varchar
//     curr_date: time::Date, //date
//     curr_time: time::Time, //time
//     incoming_status_code: u16, //smallint unsigned
// }

// struct Wrapper {
//     context: Arc<Mutex<FpContext>>,
// }

// unsafe impl Send for Wrapper {}
// unsafe impl Sync for Wrapper {}

struct ManagedFpDevice(Option<Mutex<FpDevice>>);
struct ManagedFprintList(Option<Mutex<Vec<FpPrint>>>);

impl Default for ManagedFpDevice {
    fn default() -> Self {
        let context = FpContext::new();
        match context.devices().len() {
            0 => Self(None),
            _ => Self(Some(Mutex::new(context.devices().remove(0)))),
        }
    }
}

impl Default for ManagedFprintList {
    fn default() -> Self {
        Self(Some(Mutex::new(
            futures::executor::block_on(async {
                obtain_fingerprints_from_db().await.unwrap()
             })
            
        )))
    }
}

// impl Default for Note {
//     fn default() -> Self {
//         Self(Mutex::new(FpContext::new().devices().remove(0)))
//     }
// }

#[tokio::main]
async fn main() {
    tauri::Builder::default()
        .setup(|_app| Ok(()))
        .manage(ManagedFpDevice::default())
        .manage(ManagedFprintList::default())
        .invoke_handler(tauri::generate_handler![
            start_identify,
            manual_attendance
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
