use std::{
    env, 
    fs::OpenOptions,
    io::{self, Write}, 
    sync::{Arc, Mutex},
};

use libfprint_rs::{
    FpContext, 
    FpPrint, 
    FpDevice,
};

use sqlx::{
    MySqlPool,
    Row,
};

use uuid::Uuid;

use prettytable::{
    Table,
    Cell,
};

/*  algorithm:
    check for any unenrolled employees
    create a table showing which employees are unenrolled
    prompt user to select an employee
    open fingerprint scanner
    scan fingerprint
    enroll fingerprint
    store fingerprint in database
    close fingerprint scanner
*/

#[tokio::main]
async fn main() { //entry point of the program
    enroll_employee().await.unwrap();
}

async fn enroll_employee() -> anyhow::Result<()> { //list employees which are candidates for enrollment

    dotenvy::dotenv()?;
    let pool = MySqlPool::connect(&env::var("DATABASE_URL")?).await?; 
    let result = sqlx::query!("CALL enumerate_unenrolled_employees")
        .fetch_all(&pool)
        .await?;

    //print how many rows found for debugging purposes
    //println!("{} rows found", result.len());

    //check if result set is empty
    if result.is_empty() {
        println!("No unenrolled employees found");
        return Ok(());
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
    for (row_number, row) in result.iter().enumerate() {
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
    }
    // Print the table to the command line
    table.printstd();

    println!("Please enter the row number corresponding to the Employee you would like to enroll: "); //take input
    let mut line = String::new();
    let row_num;

    loop { //while (true)
        //read input
        io::stdin().read_line(&mut line).expect("Input should be read here");        
        if let Ok(num) = line.trim().parse::<usize>() {
            if result.get(num).is_some() { //get returns Some if the row exists
                row_num = num; //get the value inside the Some
                break; //terminate the loop
            } else { //get returns None if the row does not exist
                println!("Row number {num} does not exist in the table, please try again");
            }
        } else { //parse returns an error, will not allow negative numbers (as negative rows do not exist in the table)
            println!("Invalid input, please try again");
        }
        line.clear()
    }

    //retrieve result set
    let row_queried = &result.get(row_num)
        .expect("A row should be present here");

    //generates a random uuid
    let uuid = Uuid::new_v4();

    //enroll fingerprint
    let context = FpContext::new();
    let devices = context.devices();
    let fp_scanner = devices.first().expect("Devices could not be retrieved");

    // println!("{:#?}", fp_scanner.scan_type()); //print the scan type of the device (for debugging purposes)
    // println!("{:#?}", fp_scanner.features());  //print the features of the device (for debugging purposes)

    //open the fingerprint scanner
    fp_scanner.open_sync(None).expect("Device could not be opened");

    //create a template for the user
    let template = FpPrint::new(fp_scanner);

    //still not sure if the *kind* of finger is to be included as fingerprint metadata
    //template.set_finger(FpFinger::RightIndex);

    //set the username of the template to the uuid generated
    template.set_username(&uuid.to_string()); 

    //print username of the fingerprint template for debugging purposes
    //(fingerprint template SHOULD have a username at this point)
    println!("Username of the fingerprint: {}", template.username().expect("Username should be included here"));

    //counter for later use
    let counter = Arc::new(Mutex::new(0));

    //prompt for the user
    println!("Please put your right index finger onto the fingerprint scanner.");

    //scan a new fingerprint
    let new_fprint = fp_scanner
        .enroll_sync(template, None, Some(enroll_cb), None)
        .expect("Fingerprint could not be enrolled");

    //println!("new_print contents: {:#?}",new_fprint);   //print the FpPrint struct (for debugging purposes)
    //println!("new_print username: {:#?}",new_fprint.username().unwrap());   //print the username of the FpPrint (for debugging purposes)

    //close the fingerprint scanner
    fp_scanner.close_sync(None).expect("Device could not be closed");

    //show the total enroll stages (for debugging purposes)
    println!("Total enroll stages: {}", counter.lock().unwrap());

    //create a file to store the fingerprint in (at the root folder, which is securely located in the home directory)
    let mut file = OpenOptions::new()
                            .write(true)
                            .create(true)
                            .open(dirs::home_dir()
                                            .expect("Failed to get home directory")
                                            .join(format!("print/fprint_{}",uuid)))
                            .expect("Creation of file failed");

    //fingerprint serialized for storage at the file location
    file.write_all(&new_fprint.serialize().expect("Could not serialize fingerprint"))
        .expect("Error: Could not store fingerprint to the txt file");

    //get Employee ID
    let emp_id: u64 = row_queried.get::<u64, usize>(0);

    //call save_fprint_identifier stored procedure using the emp_id (from the row queried) and uuid (randomly generated and attached to the fingeprint)
    let insert = sqlx::query!("CALL save_fprint_identifier(?,?)",emp_id,uuid.to_string())
        .execute(&pool) //execute the stored prodcedure
        .await?; //wait for the query to finish

    match insert.rows_affected() { //check how many rows were affected by the stored procedure that was previously queried
        0 => println!("No rows affected"),
        _ => println!("Rows affected: {}", insert.rows_affected()),
    }
    
    pool.close().await; //close the connection to the database
    
    Ok(()) //return the function with no errors
}

//function below is a callback function for the enroll function's progress
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
    //print enroll stage of the enroll function
    println!("Progress_cb Enroll stage: {}", enroll_stage);
}

//function below is a callback function for the enroll function's stage
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