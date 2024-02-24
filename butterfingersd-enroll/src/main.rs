use std::{env, fs::{self, OpenOptions}, io::{self, Write}, num::IntErrorKind, sync::{Arc, Mutex}, path::PathBuf};

use libfprint_rs::{FpContext, FpPrint, FpDevice, FpFinger};

use sqlx::{MySqlPool, Row, Column};
use uuid::Uuid;

use prettytable::{Table, Cell};
use tokio;

async fn enroll_employee() -> anyhow::Result<()> { //list employees which are candidates for enrollment
    dotenvy::dotenv()?;
    let pool = MySqlPool::connect(&env::var("DATABASE_URL")?).await?; 
    let result = sqlx::query!("CALL enumerate_unenrolled_employees")
        .fetch_all(&pool)
        .await?;
    //sqlx::query!("select production_staff.emp_id As "Employee ID", employee.fname As "First Name",employee.lname As "Last Name" from production_staff join employee using(emp_id) where production_staff.emp_id not in (select emp_id from enrolled_fingerprints)") //backup query
    
    //transfer query contents to a vector (already done by line 11)
    println!("Query has been completed");
    for row in &result {
        let columns = row.columns();
        for col in columns {
            println!("Column name: {}", col.name());
        }
    }

    // Display the rows in a table format
    let mut table = Table::new();
    // Add table headers
    table.add_row(prettytable::Row::new(vec![
        Cell::new("Row Number"),
        Cell::new("Employee ID"),
        Cell::new("First Name"),
        Cell::new("Last Name"),
        // ... add more headers as needed
    ]));
    // Add row data to the table
    let mut row_number = 0;
    for row in &result {
        let emp_id: u64 = row.get::<u64, usize>(0);//&str>("Employee ID");//.expect("Employee ID should be found here"); //bigint unsigned
        let fname: String = row.get::<String, usize>(1); //&str>("fname");//.expect("First Name should be found here"); //varchar
        let lname: String = row.get::<String, usize>(2);//&str>("lname");//.expect("Last Name should be found here"); //varchar
        table.add_row(prettytable::Row::new(vec![
            Cell::new(&row_number.to_string()),
            Cell::new(&emp_id.to_string()),
            Cell::new(&fname.to_string()),
            Cell::new(&lname.to_string()),
            // ... add more cells with row data as needed
        ]));
        row_number += 1;
    }
    // Print the table to the command line
    table.printstd();

    println!("Please enter the row number corresponding to the Employee you would like to enroll: "); //take input
    let mut line = String::new();
    let mut row_num;
    loop {
        io::stdin().read_line(&mut line).expect("Input should be read here");
        // if let Err(e) = line.trim().parse::<i32>() {
        //     match e.kind() {
        //         IntErrorKind::Empty => {
        //             println!("Exiting...");
        //             break;
        //         }
        //         IntErrorKind::InvalidDigit => {
        //             println!("Invalid digit, try again")
        //         }
        //         error => {
        //             panic!("Unexpected error {error:?}")
        //         }
        //     }
        // }
        
        if let Ok(num) = line.trim().parse::<usize>() {
            match num {
                num => {
                    if let Some(row) = &result.get(num) {
                        row_num = num;
                        break;
                    } else { //get returns None if the row does not exist
                        println!("Row number {num} does not exist in the table, please try again");
                    }
                },
            }
        } else { //parse returns an error, will not allow negative numbers (as negative rows do not exist in the table)
            println!("Invalid input, please try again");
        }
        line.clear()
    }

    //retrieve result set
    let row_queried = &result.get(row_num)
        .expect("A row should be present here");

    let uuid = Uuid::new_v4(); //generates a random uuid

    //enroll fingerprint
    let context = FpContext::new();
    let devices = context.devices();
    let fp_scanner = devices.first().expect("Devices could not be retrieved");

    println!("{:#?}", fp_scanner.scan_type()); //print the scan type of the device
    println!("{:#?}", fp_scanner.features());  //print the features of the device

    fp_scanner.open_sync(None).expect("Device could not be opened");

    let template = FpPrint::new(fp_scanner);
    template.set_finger(FpFinger::RightIndex);
    template.set_username(&uuid.to_string()); //inputted the uuid generated

    println!("Username of the fingerprint: {}", template.username().expect("Username should be included here"));

    let counter = Arc::new(Mutex::new(0));

    println!("Please put your right index finger onto the fingerprint scanner.");

    let new_fprint = fp_scanner
        .enroll_sync(template, None, Some(enroll_cb), None)
        .unwrap();

    println!("new_print contents: {:#?}",new_fprint);   //print the FpPrint struct
    println!("new_print username: {:#?}",new_fprint.username().unwrap());   //print the username of the FpPrint

    fp_scanner.close_sync(None).expect("Device could not be closed");
    println!("Total enroll stages: {}", counter.lock().unwrap());

    //fs::create_dir("print").expect("Should create a directory called print");
    let mut file = OpenOptions::new().write(true).create(true).open(format!("print/fprint_{uuid}")).expect("Creation of file failed");//PathBuf::from("print/").join(format!("fprint_{uuid}")); //changed from File::create to OpenOptions::create
    //fingerprint serialized for storage
    //println!("Path: {}", file.display());
    file.write_all(&new_fprint.serialize().expect("Could not serialize fingerprint")).expect("Error: Could not store fingerprint to the txt file");

    let emp_id: u64 = row_queried.get::<u64, usize>(0);//&str>("emp_id"); //get Employee ID

    let insert = sqlx::query!("CALL save_fprint_identifier(?,?)",emp_id, uuid.to_string())
        .execute(&pool)
        .await?;

    match insert.rows_affected() {
        0 => println!("No rows affected"),
        _ => println!("Rows affected: {}", insert.rows_affected()),
    }
    
    pool.close().await;
    
    Ok(())
}

