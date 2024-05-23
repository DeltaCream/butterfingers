// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use libfprint_rs::{Cancellable, CancellableExt, FpContext, FpDevice, FpPrint};
use serde_json::json;
use sqlx::mysql::{MySqlPoolOptions, MySqlQueryResult};
use sqlx::Row;
use sqlx::{mysql::MySqlRow, types::time, MySqlPool};
use std::sync::Arc;
use std::{env, sync::Mutex};
use tauri::State;
use tokio::sync::RwLock;

/*
use crate::utils::{
    check_fingerprint_scanner,
    get_device_enroll_stages,
    load_fingerprints,
};

use crate::manage::{
    enumerate_enrolled_employees,
    delete_fingerprint,
    verify_fingerprint,
};

use crate::enroll::{
    enumerate_unenrolled_employees,
    enroll_proc,
};

use crate::attendance::{
    manual_attendance,
    cancel_identify,
    start_identify,
};
*/

//Main Function
#[tokio::main]
async fn main() {
    env::set_var("WEBKIT_DISABLE_COMPOSITING_MODE", "1");
    tauri::Builder::default()
        .setup(|_app| Ok(()))
        .manage(FpDeviceManager::default())
        .manage(ManagedMySqlPool::default())
        .invoke_handler(tauri::generate_handler![
            // check_fingerprint_scanner,
            // get_device_enroll_stages,
            // load_fingerprints,
            // enumerate_enrolled_employees,
            // delete_fingerprint,
            // verify_fingerprint,
            // enumerate_unenrolled_employees,
            // enroll_proc,
            // manual_attendance,
            // cancel_identify,
            // start_identify,
            check_fingerprint_scanner,
            delete_fingerprint,
            verify_fingerprint,
            enumerate_unenrolled_employees,
            enumerate_enrolled_employees,
            enroll_proc,
            get_device_enroll_stages,
            start_identify,
            manual_attendance,
            load_fingerprints,
            cancel_function
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/*
    Utils Section:
    Functions that are used in multiple commands and miscellaneous
    Also includes struct definitions for state variables
*/

/// Check if fingerprint scanner is connected.
/// Returns a json object with a responsecode and body.
/// It's commonly used to check whether or not the fingerprint scanner is plugged in *before* any fingerprint commands are run.
/// Error messages are returned in the body if there is a problem with the fingerprint scanner and should be handled.
#[tauri::command]
fn check_fingerprint_scanner(device: State<FpDeviceManager>) -> String {
    if device.0.is_none() {
        return json!({
            "responsecode": "failure",
            "body": "Device could not be opened. Please try plugging in your fingerprint scanner and restarting the app.",
        })
        .to_string();
    }
    json!({
        "responsecode": "success",
        "body": "Fingerprint scanner is connected",
    })
    .to_string()
}

/// Function to get the number of enroll stages for the fingerprint scanner.
#[tauri::command]
fn get_device_enroll_stages(device: State<FpDeviceManager>) -> i32 {
    //return device.0.as_ref().unwrap().lock().unwrap().nr_enroll_stage();
    return device.0.as_ref().unwrap().blocking_read().nr_enroll_stage();
}

/// Function to get the database URL from the .env file, to be passed to sqlx::mysql::MySqlPoolOptions later when connecting to the database at the start of the program.
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

/// A struct to manage the fingerprint scanner, as well as the cancellation object and the fingerprint list.
struct FpDeviceManager(
    Option<RwLock<FpDevice>>, //Option<Mutex<FpDevice>>,
    Option<RwLock<Cancellable>>,
    Option<Mutex<Vec<FpPrint>>>,
);

//struct ManagedCancellable(Option<RwLock<Cancellable>>);
//struct ManagedFprintList(Option<Mutex<Vec<FpPrint>>>);

/// A struct to manage the database connection. This is created at startup to prevent the overhead of creating a pool everytime a database query is made.
struct ManagedMySqlPool(Option<MySqlPool>);

impl Default for FpDeviceManager {
    fn default() -> Self {
        let context = FpContext::new();
        match context.devices().len() {
            0 => Self(None, None, Some(Mutex::new(Vec::new()))), //there should be a vector for the fingerprint list regardless if the fingerprint scanner is plugged in or not
            _ => Self(
                Some(RwLock::new(context.devices().remove(0))),//Some(Mutex::new(context.devices().remove(0))),
                Some(RwLock::new(Cancellable::new())),
                Some(Mutex::new(Vec::new())),
            ),
        }
    }
}

// impl Default for ManagedFprintList {
//     fn default() -> Self {
//         Self(Some(Mutex::new(Vec::new())))
//     }
// }

// impl Default for ManagedCancellable {
//     fn default() -> Self {
//         Self(Some(RwLock::new(Cancellable::new())))
//     }
// }

impl FpDeviceManager {
    /// Function to cancel the current process involving the fingerprint scanner.
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

    /// Function to obtain fingerprints from the database.
    /// This function is used to dynamically load the list of enrolled fingerprints as it is called with every reload via the load_fingerprints command.
    async fn obtain_fingerprints_from_db(&self, pool: &MySqlPool) -> Result<String, String> {
        let row = sqlx::query!("SELECT fprint FROM enrolled_fingerprints")
            .fetch_all(pool)
            .await;

        if row.is_err() {
            return Err(row.err().unwrap().to_string());
        }

        // match e {
        //     sqlx::Error::Database(e) => {
        //         return Err(e.message().to_string());
        //     }

        //     _ => {
        //         return Err(e.to_string());
        //     }
        // }

        let raw_fprints = row.ok().unwrap();
        let mut managed_fprint_list = self.2.as_ref().unwrap().lock().unwrap();

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
        Ok(String::from("Fingerprints Successfully loaded!"))
    }
}

impl Default for ManagedMySqlPool {
    fn default() -> Self {
        let database_url = match db_url() {
            Ok(url) => url,
            Err(e) => {
                println!("DATABASE_URL not set: {}", e);
                return Self(None);
            }
        };

        let pool = match futures::executor::block_on(async {
            //MySqlPool::connect(&database_url).await //customized version of this line of code below
            MySqlPoolOptions::new()
                .min_connections(1) //below will be the portion where you configure the database pool
                .max_connections(10) //view https://docs.rs/sqlx-core/0.7.3/src/sqlx_core/pool/options.rs.html#136 for more details
                .connect(&database_url)
                .await
        }) {
            Ok(pool) => pool,
            Err(e) => {
                // return Err(json!({
                //     "error": format!("Could not connect to database: {}", e)
                // })
                // .to_string())
                println!("Could not connect to database: {}", e);
                return Self(None);
            }
        };

        Self(Some(pool))
    }
}

/// Load fingerprints from database. This function is called with every reload in JavaScript, and calls the obtain_fingerprints_from_db function which modifies the fingerprint list.
#[tauri::command]
fn load_fingerprints(
    managed: State<FpDeviceManager>,
    managed_pool: State<ManagedMySqlPool>,
) -> String {
    let pool = managed_pool.0.as_ref().unwrap();
    match futures::executor::block_on(async { managed.obtain_fingerprints_from_db(pool).await }) {
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
    }
}

// This function is classified under utils because it is called by enumerate_unenrolled_employees() and enumerate_enrolled_employees()
/// Function called by enumerate_unenrolled_employees() and enumerate_enrolled_employees() to retrieve employees from the database.
/// This returns a JSON object containing the list of employees from the database.
/// The enrolled parameter is used to determine if the list of employees requested are enrolled or unenrolled.
/// A true value for enrolled will return the list of enrolled employees, while a false value will return the list of unenrolled employees.
async fn query_employees(pool: &MySqlPool, enrolled: bool) -> Result<String, String> {
    if enrolled {
        let result = match sqlx::query!("CALL enumerate_enrolled_employees_json")
            .fetch_all(pool)
            .await
        {
            Ok(result) => result,
            Err(e) => match e {
                sqlx::Error::Database(e) => {
                    return Err(e.message().to_string());
                }

                _ => {
                    return Err(e.to_string());
                }
            },
        };

        if result.is_empty() {
            return Err("No unenrolled employees found".to_string());
        }

        let mut enrolled_employees: String = String::from("");

        for row in result.iter() {
            let json = row.get::<serde_json::Value, usize>(0);
            enrolled_employees = json.to_string();
        }

        Ok(enrolled_employees)
    } else {
        let result = match sqlx::query!("CALL enumerate_unenrolled_employees_json")
            .fetch_all(pool)
            .await
        {
            Ok(result) => result,
            Err(e) => match e {
                sqlx::Error::Database(e) => {
                    return Err(e.message().to_string());
                }

                _ => {
                    return Err(e.to_string());
                }
            },
        };

        if result.is_empty() {
            return Err("No unenrolled employees found".to_string());
        }

        let mut unenrolled_employees: String = String::from("");

        for row in result.iter() {
            let json = row.get::<serde_json::Value, usize>(0);
            unenrolled_employees = json.to_string();
        }

        Ok(unenrolled_employees)
    }
}

/*
    Manage Fingerprints and Verify Section
    Contains functions used at the Manage Fingerprints section.
    As verifying the fingerprints are included at Manage Fingerprints, functions involving verify_sync() fall under here.
*/

/// Function used to enumerate enrolled employees which will be candidates for fingerprint management and verification.
/// This function returns a JSON object containing the list of enrolled employees.
/// For errors, it returns a JSON object with an error message.
#[tauri::command]
fn enumerate_enrolled_employees(pool: State<ManagedMySqlPool>) -> String {
    let pool = pool.0.as_ref().unwrap();

    futures::executor::block_on(async {
        match query_employees(pool, true).await {
            Ok(result) => result,
            Err(e) => json!({
                "error": format!("Could not enumerate enrolled employees: {}",e)
            })
            .to_string(),
        }
    })
}

/// Deletes a fingerprint from the database.
/// However, this function does not affect the preloaded fingerprints (it does not affect the preloaded fingerprints unless they are reloaded)
#[tauri::command]
fn delete_fingerprint(emp_id: String, pool: State<ManagedMySqlPool>) -> String {
    println!("Deleting fingerprint for {}", emp_id);

    let pool = pool.0.as_ref().unwrap();

    futures::executor::block_on(async {
        match delete_fingerprint_from_db(&emp_id, pool).await {
            Ok(result) => {
                println!("Deleted {} rows", result.rows_affected());
                json!({
                    "responsecode" : "success",
                    "body" : "Fingerprint deleted successfully"
                })
                .to_string()
            }
            Err(e) => json!({
                "responsecode" : "failure",
                "body" : format!("Error deleting fingerprint: {}", e)
            })
            .to_string(),
        }
    })
}

/// Function called by delete_fingerprint() to delete fingerprint from the database.
async fn delete_fingerprint_from_db(
    emp_id: &str,
    pool: &MySqlPool,
) -> Result<MySqlQueryResult, String> {
    //delete fingerprint from database

    let result = match sqlx::query!("DELETE FROM enrolled_fingerprints WHERE emp_id = ?", emp_id)
        .execute(pool)
        .await
    {
        Ok(result) => result,
        Err(e) => match e {
            sqlx::Error::Database(e) => {
                return Err(e.message().to_string());
            }

            _ => {
                return Err(e.to_string());
            }
        },
    };

    Ok(result)
}

/// Takes an employee's employee ID and scans a fingerprint.
/// Returns a json object for almost every error that may occur.
/// The json object contains a responsecode and body.
/// On the very last section of this function lies the cases where the fingerprint scanning is successful and can either be a successful verification or not.
/// The rest before that section are typically errors.
#[tauri::command]
fn verify_fingerprint(
    emp_id: String,
    device: State<FpDeviceManager>,
    managed: State<FpDeviceManager>,
) -> String {
    println!("Verifying fingerprint for {}", emp_id);

    if device.0.is_none() {
        //if there is no fingerprint scanner plugged in
        return json!({
            "responsecode": "failure",
            "body": "Device could not be opened. Please try plugging in your fingerprint scanner and restarting the app.",
        }).to_string();
    }

    {
        let mut cancellable =
            futures::executor::block_on(async { managed.1.as_ref().unwrap().write().await });
        if cancellable.is_cancelled() {
            *cancellable = Cancellable::new();
        }
    }

    // let fp_scanner = match device.0.as_ref().unwrap().blocking_read() { //match device.0.as_ref().unwrap().lock() {
    //     Ok(fp_scanner) => fp_scanner,
    //     Err(e) => {
    //         return json!({
    //             "responsecode": "failure",
    //             "body": format!("Could not retrieve fingerprint scanner due to Mutex Poisoning. Error: {}", e.to_string()),
    //         })
    //         .to_string();
    //     }
    // };

    let fp_scanner = device.0.as_ref().unwrap().blocking_read();

    //try to open fingerprint scanner
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

    let fprint_list = match managed.2.as_ref().unwrap().lock() {
        //get the list of fingerprints
        Ok(fprint_list) => fprint_list, //try to retrieve the list of fingerprints
        Err(e) => {
            return json!({
                "responsecode": "failure",
                "body": format!("Could not retrieve the list of fingerprints. Error: {}", e.to_string()),
            })
            .to_string();
        }
    };

    //find the fingerprint in the list that matches the current emp_id from the preloaded fingerprints
    let fprint = match fprint_list.iter().find(|fprint| {
        fprint
            .username()
            .expect("List of fingerprints should have a username")
            == emp_id
    }) {
        Some(fprint) => fprint,
        None => {
            return json!({ //normally, this error shouldn't happen
                "responsecode": "failure",
                "body": "Fingerprint not found among the preloaded fingerprints. Please try scanning again, or enroll first.",
            })
            .to_string();
        }
    };

    let verify_result: bool;
    {
        let cancellable =
            futures::executor::block_on(async { managed.1.as_ref().unwrap().read().await });

        //verify the scanned fingerprint with verify_sync, it returns false if the fingerprint does not match the selected fingerprint, and returns true when matched
        verify_result = match fp_scanner.verify_sync(
            fprint,
            Some(&cancellable),
            Some(match_cb),
            None,
            Some(&mut new_print),
        ) {
            Ok(verify_result) => verify_result,
            Err(e) => {
                fp_scanner
                    .close_sync(None)
                    .expect("Could not close the fingerprint scanner");
                if cancellable.is_cancelled() {
                    return json!({
                        "responsecode": "failure",
                        "body": format!("Fingerprint scan cancelled"),
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

    if verify_result {
        json!({
            "responsecode": "success",
            "body": "Fingerprint verified successfully", //fingerprint matched
        })
        .to_string()
    } else {
        json!({
            "responsecode": "failure",
            "body": "Fingerprint did not match", //fingerprint did not match
        })
        .to_string()
    }
}

/*
    Enroll Section
*/

/// Function used to enumerate unenrolled employees which will be candidates for enrollment.
/// This function returns a JSON object containing the list of unenrolled employees.
/// For errors, it returns a JSON object with an error message.
#[tauri::command]
fn enumerate_unenrolled_employees(pool: State<ManagedMySqlPool>) -> String {
    let pool = pool.0.as_ref().unwrap();

    futures::executor::block_on(async {
        match query_employees(pool, false).await {
            Ok(result) => result,
            Err(e) => json!({
                "error": format!("Could not enumerate unenrolled employees: {}",e)
            })
            .to_string(),
        }
    })
}

/// Function called to enroll a fingerprint into the database.
/// The employee ID is passed in as an argument as emp.
#[tauri::command]
fn enroll_proc(
    emp: String,
    device: State<FpDeviceManager>,
    managed: State<FpDeviceManager>,
    pool: State<ManagedMySqlPool>,
) -> String {
    //function that is called when scanning a fingerprint for enrollment

    if device.0.is_none() {
        return json!({
            "responsecode": "failure",
            "body": "Device could not be opened. Please try plugging in your fingerprint scanner and restarting the app.",
        }).to_string();
    }

    {
        let mut cancellable =
            futures::executor::block_on(async { managed.1.as_ref().unwrap().write().await });
        if cancellable.is_cancelled() {
            *cancellable = Cancellable::new();
        }
    }

    // let fp_scanner = match device.0.as_ref().unwrap().lock() {
    //     Ok(fp_scanner) => fp_scanner,
    //     Err(_) => {
    //         return json!({
    //           "responsecode" : "failure",
    //           "body" : "Could not get device",
    //         })
    //         .to_string()
    //     }
    // };

    let fp_scanner = device.0.as_ref().unwrap().blocking_read();

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

    //set the username of the template to the employee ID to which the fingerprint belongs to
    template.set_username(&emp.to_string());

    println!(
        "Username of the fingerprint: {}",
        template
            .username()
            .expect("Username should be included here")
    );

    //let counter = Arc::new(Mutex::new(0)); //a counter for the current scanning phase of the enrollment process

    // let new_fprint = match fp_scanner.enroll_sync(template, None, Some(enroll_cb), None) {
    //     Ok(new_fprint) => new_fprint,
    //     Err(_) => {
    //         fp_scanner
    //             .close_sync(None)
    //             .expect("Could not close fingerprint scanner");
    //         return json!({
    //           "responsecode" : "failure",
    //           "body" : "Could not enroll fingerprint",
    //         })
    //         .to_string();
    //     }
    // };

    let new_fprint: FpPrint;
    {
        let cancellable =
            futures::executor::block_on(async { managed.1.as_ref().unwrap().read().await });
        //identify the scanned fingerprint with identify_sync, it returns nothing if the fingerprint is not in the database, and returns a fingerprint when matched
        new_fprint = match fp_scanner.enroll_sync(
            template,
            Some(&cancellable),
            Some(enroll_cb),
            None,
        ) {
            Ok(print) => print,
            Err(e) => {
                fp_scanner
                    .close_sync(None)
                    .expect("Could not close the fingerprint scanner");
                if cancellable.is_cancelled() {
                    return json!({
                        "responsecode": "failure",
                        "body": format!("Fingerprint scan cancelled"),
                    })
                    .to_string();
                } else {
                    return json!({
                    "responsecode": "failure",
                    "body": format!("Could not enroll fingerprint due to an error: {}", e.to_string()),
                    }).to_string();
                }
            }
        };
    }

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

    //println!("Total enroll stages: {}", counter.lock().unwrap());

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

    let pool = pool.0.as_ref().unwrap();

    futures::executor::block_on(async {
        match save_fprint_to_db(&emp, new_fprint, pool).await {
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

/// Function called by enroll_proc() to save fingerprint in the database.
async fn save_fprint_to_db(
    emp_id: &String,
    fprint: Vec<u8>,
    pool: &MySqlPool,
) -> Result<(), String> {
    // //save a fingerprint in the database to be associated with an employee id

    //query the record_attendance_by_empid stored procedure (manual attendance)
    match sqlx::query!("CALL save_fprint(?,?)", emp_id, fprint)
        .execute(pool)
        .await
    {
        Ok(row) => {
            match row.rows_affected() {
                //check how many rows were affected by the stored procedure that was previously queried
                0 => println!("No rows affected"),
                _ => println!("Rows affected: {}", row.rows_affected()),
            }
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

    Ok(()) //return from the function with no errors
}

/// A callback function for the enroll function which shows the current stage of the enroll process.
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

/*
    Attendance/Identify Section:
    All functions involving taking attendance, be it manual or by fingerprint scanner (through identify_sync()) are located in this section.
*/

/// A function for manual attendance where an employee puts their employee ID and takes manual attendance with it.
#[tauri::command]
fn manual_attendance(emp: String, pool: State<ManagedMySqlPool>) -> String {
    //manual attendance where an employee puts their employee ID and takes manual attendance with it
    println!("Entering manual attendance");
    println!("Emp: {}", emp);
    if emp.len() > 9 {
        return json!({
            "responsecode" : "failure",
            "body" : "Employee ID should be 9 characters or less. e.g. 12-345-67.",
        })
        .to_string();
    }

    let pool = pool.0.as_ref().unwrap();

    let row = futures::executor::block_on(async { record_attendance(&emp, true, pool).await }); //query_record_attendance(&emp_num).await

    let output = if row.is_ok() {
        let row = row.unwrap();
        let row_emp_id = row.get::<String, usize>(0);
        println!("Emp ID: {}", row_emp_id);
        let row_fname = row.get::<String, usize>(1);
        println!("Fname: {}", row_fname);
        let row_lname = row.get::<String, usize>(2);
        println!("Lname: {}", row_lname);
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
        // let row_attendance_status = match row.try_get::<u64, usize>(5) {
        //     Ok(status) => status.to_string(),
        //     Err(e) => match e {
        //         sqlx::Error::ColumnNotFound(_) => {
        //             println!("Column not found");
        //             "error".to_string()
        //         }
        //         sqlx::Error::ColumnDecode { index, source } => {
        //             println!("Column decode error: {} at index {}", source, index);
        //             "error".to_string()
        //         }
        //         _ => {
        //             println!("Unknown error: {}", e);
        //             "error".to_string()
        //         }
        //     },
        // };
        // println!("Attendance Status: {}", row_attendance_status);

        let row_attendance_message = match row.try_get::<String, usize>(5) {
            Ok(message) => message,
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
        println!("Attendance Message: {}", row_attendance_message);

        //return the json containing the employee and the attendance details
        json!({
           "responsecode" : "success",
           "body" : [
                row_emp_id,
                row_fname,
                row_lname,
                row_time,
                row_date,
                row_attendance_message,//row_attendance_status,
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

/// Function call to cancel start_identify().
#[tauri::command]
fn cancel_function(managed: State<FpDeviceManager>) {
    managed.cancel_managed();
}

/// Function used to identify a scanned fingerprint from the list of fingerprints stored in the database.
/// The aforementioned list of fingerprints is retrieved via the "managed" state variable.
/// Once identified, it takes the attendance of the employee. Otherwise, it returns an error.
/// For both cases, the function returns a json object containing the response code and the body.
#[tauri::command]
fn start_identify(
    device: State<FpDeviceManager>,
    managed: State<FpDeviceManager>,
    pool: State<ManagedMySqlPool>,
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

    //Access the fingerprint scanner
    // let fp_scanner = match device.0.as_ref().unwrap().lock() {
    //     Ok(fp_scanner) => fp_scanner,
    //     Err(e) => {
    //         return json!({
    //             "responsecode": "failure",
    //             "body": format!("Could not retrieve fingerprint scanner due to Mutex Poisoning. Error: {}", e.to_string()),
    //         })
    //         .to_string();
    //     }
    // };

    let fp_scanner = device.0.as_ref().unwrap().blocking_read();

    //Try to open fingerprint scanner
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

    let fprint_list = match managed.2.as_ref().unwrap().lock() {
        //get the list of fingerprints
        Ok(fprint_list) => fprint_list, //try to retrieve the list of fingerprints
        Err(e) => {
            return json!({
                "responsecode": "failure",
                "body": format!("Could not retrieve the list of fingerprints. Error: {}", e.to_string()),
            })
            .to_string();
        }
    };

    let print_identified: Option<FpPrint>; //initially None but is irrelevant as it will be overwritten later
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
                        "body": format!("Fingerprint scan cancelled"),
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

    let pool = pool.0.as_ref().unwrap();

    if print_identified.is_some() {
        //put another check sa db side if the preloaded fprint is in the db
        let fprint = print_identified.expect("Print should be able to be unwrapped here");
        let emp_id = fprint.username();
        match emp_id {
            Some(emp_id) => {
                futures::executor::block_on(async {
                    println!("emp_id of the fingerprint: {}", emp_id);
                    println!("Before recording attendance");
                    let result = record_attendance(&emp_id, false, pool).await; //record_attendance(&emp_id, false).await;
                    if result.is_ok() {
                        let row = result.expect("MySqlRow should be able to be unwrapped here");
                        let row_emp_id = row.get::<String, usize>(0);
                        let row_fname = row.get::<String, usize>(1);
                        let row_lname = row.get::<String, usize>(2);
                        let row_date = row.get::<time::Date, usize>(3).to_string();
                        let row_time = row.get::<time::Time, usize>(4).to_string();
                        //let row_attendance_status = row.get::<u16, usize>(5);
                        let row_attendance_message = row.get::<String, usize>(5);

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
                                row_attendance_message,//row_attendance_status,
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

/// This function is a callback function that is called when a scanned fingerprint is to be matched with previously enrolled fingerprints.
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

/// This function is called by both start_identify() and manual_attendance() to record the attendance of an employee to the database.
async fn record_attendance(
    emp_id: &str,
    manual_attendance: bool,
    pool: &MySqlPool,
) -> Result<MySqlRow, String> {
    //record attendance by emp_id (String type, fingerprint attendance)
    println!("recording attendance manually");

    //record the attendance
    let row: Option<MySqlRow> = match manual_attendance {
        true => {
            match sqlx::query!("CALL record_attendance_by_empid(?)", emp_id)
                .fetch_one(pool)
                .await
            {
                Ok(row) => Some(row),
                Err(e) => match e {
                    sqlx::Error::Database(e) => {
                        return Err(e.message().to_string());
                    }

                    _ => {
                        return Err(e.to_string());
                    }
                },
            }
        }
        false => {
            match sqlx::query!("CALL check_fprint_and_record_attendance(?)", emp_id)
                .fetch_one(pool)
                .await
            {
                Ok(row) => Some(row),
                Err(e) => match e {
                    sqlx::Error::Database(e) => {
                        return Err(e.message().to_string());
                    }

                    _ => {
                        return Err(e.to_string());
                    }
                },
            }
        }
    };

    Ok(row.unwrap())
}
