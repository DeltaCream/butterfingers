// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde_json::json;
use tauri::State;
use std::{env, sync::Mutex};

use libfprint_rs::{FpContext, FpDevice, FpPrint};

use sqlx::Row;

use sqlx::{mysql::MySqlRow, types::time, MySqlPool};

#[tauri::command]
fn load_fingerprints(fingerprints: State<ManagedFprintList>) -> String {
    let result = match futures::executor::block_on(async { 
        fingerprints.obtain_fingerprints_from_db().await
    }) {
        Ok(o) => json!({
            "responsecode" : "failure",
            "body" : o,
        }).to_string(),
        Err(e) => json!({
            "responsecode" : "failure",
            "body" : e,
        }).to_string(),
    };
    return result;
}

#[tauri::command]
fn manual_attendance(emp: String) -> String {
    //manual attendance where an employee puts their employee ID and takes manual attendance with it
    println!("Entering manual attendance");
    println!("Emp: {}", emp);

    let row = futures::executor::block_on(async { record_attendance(&emp).await }); //query_record_attendance(&emp_num).await

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

        //return the json containing the employee and the attendance details
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
fn start_identify(
    device: State<ManagedFpDevice>,
    fingerprints: State<ManagedFprintList>,
) -> String {
    println!("entering verify mode!");

    if device.0.is_none() {
        return json!({
            "responsecode": "failure",
            "body": "Device could not be opened. Please try plugging in your fingerprint scanner and restarting the app.",
        }).to_string();
    }

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

    let mut new_print = FpPrint::new(&fp_scanner); //create a new fingerprint
    println!("Please scan your fingerprint");

    let fprint_list = match fingerprints.0.as_ref().unwrap().lock() {
        //get the list of fingerprints
        Ok(fprint_list) => fprint_list,
        Err(e) => {
            return json!({
                "responsecode": "failure",
                "body": format!("Could not parse list of fingerprints. Error: {}", e.to_string()),
            })
            .to_string();
        }
    };

    //identify the scanned fingerprint with identify_sync, it returns nothing if the fingerprint is not in the database, and returns a fingerprint when matched
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

    match fp_scanner.close_sync(None) {
        //close fingerprint scanner
        Ok(()) => (),
        Err(e) => {
            return json!({
                "responsecode": "failure",
                "body": format!("Could not close the fingerprint scanner. Error: {}",&e.to_string()),
            }).to_string();
        }
    }

    if print_identified.is_some() {
        //put another check sa db side if the preloaded fprint is in the db
        let fprint = print_identified.expect("Print should be able to be unwrapped here");
        let emp_id = fprint.username();
        match emp_id {
            Some(emp_id) => {
                futures::executor::block_on(async {
                    println!("emp_id of the fingerprint: {}", emp_id);
                    println!("Before recording attendance");
                    let result = record_attendance(&emp_id).await;
                    if result.is_ok() {
                        let row = result.expect("MySqlRow should be able to be unwrapped here");
                        let row_emp_id = row.get::<u64, usize>(0);
                        let row_fname = row.get::<String, usize>(1);
                        let row_lname = row.get::<String, usize>(2);
                        let row_date = row.get::<time::Date, usize>(3).to_string();
                        let row_time = row.get::<time::Time, usize>(4).to_string();
                        let row_attendance_status = row.get::<u16, usize>(5);

                        let msg =
                            format!("\nAttendance recorded for {} {}\n", row_fname, row_lname);

                        println!("{}", msg);

                        json!({ //return the json containing the employee and the attendance details
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
                println!("No employee associated with the scanned fingerprint."); //uuid did not contain a string (essentially None acts as a null value)
                json!({
                    "responsecode": "failure",
                    "body": "No employee associated with the scanned fingerprint. Please try scanning again, or enroll first.",
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

/* Below are the sample json responses that can be returned (success and failure respectively):
{
    "responsecode": "success",
    "body": [
        "emp_id",
        "fname",
        "lname",
        "time",
        "date",
        "attendance_status"
    ]
}
*/

/*
{
    "responsecode": "failure",
    "body": "thingy",
}
*/

//function below is a callback function that is called when a scanned fingerprint is to be matched with previously enrolled fingerprints
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
    //record attendance by emp_id (String type, fingerprint attendance)
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
    let row = match sqlx::query!("CALL check_fprint_and_record_attendance(?)", emp_id).fetch_one(&pool).await{
        Ok(row) => row,
        Err(e) => {
            return Err(e.to_string());
        }
    };

    pool.close().await; //close connection to database

    // if row.is_err() {
    //     return Err(row.err().unwrap().to_string());
    // }

    Ok(row)
}



fn db_url() -> Result<String, String> {
    // match dotenvy::dotenv() {
    //     Ok(_) => (),
    //     Err(e) => return Err(format!("Failed to load .env file: {}", e)),
    // }

    let db_type = dotenvy_macro::dotenv!("DB_TYPE");
    // {
    //     Ok(db_type) => db_type,
    //     Err(_) => return Err("DB_TYPE not set".to_string()),
    // };

    let db_username = dotenvy_macro::dotenv!("DB_USERNAME");
    // {
    //     Ok(username) => username,
    //     Err(_) => return Err("DB_USERNAME not set".to_string()),
    // };

    let db_password = dotenvy_macro::dotenv!("DB_PASSWORD");
    //  {
    //     Ok(password) => password,
    //     Err(_) => return Err("DB_PASSWORD not set".to_string()),
    // };

    let hostname = dotenvy_macro::dotenv!("HOSTNAME");
    // {
    //     Ok(name) => name,
    //     Err(_) => return Err("HOSTNAME not set".to_string()),
    // };

    let db_port = dotenvy_macro::dotenv!("DB_PORT");
    // {
    //     Ok(port) => port,
    //     Err(_) => return Err("DB_PORT not set".to_string()),
    // };

    let db_name = dotenvy_macro::dotenv!("DB_NAME");
    // {
    //     Ok(name) => name,
    //     Err(_) => return Err("DB_NAME not set".to_string()),
    // };

    let db_params = dotenvy_macro::dotenv!("DB_PARAMS"); 
    // {
    //     Ok(params) => params,
    //     Err(_) => return Err("DB_PARAMS not set".to_string()),
    // };

    let database_url = format!(
        "{}://{}:{}@{}:{}/{}?{}",
        db_type, db_username, db_password, hostname, db_port, db_name, db_params
    );
    Ok(database_url)
}

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
        Self(Some(Mutex::new(Vec::new())))
    }
}

impl ManagedFprintList {
    async fn obtain_fingerprints_from_db(&self) -> Result<String, String> {
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
        let mut managed_fprint_list = self.0.as_ref().unwrap().lock().unwrap();
        
        if managed_fprint_list.is_empty() {
            println!("size of fprint list: {}", managed_fprint_list.len());
            for fprint_file in raw_fprints {
                let deserialized_print = match FpPrint::deserialize(&fprint_file.fprint) {
                    Ok(deserialized_print) => deserialized_print,
                    Err(e) => {
                        return Err(format!(
                            "Could not deserialize one of the fingerprints: {}",
                            e
                        ));
                    }
                };
                managed_fprint_list.push(deserialized_print);
            }
        } else {
            println!("size of fprint list before de-allocation: {}", managed_fprint_list.len());
            managed_fprint_list.clear();
            println!("size of fprint list after de-allocation: {}", managed_fprint_list.len());
            assert!(managed_fprint_list.is_empty(), "vector is not empty!");
            //let mut fprint_list = Vec::new();
            for fprint_file in raw_fprints {
                let deserialized_print = match FpPrint::deserialize(&fprint_file.fprint) {
                    Ok(deserialized_print) => deserialized_print,
                    Err(e) => {
                        return Err(format!(
                            "Could not deserialize one of the fingerprints: {}",
                            e
                        ));
                    }
                };
                managed_fprint_list.push(deserialized_print);
            }
            //*managed_fprint_list = fprint_list;
        }

        //let mut fprint_list = Vec::new(); //ideally should work similar to above
        // for fprint_file in raw_fprints {
        //     let deserialized_print = match FpPrint::deserialize(&fprint_file.fprint) {
        //         Ok(deserialized_print) => deserialized_print,
        //         Err(e) => {
        //             return Err(format!(
        //                 "Could not deserialize one of the fingerprints: {}",
        //                 e
        //             )); //modified to early return in case one of the fingerprints cannot be deserialized
        //         }
        //     };
        //     fprint_list.push(deserialized_print);
        // }
    
        // self.0 = Some(Mutex::new(fprint_list));
        Ok(String::from("Fingerprints Successfully loaded!"))
    }
}

#[tokio::main]
async fn main() {
    env::set_var("WEBKIT_DISABLE_COMPOSITING_MODE", "1");
    tauri::Builder::default()
        .setup(|_app| Ok(()))
        .manage(ManagedFpDevice::default())
        .manage(ManagedFprintList::default())
        .invoke_handler(tauri::generate_handler![start_identify, manual_attendance, load_fingerprints])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// async fn query_record_attendance(emp_id: &u64) -> Result<MySqlRow, String> {
//     //record attendance by emp_id (u64 type, manual attendance)
//     let database_url = match db_url() {
//         Ok(url) => url,
//         Err(e) => return Err(format!("DATABASE_URL not set: {}", e)),
//     };

//     //connect to the database
//     let pool = match MySqlPool::connect(&database_url).await {
//         Ok(pool) => pool,
//         Err(e) => return Err(e.to_string()),
//     };

//     //query the record_attendance_by_empid stored procedure (manual attendance)
//     let result = match sqlx::query!("CALL record_attendance_by_empid(?)", emp_id)
//         //.execute(&pool)
//         .fetch_one(&pool)
//         .await
//     {
//         Ok(result) => {
//             println!("Attendance recorded successfully");
//             result
//         }
//         Err(e) => match e {
//             sqlx::Error::Database(e) => {
//                 return Err(e.message().to_string());
//             }
//             _ => {
//                 return Err(e.to_string());
//             }
//         },
//     };

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

//     pool.close().await; //close connection to database
//     Ok(result) //return from the function with no errors
// }

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