//enroll employee
//enroll production

// async fn enroll_employees() -> anyhow::Result<()> {
//     Ok(())
// }

// async fn select(query: &str) -> anyhow::Result<()> {
    
// }

// async fn attendance() -> anyhow::Result<()> {
//     dotenvy::dotenv()?;
//     let pool = MySqlPool::connect(&env::var("DATABASE_URL")?).await?;
    
//     let context = FpContext::new();
//     let devices = context.devices();
//     let fp_scanner = devices.first().expect("Devices could not be retrieved");

//     println!("{:#?}", fp_scanner.scan_type()); //print the scan type of the device
//     println!("{:#?}", fp_scanner.features());  //print the features of the device

//     fp_scanner.open_sync(None).expect("Device could not be opened");

//     let template = FpPrint::new(fp_scanner);
//     template.set_finger(FpFinger::RightIndex);
//     template.set_username("tbd"); //will input later

//     println!("Username of the fingerprint: {}", template.username().expect("Username should be included here"));

//     let counter = Arc::new(Mutex::new(0));

//     let new_fprint = fp_scanner
//         .enroll_sync(template, None, Some(enroll_cb), None)
//         .unwrap();

//     println!("new_print contents: {:#?}",new_fprint);   //print the FpPrint struct
//     println!("new_print username: {:#?}",new_fprint.username().unwrap());   //print the username of the FpPrint

//     fp_scanner.close_sync(None).expect("Device could not be closed");
//     println!("Total enroll stages: {}", counter.lock().unwrap());

//     //generate uuid
//     let uuid = "Hello UUID";


//     let attendance = sqlx::query!( //use query_as! later on //call prepared statement
//         r#"CALL record_attendance(?)"# //input uuid inside the record_attendance
//     ,uuid)
//     .execute(&pool)
//     .await?
//     .last_insert_id();
    
//     Ok(())
// }

