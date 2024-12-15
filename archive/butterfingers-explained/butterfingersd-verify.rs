use std::{
    env,
    fs::{self, OpenOptions},
    io::{self, BufReader, Read}
};

use libfprint_rs::{
    FpContext,
    FpPrint,
    FpDevice,
};

use sqlx::MySqlPool;

/* Harlan's initial algorithm
while true{
	scan for finger
	select the uuid of finger
	check if uuid exists in db
	if exists, call record_attendance(uuid)
	else error
}
*/


/* Final working algorithm with details
loop {
    check if manual attendance is needed based on the number of tries
    if yes,
        call manual_attendance
    else
        scan finger
        select uuid
        check if uuid exists in db
        SELECT emp_id from employee where uuid = ?, uuid.to_string()
        if exists, call record_attendance(uuid)
            else println!("Fingerprint does not exist in the database")
}
*/

#[tokio::main]
async fn main() {
    //Get FpContext to get devices
    let context = FpContext::new();
    //Use FpContext to get devices (returns a vector/array of devices)
    let devices = context.devices();
    //Get the first device (which, in this case, is the only device, and it is the fingerprint scanner)
    let fp_scanner = devices.first().expect("Devices could not be retrieved");
    
    //Open the fingerprint scanner
    fp_scanner.open_sync(None).expect("Device could not be opened");

    // Get a list of all entries in the folder
    let entries = fs::read_dir(dirs::home_dir()
                                        .expect("Home directory could not be found")
                                        .join("print/"))
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
    let fingerprints = file_names.iter().map(|filename| { //for every file name
        //go to the directory where the file will be placed
        let fpprint_file = OpenOptions::new()
                                    .read(true)
                                    .open(dirs::home_dir()
                                                    .expect("Home directory could not be found")
                                                    .join(format!("print/{}",filename)))
                                    .expect("Could not read the fingerprint file");

        //create a buffer for the files
        let mut reader = BufReader::new(fpprint_file);
        let mut buffer = Vec::new();

        //read file into buffer vector
        reader.read_to_end(&mut buffer).expect("Could not retrieve contents of file");

        //deserialize the fingerprint stored in the file
        let deserialized_print = FpPrint::deserialize(&buffer);

        //retrieve the enrolled print from deserialized_print
        deserialized_print.expect("Could not unwrap the deserialized print") //let enrolled_print = deserialized_print.expect("Could not unwrap the deserialized print");
    }).collect::<Vec<FpPrint>>();

    for (i, fingerprint) in fingerprints.iter().enumerate() {
        //print the fingeprint number (on the array of fingerprints returned) and its corresponding username
        println!("Username for Fingerprint #{}: {:?}", i, fingerprint.username().expect("Fingerprint username could not be retrieved"));
    }

    //print that the fingerprints are retrieved (for debugging purposes, commented out on production)
    println!("Fingerprints retrieved");

    let mut number_of_tries = 0;

    loop { //equivalent to while(true)
        if number_of_tries >= 3 { //if condition for manual attendance is satisfied
            loop {
                println!("Please manually enroll the employee's attendance");
                println!("What is the employee's id?");
                let mut buffer = String::new();
                match io::stdin().read_line(&mut buffer) {
                    Ok(_) => {
                        let emp_id = match buffer.trim().parse::<u64>() {
                            Ok(num) => num,
                            Err(_) => {
                                println!("Error: employee id could not be read");
                                continue;
                            },
                        };
                        let result = manual_attendance(&emp_id).await;
                        if result.is_ok() {
                            println!("Attendance manually recorded for {}", employee_name_from_empid(&emp_id).await);
                            number_of_tries = 0;
                            break;
                        } else {
                            println!("Attendance could not be recorded");
                        }
                    },
                    Err(_error) => {
                        println!("Error: employee id could not be read");
                    },
                }
            }
        } else { //if condition for manual attendance is not satisfied
            //println!("Before new_print declaration"); //for debugging purposes

            // The variable that will hold the new fingerprint
            let mut new_print = FpPrint::new(fp_scanner);

            //prompt for the user to scan their fingerprint
            println!("Please scan your fingerprint");

            //identify the scanned fingerprint from the list of fingerprints that were previously stored from enrollment
            let print_identified = fp_scanner.identify_sync(&fingerprints, None, Some(match_cb), None, Some(&mut new_print)).expect("Fingerprint could not be identified due to an error");
            
            if print_identified.is_some() { //if print_identified identified a fingerprint
                let fprint = print_identified.expect("Print could not be unwrapped");
                //retrieves the username of the fingerprint (remember that the username is part of the fingerprint's metadata)
                let uuid = fprint.username();
                match uuid { //switch statement
                    Some(uuid) => { //if the uuid contained a string (which is the username)
                        //print the uuid of the fingerprint
                        println!("UUID of the fingerprint: {}", uuid);
                        //call record_attendance function (non-manual attendance)
                        let result = record_attendance(&uuid).await;
                        if result.is_ok() { //if nothing wrong happened with record_attendance function
                            //show that attendance was recorded for "employee name"
                            println!("Attendance recorded for {}", employee_name_from_uuid(&uuid).await);
                            //reset number of tries
                            number_of_tries = 0;
                        } else { //if something wrong happened with record_attendance function
                            //show that attendance could not be recorded
                            println!("Attendance could not be recorded");
                            //increment number of tries, possibly resulting to manual attendance in the next iteration of the loop
                            number_of_tries += 1;
                        }
                    },
                    None => println!("UUID could not be retrieved"), //uuid did not contain a string (essentially None acts as a null value)
                }
                //println!("UUID of the fingerprint: {}", uuid);
            } else { //print_identified did not identify a fingerprint
                println!("No matching fingerprint could be found");
                number_of_tries += 1;
            }
        }
        
    }
}

