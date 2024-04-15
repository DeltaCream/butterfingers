// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use std::{
    sync::Arc,
    thread::{self, current},
};
use serde_json::{json, Value};
use tauri::{App, AppHandle, Manager, State, Window};
// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
use std::{
    env,
    fmt::format,
    fs::{self, OpenOptions},
    io::{self, BufReader, Read},
    sync::Mutex,
};

use libfprint_rs::{FpContext, FpDevice, FpPrint};

use sqlx::Row;

use sqlx::{mysql::MySqlRow, types::time, MySqlPool};
use tokio::runtime::{Builder, Runtime};
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}
#[derive(Clone, serde::Serialize)]
struct Payload {
    message: String,
}

#[tauri::command]
fn test_function(emp: u64) -> String {
    println!("Entering test function");
    let output = json!({
        "responsecode" : "success",
        "body" : emp,
    }).to_string();
    output
}

#[tauri::command]
fn manual_attendance(emp: String) -> String {
    println!("Entering manual attendance");
    println!("Emp: {}", emp);

    let emp_num = match emp.trim().parse::<u64>() {
        Ok(num) => num,
        Err(_) => return json!({
            "responsecode" : "failure",
            "body" : "Invalid employee ID",
        }).to_string(),
    };
    println!("asdsadsad");
    let mut output: Value = Default::default();
    let row = futures::executor::block_on(async {
        query_record_attendance(&emp_num).await
    });

    if row.is_ok() {
        let row = row.unwrap();
        let row_emp_id = row.get::<u64, usize>(0);
        let row_fname = row.get::<String, usize>(1);
        let row_lname = row.get::<String, usize>(2);
        let row_date = row.get::<time::Date, usize>(3).to_string();
        let row_time = row.get::<time::Time, usize>(4).to_string();
        let row_attendance_status = row.get::<u16, usize>(5);

        output = json!({
            "responsecode" : "success",
            "body" : [
                 row_emp_id,
                 row_fname,
                 row_lname,
                 row_time,
                 row_date,
                 row_attendance_status,
            ] 
         });
    } else {
        output = json!({
            "responsecode" : "failure",
            "body" : row.err().unwrap().to_string(),
        });
    }

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
fn start_identify(device: State<Note>) -> String {
    //let mut fun_result: Option<String> = Some(String::from(""));
    let mut fun_result: Value = Default::default();
    println!("entering verify mode!");

    let fp_scanner = device.0.lock().unwrap();

    let scanner_result = fp_scanner
        .open_sync(None);

    match scanner_result {
        Ok(()) => {
            println!("Fingerprint scanner opened!");
        },
        Err(e) => {
            return json!({
                "responsecode": "failure",
                "body": format!("Could not open fingerprint scanner. Error: {}", e.to_string()),
            }).to_string();
        }
    }    

    //Open the fingerprint scanner
    println!("Opening fingerprint scanner...");
    // fp_scanner.open_sync(None).expect("Device could not be opened. Please try plugging in your fingerprint scanner.");
    println!("Fingerprint scanner opened!");

    // Get a list of all entries in the folder
    let entries = fs::read_dir(
        dirs::home_dir()
            .expect("Home directory could not be found")
            .join("print/"),
    )
    .expect("Could not read the directory");

    // Extract the filenames from the directory entries and store them in a vector
    let file_names: Vec<String> = entries
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            if path.is_file() {
                path.file_name()?.to_str().map(|s| s.to_owned())
            } else {
                None
            }
        })
        .collect();
    // Get a list of all entries in the folder
    let entries = fs::read_dir(
        dirs::home_dir()
            .expect("Home directory could not be found")
            .join("print/"),
    )
    .expect("Could not read the directory");

    // Extract the filenames from the directory entries and store them in a vector
    let file_names: Vec<String> = entries
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            if path.is_file() {
                path.file_name()?.to_str().map(|s| s.to_owned())
            } else {
                None
            }
        })
        .collect();
    // Print the list of files (for debugging purposes, commented out on production)
    // println!("File names: {:?}", file_names);

    // Iterate over the file names
    let fingerprints = file_names
        .iter()
        .map(|filename| {
            //for every file name
            //go to the directory where the file will be placed
            let fpprint_file = OpenOptions::new()
                .read(true)
                .open(
                    dirs::home_dir()
                        .expect("Home directory could not be found")
                        .join(format!("print/{}", filename)),
                )
                .expect("Could not read the fingerprint file");

            //create a buffer for the files
            let mut reader = BufReader::new(fpprint_file);
            let mut buffer = Vec::new();

            //read file into buffer vector
            reader
                .read_to_end(&mut buffer)
                .expect("Could not retrieve contents of file");

            //deserialize the fingerprint stored in the file
            let deserialized_print = FpPrint::deserialize(&buffer);

            //retrieve the enrolled print from deserialized_print
            deserialized_print.expect("Could not unwrap the deserialized print")
            //let enrolled_print = deserialized_print.expect("Could not unwrap the deserialized print");
        })
        .collect::<Vec<FpPrint>>();

    for (i, fingerprint) in fingerprints.iter().enumerate() {
        //print the fingeprint number (on the array of fingerprints returned) and its corresponding username
        println!(
            "Username for Fingerprint #{}: {:?}",
            i,
            fingerprint
                .username()
                .expect("Fingerprint username could not be retrieved")
        );
    }
    //print that the fingerprints are retrieved (for debugging purposes, commented out on production)
    println!("Fingerprints retrieved");

    let mut new_print = FpPrint::new(&fp_scanner);
    println!("Please scan your fingerprint");

    println!("Before identify_sync call");
    let print_identified = fp_scanner
        .identify_sync(
            &fingerprints,
            None,
            Some(match_cb),
            None,
            Some(&mut new_print),
        )
        .expect("Fingerprint could not be identified due to an error");
    println!("After identify_sync call");
    // let rt = tokio::runtime::Handle::current(); //Runtime::new().expect("Failed to create Tokio runtime");
    // tauri::async_runtime
    if print_identified.is_some() {
        let fprint = print_identified.expect("Print could not be unwrapped");
        let uuid = fprint.username();
        match uuid {
            Some(uuid) => {
                futures::executor::block_on(async {
                    println!("UUID of the fingerprint: {}", uuid);
                    println!("Before recording attendance");
                    let result = record_attendance(&uuid).await;
                    if result.is_ok() {
                        let msg = format!(
                            "Attendance recorded for {}\n",
                            employee_name_from_uuid(&uuid).await
                        );
                        println!("{}", msg);
                        let row = result.unwrap();
                        let row_emp_id = row.get::<u64, usize>(0);
                        let row_fname = row.get::<String, usize>(1);
                        let row_lname = row.get::<String, usize>(2);
                        let row_date = row.get::<time::Date, usize>(3).to_string();
                        let row_time = row.get::<time::Time, usize>(4).to_string();
                        let row_attendance_status = row.get::<u16, usize>(5);

                        // fun_result = Some(msg);
                        fun_result = json!({
                            "responsecode": "success",
                            "body": [
                                row_emp_id,
                                row_fname,
                                row_lname,
                                row_time,
                                row_date,
                                row_attendance_status,
                            ]
                        });
                    } else {
                        //show that attendance could not be recorded
                        println!("Attendance could not be recorded\n");
                        fun_result = json!({
                            "responsecode": "failure",
                            "body": result.err().unwrap().to_string(),
                        });
                    }
                });
            },
            None => {
                println!("No user associated with fingerprint"); //uuid did not contain a string (essentially None acts as a null value)
                fun_result = json!({
                    "responsecode": "failure",
                    "body": "No user associated with fingerprint",
                });
            }
        }
    } else {
        println!("No matching fingerprint could be found.");
        fun_result = json!({
            "responsecode": "failure",
            "body": "No matching fingerprint could be found.",
        })
    }
    fp_scanner.close_sync(None).unwrap();
    fun_result.to_string()
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