#[tokio::main]
async fn main() {

    enroll_employee().await.unwrap();

    //attendance().await.unwrap();

    // println!("For whom will you scan the fingerprint?");

    // let mut user = String::new();
    // io::stdin().read_line(&mut user).expect("Failed to get username");

    // let context = FpContext::new();

    // let devices = context.devices();

    // let dev = devices.first().expect("Devices could not be retrieved");

    // println!("{:#?}", dev.scan_type()); //print the scan type of the device
    // println!("{:#?}", dev.features());  //print the features of the device


    // dev.open_sync(None).expect("Device could not be opened");

    // let template = FpPrint::new(dev);    //&dev); //&dev because dev might be used later
    // template.set_finger(FpFinger::RightIndex);
    // template.set_username("test");  //&user);

    // println!("Username of the fingerprint: {}", template.username().expect("Fingerprint username could not be retrieved"));

    // let counter = Arc::new(Mutex::new(0));

    // let new_print = dev
    //     .enroll_sync(template, None, None, Some(counter.clone())) //Some(progress_cb)
    //     .unwrap();

    // println!("new_print contents: {:#?}",new_print);   //print the FpPrint struct
    // println!("new_print username: {:#?}",new_print.username().unwrap());   //print the username of the FpPrint


    // fs::create_dir("print").expect("Should create a directory called print");
    // let mut file = OpenOptions::new().write(true).create(true).open("print/fprint").expect("Creation of file failed"); //changed from File::create to OpenOptions::create
    // //fingerprint serialized for storage
    // file.write_all(&new_print.serialize().expect("Could not serialize fingerprint")).expect("Error: Could not store fingerprint to the txt file");
    // //type analysis: new_print.serialize() returns Result<Vec<u8>, Error>
    // //calling expect unwraps the Result and returns either a Vec<u8> or an Error
    // //assuming that it *does* return Vec<u8>
    // //we can do type coercion using & to first get a reference of Vec<u8>, which is &Vec<u8>
    // //then convert the &Vec<u8> into a &[u8] via the AsRef trait which is implemented out of the box
    // //writeln!(&mut file, "{:#?}", new_print.serialize().expect("Could not serialize fingerprint")).expect("Error: Could not copy fingerprint to the txt file");

    // println!("Total enroll stages: {}", counter.lock().unwrap());

    //Enrolling a fingerprint example code
    // // Get context
    // let ctx = FpContext::new();
    // // Collect connected devices
    // let devices = ctx.devices();

    // // Get the first connected device
    // let dev = devices.get(0).unwrap();

    // // Open the device to start operations
    // dev.open_sync(None).unwrap();

    // // Create a template print
    // let template = FpPrint::new(&dev);
    // template.set_finger(FpFinger::RightRing);
    // template.set_username("test");

    // // User data that we will use on the callback function,
    // // to mutate the value of a counter, it must be wrapped in an Arc<Mutex<T>>
    // let counter = Arc::new(Mutex::new(0));

    // // Get the new print from the user
    // let _new_print = dev
    //     .enroll_sync(template, None, Some(progress_cb), Some(counter.clone()))
    //     .unwrap();

    // // Get the total of time the enroll callback was called
    // println!("Total enroll stages: {}", counter.lock().unwrap());

    //Verifying a fingerprint example code
    // Get devices
    // let ctx = FpContext::new();
    // let devices = ctx.devices();
    // let dev = devices.get(0).unwrap();
    // dev.open_sync(None).unwrap();

    // // Create a template print
    // let template = FpPrint::new(&dev);
    // let enrolled_print = dev
    //     .enroll_sync(template, None, Some(progress_cb), None)
    //     .unwrap();

    // // New print where we will store the next print
    // let mut new_print = FpPrint::new(&dev);

    // // Verify if the next print matches the previously enrolled print
    // let matched = dev
    //     .verify_sync(
    //         &enrolled_print,
    //         None,
    //         Some(match_cb),
    //         None,
    //         Some(&mut new_print),
    //     )
    //     .unwrap();
    // if matched {
    //     println!("Matched again");
    // }

    //Example code stops here

    // let context = FpContext::new();
    // let devices = context.devices();

    //Enrolling a new fingerprint
    // let context = FpContext::new();
    // let devices = context.devices();

    // let template = FpPrint::new(&devices.get(0).or());

    // let dev = devices.get(0).expect("No devices found");
    // dev.open_sync(None).expect("Should open synchronously with no problems"); //?

    // //adding a new fingerprint via enroll_sync
    // let template = FpPrint::new(dev);
    // template.set_username("Bruce Banner");
    // let new_print = dev
    //                                         .enroll_sync(template, None,
    //                                             Some(enroll_cb), 
    //                                             Some(10));
    // dev.close_sync(None).expect("Closing device encountered an error");
    // //template.set_username("Bruce Banner");
    // println!("{:?}", new_print);
    // println!("{}", new_print.unwrap().username().unwrap());


    //

    //Hello world for sanity check
    // println!("Hello, world!");
    // println!("Hello World");

    // //Verifying a fingerprint
    // let context = FpContext::new();
    // let devices = context.devices();

    // let dev = devices.get(0).expect("No devices found");
    // dev.open_sync(None).expect("Should work here");

    // // let enrolled_print = load_print_from_file();

    // // let match_res = dev.verify_sync(enrolled_print, None, None, None::<()>, None).expect("Some error encountered while verifying the fingerprint."); //?
}

pub fn progress_cb(
    _device: &FpDevice,
    enroll_stage: i32,
    _print: Option<FpPrint>,
    _error: Option<glib::Error>,
    data: &Option<Arc<Mutex<i32>>>,
) {
    if let Some(data) = data {
        let mut d = data.lock().unwrap();
        *d += 1;
    }
    println!("Progress_cb Enroll stage: {}", enroll_stage);
}

pub fn enroll_cb(
    device: &FpDevice, 
    enroll_stage: i32, 
    print: Option<FpPrint>, 
    error: Option<libfprint_rs::GError>, 
    data: &Option<i32>,
) {
    println!("Enroll_cb Enroll stage: {}", enroll_stage);
}

// pub fn match_cb(
//     _device: &FpDevice,
//     matched_print: Option<FpPrint>,
//     _print: FpPrint,
//     _error: Option<glib::Error>,
//     _data: &Option<()>,
// ) -> () {
//     if matched_print.is_some() {
//         println!("Matched");
//     } else {
//         println!("Not matched");
//     }
// }
