// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use libfprint_rs::{Cancellable, CancellableExt, FpContext, FpDevice, FpPrint};
use serde_json::json;
use sqlx::Row;
use sqlx::{mysql::MySqlRow, types::time, MySqlPool};
use std::sync::Arc;
use std::{env, sync::Mutex};
use tauri::State;
use tokio::sync::RwLock;

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command

// #[derive(Serialize, Deserialize)]
// struct Employee {
//     emp_id: u64,
//     fname: String,
//     lname: String,
// }

#[tauri::command]
async fn enumerate_unenrolled_employees() -> String {
    let database_url = match db_url() {
        Ok(url) => url,
        Err(e) => {
            return json!({
              "error": format!("DATABASE_URL not set: {}", e)
            })
            .to_string()
        }
    };

    println!("database_url: {}", database_url);

    let pool = match MySqlPool::connect(&database_url).await {
        Ok(pool) => pool,
        Err(e) => {
            return json!({
              "error": format!("Could not connect to database: {}",e)
            })
            .to_string()
        }
    };

    let result = match sqlx::query!("CALL enumerate_unenrolled_employees_json")
        .fetch_all(&pool)
        .await
    {
        Ok(result) => result,
        Err(_) => {
            return json!({
              "error" : "Failed to execute query"
            })
            .to_string()
        }
    };

    pool.close().await;

    //println!("Found {} unenrolled employees", result.len());

    if result.is_empty() {
        println!("No unenrolled employees found");
        return json!({
          "error" : "No unenrolled employees found"
        })
        .to_string();
    }

    //println!("{:?}", result.get(0).unwrap());

    let mut unenrolled: String = String::from("");

    for row in result.iter() {
        let json = row.get::<serde_json::Value, usize>(0);
        unenrolled = json.to_string();
    }

    unenrolled
}

#[tauri::command]
fn enroll_proc(emp: String, device: State<FpDeviceManager>) -> String {
    //function that is called when scanning a fingerprint for enrollment
    // let emp_num = match emp.trim().parse::<u64>() {
    //     Ok(num) => num,
    //     Err(_) => {
    //         return json!({
    //             "responsecode" : "failure",
    //             "body" : "Invalid employee ID",
    //         })
    //         .to_string()
    //     }
    // };

    /*
     * Get emp_id and check if it already is enrolled.
     */

    // let result = match futures::executor::block_on(async {
    //   query_count(emp_num).await
    // }) {
    //   Ok(result) => result,
    //   Err(e) => return json!({
    //     "responsecode" : "failure",
    //     "body" : format!("Failed to execute query: {}",e),
    //   }).to_string()
    // };

    let fp_scanner = match device.0.as_ref().unwrap().lock() {
        Ok(fp_scanner) => fp_scanner,
        Err(_) => {
            return json!({
              "responsecode" : "failure",
              "body" : "Could not get device",
            })
            .to_string()
        }
    };

    //open the fingerprint scanner
    match fp_scanner.open_sync(None) {
        Ok(_) => (),
        Err(_) => {
            return json!({
              "responsecode" : "failure",
              "body" : "Could not open device",
            })
            .to_string()
        }
    }

    //create a template for the user
    let template = FpPrint::new(&fp_scanner);

    //generates a random uuid
    //let uuid = Uuid::new_v4();

    //OUTDATED: set the username of the template to the uuid generated
    //NEW: set the username of the template to the employee ID to which the fingerprint belongs to
    template.set_username(&emp.to_string());

    println!(
        "Username of the fingerprint: {}",
        template
            .username()
            .expect("Username should be included here")
    );

    let counter = Arc::new(Mutex::new(0)); //a counter for the current scanning phase of the enrollment process

    let new_fprint = match fp_scanner.enroll_sync(template, None, Some(enroll_cb), None) {
        Ok(new_fprint) => new_fprint,
        Err(_) => {
            fp_scanner
                .close_sync(None)
                .expect("Could not close fingerprint scanner");
            return json!({
              "responsecode" : "failure",
              "body" : "Could not enroll fingerprint",
            })
            .to_string();
        }
    };

    println!("Fingerprint has been scanned");

    //close the fingerprint scanner
    match fp_scanner.close_sync(None) {
        Ok(_) => (),
        Err(_) => {
            return json!({
              "responsecode" : "failure",
              "body" : "Could not close fingerprint scanner",
            })
            .to_string();
        }
    } //.expect("Device could not be closed");

    println!("Total enroll stages: {}", counter.lock().unwrap());

    //serialize the fingerprint
    let new_fprint = match new_fprint.serialize() {
        Ok(new_fprint) => new_fprint.to_owned(),
        Err(_) => {
            return json!({
              "responsecode" : "failure",
              "body" : "Could not serialize fingerprint",
            })
            .to_string();
        }
    };

    futures::executor::block_on(async {
        match save_fprint_to_db(&emp, new_fprint).await {
            Ok(_insert) => {
                println!("Fingerprint has been saved in the database");
                json!({
                  "responsecode" : "success",
                  "body" : "Successfully enrolled fingerprint",
                })
                .to_string()
            }
            Err(result) => json!({
              "responsecode" : "failure",
              "body" : result.to_string(),
            })
            .to_string(),
        }
    })
}