async fn query_record_attendance(emp_id: &u64) -> Result<MySqlRow, Box<dyn std::error::Error>> {
    //setup involving the .env file
    dotenvy::dotenv()?;
    //connect to the database
    let pool = MySqlPool::connect(&env::var("DATABASE_URL")?).await?;
    //query the record_attendance_by_empid stored procedure (manual attendance)
    let result = sqlx::query!("CALL record_attendance_by_empid(?)", emp_id)
        .execute(&pool) //execute the query
        .await?; //wait for the query to finish (some asynchronous programming shenanigans)
                 //if the query was successful
    // if result.rows_affected() > 0 {
    //     println!("Attendance manually recorded"); //print that the attendance was recorded
    // }
    
    let uuid_query = sqlx::query!("SELECT fprint_uuid from enrolled_fingerprints where emp_id = ?", emp_id)
        .fetch_one(&pool)
        .await
        .expect("Could not retrieve uuid");
        
    let uuid = uuid_query.fprint_uuid;   //.get::<String, usize>(0);

    println!("UUID: {}", uuid);

    let row = sqlx::query!("CALL get_latest_attendance_record(?)", uuid)
        .fetch_one(&pool)
        .await
        .expect("Could not retrieve latest attendance record");

    pool.close().await; //close connection to database
    Ok(row) //return from the function with no errors
}