//async fn 

async fn record_attendance(uuid: &str) -> Result<(), Box<dyn std::error::Error>> {
    //setup involving the .env file
    dotenvy::dotenv()?;
    //connect to the database
    let pool = MySqlPool::connect(&env::var("DATABASE_URL")?).await?;
    //query the record_attendance stored procedure (non-manual attendance)
    let result = sqlx::query!("CALL record_attendance(?)", uuid)
        .execute(&pool) //execute the query
        .await?; //wait for the query to finish (some asynchronous programming shenanigans)
    //if the query was successful
    if result.rows_affected() > 0 {
        println!("Attendance recorded"); //print that the attendance was recorded
    }
    pool.close().await; //close connection to database
    Ok(()) //return from the function with no errors
}

async fn manual_attendance(emp_id: &u64) -> Result<(), Box<dyn std::error::Error>> {
    //setup involving the .env file
    dotenvy::dotenv()?;
    //connect to the database
    let pool = MySqlPool::connect(&env::var("DATABASE_URL")?).await?;
    //query the record_attendance_by_empid stored procedure (manual attendance)
    let result = sqlx::query!("CALL record_attendance_by_empid(?)", emp_id)
        .execute(&pool)//execute the query
        .await?; //wait for the query to finish (some asynchronous programming shenanigans)
    //if the query was successful
    if result.rows_affected() > 0 {
        println!("Attendance manually recorded"); //print that the attendance was recorded
    }
    pool.close().await; //close connection to database
    Ok(()) //return from the function with no errors
}

async fn employee_name_from_uuid(uuid: &str) -> String {
    dotenvy::dotenv().unwrap();
    let pool = MySqlPool::connect(&env::var("DATABASE_URL").unwrap()).await.unwrap();
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
    let pool = MySqlPool::connect(&env::var("DATABASE_URL").unwrap()).await.unwrap();
    let result = sqlx::query!(r#"SELECT production_staff.emp_id AS "emp_id", 
    employee.fname AS "fname", employee.mname AS "mname", employee.lname AS "lname" FROM production_staff JOIN employee USING(emp_id) WHERE emp_id = ?"#, emp_id)
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
    if let Some(matched_print) = &matched_print { //get the matched print
        //print the matched print's username
        println!("Matched print: {:#}", matched_print.username().expect("Fingerprint username could not be retrieved"));

        //set the matched print's username to the print
        if print.username().is_some() {
            println!("Print: {:#}", &print.username().expect("Fingerprint username could not be retrieved"));
        } else {
            println!("Print does not have a username");
        }

        //set the scanned fingerprint's username to the matched print's username 
        //(because the scanned fingerprint was matched with the previously enrolled fingerprint, 
        //and currently, the scanned fingerprint has no username)
        print.set_username(&matched_print.username().expect("Username could not be retrieved"));

        //print the scanned fingerprint's username for debugging purposes
        //(by this point, the scanned fingerprint should already have the same username as the matched fingerprint)
        println!("Print username: {:#}", &print.username().expect("Fingerprint username could not be retrieved"));
    } else { //if matched_print is None (null value)
        //print that no fingerprint was matched with the scanned fingerprint
        println!("Not matched");
    }
}