// async fn query_count(emp_id: u64) -> Result<(), String> {

//   let database_url = match db_url() {
//     Ok(url) => url,
//     Err(e) => return Err(format!("DATABASE_URL not set: {}", e)),
//   };

//   let pool = match MySqlPool::connect(&database_url).await {
//     Ok(pool) => pool,
//     Err(e) => return Err(e.to_string()),
//   };

//   let record = match sqlx::query!("SELECT COUNT(*) AS count_result FROM enrolled_fingerprints WHERE EMP_ID = ?", emp_id)
//     .fetch_one(&pool)
//     .await {
//       Ok(result) => {
//         if result.count_result == 1 {
//           pool.close().await;
//           return Err(json!({
//             "responsecode" : "failure",
//             "body" : "Employee already enrolled",
//           }).to_string());
//         }
//       },
//       Err(e) => return Err(e.to_string()),
//     };

//   pool.close().await; //close connection to database
//   Ok(())
// }

async fn save_fprint_to_db(emp_id: &String, fprint: Vec<u8>) -> Result<(), String> {
    //save a fingerprint in the database to be associated with an employee id
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
    match sqlx::query!("CALL save_fprint(?,?)", emp_id, fprint)
        .execute(&pool)
        .await
    {
        Ok(row) => {
            pool.close().await; //close connection to database
            match row.rows_affected() {
                //check how many rows were affected by the stored procedure that was previously queried
                0 => println!("No rows affected"),
                _ => println!("Rows affected: {}", row.rows_affected()),
            }
        }
        Err(e) => {
            pool.close().await; //close connection to database before returning error
            return Err(e.to_string());
        }
    };
    //.expect("Could not retrieve latest attendance record");

    //pool.close().await;
    Ok(()) //return from the function with no errors
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

pub fn enroll_cb(
    _device: &FpDevice,
    enroll_stage: i32,
    _print: Option<FpPrint>,
    _error: Option<libfprint_rs::GError>,
    _data: &Option<i32>,
) {
    //print enroll stage of the enroll function
    println!("Enroll_cb Enroll stage: {}", enroll_stage);
}

#[tauri::command]
fn get_device_enroll_stages(device: State<FpDeviceManager>) -> i32 {
    return device.0.as_ref().unwrap().lock().unwrap().nr_enroll_stage();
}

struct FpDeviceManager(Option<Mutex<FpDevice>>, Option<RwLock<Cancellable>>);
//struct ManagedCancellable(Option<RwLock<Cancellable>>);
struct ManagedFprintList(Option<Mutex<Vec<FpPrint>>>);

impl Default for FpDeviceManager {
    fn default() -> Self {
        let context = FpContext::new();
        match context.devices().len() {
            0 => Self(None, None),
            _ => Self(
                Some(Mutex::new(context.devices().remove(0))),
                Some(RwLock::new(Cancellable::new())),
            ),
        }
    }
}

impl Default for ManagedFprintList {
    fn default() -> Self {
        Self(Some(Mutex::new(Vec::new())))
    }
}

// impl Default for ManagedCancellable {
//     fn default() -> Self {
//         Self(Some(RwLock::new(Cancellable::new())))
//     }
// }

impl FpDeviceManager {
    fn cancel_managed(&self) {
        if self.1.is_some() {
            {
                let cancellable =
                    futures::executor::block_on(async { self.1.as_ref().unwrap().read().await });
                cancellable.cancel();
                assert!(cancellable.is_cancelled(), "we did not cancel!");
            }
        }
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
            println!(
                "size of fprint list before de-allocation: {}",
                managed_fprint_list.len()
            );
            managed_fprint_list.clear();
            println!(
                "size of fprint list after de-allocation: {}",
                managed_fprint_list.len()
            );
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
        .manage(FpDeviceManager::default())
        .manage(ManagedFprintList::default())
        //.manage(ManagedCancellable::default())
        .invoke_handler(tauri::generate_handler![
            enumerate_unenrolled_employees,
            enroll_proc,
            get_device_enroll_stages,
            start_identify,
            manual_attendance,
            load_fingerprints,
            cancel_identify
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
//attendance related functions
#[tauri::command]
fn load_fingerprints(fingerprints: State<ManagedFprintList>) -> String {
    let result = match futures::executor::block_on(async {
        fingerprints.obtain_fingerprints_from_db().await
    }) {
        Ok(o) => json!({
            "responsecode" : "success",
            "body" : o,
        })
        .to_string(),
        Err(e) => json!({
            "responsecode" : "failure",
            "body" : e,
        })
        .to_string(),
    };
    return result;
}

#[tauri::command]
fn manual_attendance(emp: String) -> String {
    //manual attendance where an employee puts their employee ID and takes manual attendance with it
    println!("Entering manual attendance");
    println!("Emp: {}", emp);
    if emp.len() > 9 {
        return json!({
            "responsecode" : "failure",
            "body" : "Employee ID should be 9 characters or less. e.g. 12-345-67.",
        }).to_string();
    }
    let row = futures::executor::block_on(async { record_attendance(&emp,true).await }); //query_record_attendance(&emp_num).await

    let output = if row.is_ok() {
        let row = row.unwrap();
        let row_emp_id = row.get::<String, usize>(0);
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
#[tauri::command]
fn cancel_identify(managed: State<FpDeviceManager>) {
    managed.cancel_managed();
}

#[tauri::command]
fn start_identify(
    device: State<FpDeviceManager>,
    fingerprints: State<ManagedFprintList>,
    managed: State<FpDeviceManager>,
) -> String {
    println!("entering verify mode!");

    if device.0.is_none() {
        return json!({
            "responsecode": "failure",
            "body": "Device could not be opened. Please try plugging in your fingerprint scanner and restarting the app.",
        }).to_string();
    }

    //print that the fingerprints are retrieved (for debugging purposes, commented out on production)
    //println!("Fingerprints retrieved");

    {
        let mut cancellable =
            futures::executor::block_on(async { managed.1.as_ref().unwrap().write().await });
        if cancellable.is_cancelled() {
            *cancellable = Cancellable::new();
        }
    }
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
    let mut print_identified: Option<FpPrint> = None;
    {
        let cancellable =
            futures::executor::block_on(async { managed.1.as_ref().unwrap().read().await });
        //identify the scanned fingerprint with identify_sync, it returns nothing if the fingerprint is not in the database, and returns a fingerprint when matched
        print_identified = match fp_scanner.identify_sync(
            &fprint_list,
            Some(&cancellable),
            Some(match_cb),
            None,
            Some(&mut new_print),
        ) {
            Ok(print) => print,
            Err(e) => {
                fp_scanner
                    .close_sync(None)
                    .expect("Could not close the fingerprint scanner");
                if cancellable.is_cancelled() {
                    return json!({
                        "responsecode": "failure",
                        "body": format!("Fingerprint Scan cancelled"),
                    })
                    .to_string();
                } else {
                    return json!({
                    "responsecode": "failure",
                    "body": format!("Could not identify fingerprint due to an error: {}", e.to_string()),
                }).to_string();
                }
            }
        };
    }

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
                    let result = record_attendance(&emp_id, false).await;
                    if result.is_ok() {
                        let row = result.expect("MySqlRow should be able to be unwrapped here");
                        let row_emp_id = row.get::<String, usize>(0);
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

async fn record_attendance(emp_id: &str, manual_attendance: bool) -> Result<MySqlRow, String> {
    //record attendance by emp_id (String type, fingerprint attendance)
    println!("recording attendance manually");
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
    
    //record the attendance
    let row: Option<MySqlRow>;
    match manual_attendance {
        true => {
            row = match sqlx::query!("CALL record_attendance_by_empid(?)", emp_id)
                .fetch_one(&pool)
                .await
            {
                Ok(row) => Some(row),
                Err(e) => {
                    return Err(e.to_string());
                }
            };
        }
        false => {
            row = match sqlx::query!("CALL check_fprint_and_record_attendance(?)", emp_id)
                .fetch_one(&pool)
                .await
            {
                Ok(row) => Some(row),
                Err(e) => {
                    return Err(e.to_string());
                }
            };
        }
    };
    pool.close().await; //close connection to database

    // if row.is_err() {
    //     return Err(row.err().unwrap().to_string());
    // }

    Ok(row.unwrap())
}