async fn employee_name_from_uuid(uuid: &str) -> String {
    dotenvy::dotenv().unwrap();
    let pool = MySqlPool::connect(&env::var("DATABASE_URL").unwrap())
        .await
        .unwrap();
    let result = sqlx::query!(r#"SELECT enrolled_fingerprints.fprint_uuid AS "uuid", employee.fname AS "fname", employee.mname AS "mname", employee.lname AS "lname" 
    FROM enrolled_fingerprints JOIN employee USING(emp_id) WHERE fprint_uuid = ?"#, uuid)
        .fetch_one(&pool)
        .await
        .expect("Could not retrieve employee name from uuid");
    pool.close().await;
    match (result.fname, result.mname, result.lname) {
        (fname, Some(mname), lname) => format!("{} {} {}", fname, mname, lname),
        (fname, None, lname) => format!("{} {}", fname, lname),
    }
}

async fn employee_name_from_empid(emp_id: &u64) -> String {
    dotenvy::dotenv().unwrap();
    let pool = MySqlPool::connect(&env::var("DATABASE_URL").unwrap())
        .await
        .unwrap();
    let result = sqlx::query!(//r#"SELECT production_staff.emp_id AS "emp_id", 
    //employee.fname AS "fname", employee.mname AS "mname", employee.lname AS "lname" FROM production_staff JOIN employee USING(emp_id) WHERE emp_id = ?"#, emp_id)
        r#"SELECT employee.emp_id AS "emp_id", employee.fname AS "fname", employee.mname AS "mname", employee.lname AS "lname" FROM employee WHERE role_code = 2 AND emp_id = ?"#, emp_id)
        .fetch_one(&pool)
        .await
        .expect("Could not retrieve employee name from employee id");
    pool.close().await;
    match (result.fname, result.mname, result.lname) {
        (fname, Some(mname), lname) => format!("{} {} {}", fname, mname, lname),
        (fname, None, lname) => format!("{} {}", fname, lname),
    }
}

//function below is a callback function that is called when a scanned fingerprint is matched with previously enrolled fingerprints
pub fn match_cb(
    _device: &FpDevice,
    matched_print: Option<FpPrint>,
    print: FpPrint,
    _error: Option<libfprint_rs::GError>,
    _data: &Option<()>,
) {
    if let Some(matched_print) = &matched_print {
        //get the matched print
        //print the matched print's username
        //println!("Matched print: {:#}", matched_print.username().expect("Fingerprint username could not be retrieved"));

        //set the matched print's username to the print
        // if print.username().is_some() {
        //     println!("Print: {:#}", &print.username().expect("Fingerprint username could not be retrieved"));
        // } else {
        //     println!("Print does not have a username");
        // }

        //set the scanned fingerprint's username to the matched print's username
        //(because the scanned fingerprint was matched with the previously enrolled fingerprint,
        //and currently, the scanned fingerprint has no username)
        print.set_username(
            &matched_print
                .username()
                .expect("Username could not be retrieved"),
        );

        println!("Matched");

        //print the scanned fingerprint's username for debugging purposes
        //(by this point, the scanned fingerprint should already have the same username as the matched fingerprint)
        //println!("Print username: {:#}", &print.username().expect("Fingerprint username could not be retrieved"));
    } else {
        //if matched_print is None (null value)
        //print that no fingerprint was matched with the scanned fingerprint
        println!("Not matched");
    }
}

async fn record_attendance(uuid: &str) -> Result<MySqlRow, String> {
    //setup involving the .env file
    println!("recording attendance");
    match dotenvy::dotenv() {
        Ok(_) => (),
        Err(e) => return Err("Failed to load .env file".to_string()),
    }

    let database_url = match env::var("DATABASE_URL") {
        Ok(url) => url,
        Err(_) => return Err("DATABASE_URL not set".to_string()),
    };

    //connect to the database
    let pool = match MySqlPool::connect(&database_url).await {
        Ok(pool) => pool,
        Err(e) => return Err(e.to_string()),
    };

    //query the record_attendance stored procedure (non-manual attendance)
    let result = sqlx::query!("CALL record_attendance(?)", uuid)
        //.execute(&pool) //execute the query
        .execute(&pool)
        .await;
        //.expect("Could not record attendance");
        
    match result {
        Ok(_) => (),
        Err(e) => {
            return Err(e.to_string());
        }
    }
        //.expect("Could not record attendance"); 
                //wait for the query to finish (some asynchronous programming shenanigans)
                 //if the query was successful
                 // if result.rows_affected() > 0 {
                 //     println!("Attendance recorded"); //print that the attendance was recorded
                 // }
    

    // let row = sqlx::query!(r#"SET time_zone = "+08:00";
    // IF(EXISTS(SELECT emp_id, fprint_uuid from enrolled_fingerprints where fprint_uuid = uuid)) then
    //     SET @emp_id = (SELECT emp_id from enrolled_fingerprints where fprint_uuid = uuid);
    //     select employee.emp_id, 
    //     employee.fname, 
    //     employee.lname, 
    //     attendance_records.attendance_date,  attendance_records.attendance_time, attendance_status_code from employee join attendance_records where employee.emp_id = @emp_id and attendance_date = DATE(NOW()) ORDER BY attendance_date, record_no DESC LIMIT 1;"#, uuid)
    let row = sqlx::query!("CALL get_latest_attendance_record(?)", uuid)
        .fetch_one(&pool)
        .await;

    if row.is_err() {
        return Err(row.unwrap_err().to_string());
    }
        //.expect("Could not retrieve latest attendance record");

    // if row.is_empty() {

    // }

    // for (row_number, row) in result.iter().enumerate() {

    // }

    //let some_field = row.0;

    pool.close().await; //close connection to database
    //Ok(()) //return from the function with no errors
    // match (result.0, result.1, result.2, result.3, result.4, result.5) {
    //     (fname, Some(mname), lname) => format!("{} {} {}", fname, mname, lname),
    //     (fname, None, lname) => format!("{} {}", fname, lname),
    // }
    //Ok((result.0,result.1,result.2,result.3,result.4,result.5,result.6))
    // for row in result {
    //     println!("{} {} {}", row.0, row.1, row.2);
    // }
    //let result = result.fname;
    // match (row.0, row.1, row.2, row.3, row.4, row.5) {
    //     (fname, Some(mname), lname) => format!("{} {} {}", fname, mname, lname),
    //     (fname, None, lname) => format!("{} {}", fname, lname),
    // }
    // let field = row.get(0);
    // println!("{:?}", field);

    //Ok(row)
    Ok(row.ok().unwrap())
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

struct Note(Mutex<FpDevice>);

// impl Default for Note {
//     fn default() -> Self {
//         Self(Mutex::new(FpContext::new().devices().remove(0)))
//     }
// }

#[tokio::main]
async fn main() {
    tauri::Builder::default()
        .setup(|_app| Ok(()))
        .manage(Note(Mutex::new(FpContext::new().devices().remove(0))))
        .invoke_handler(tauri::generate_handler![greet, start_identify, manual_attendance])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
