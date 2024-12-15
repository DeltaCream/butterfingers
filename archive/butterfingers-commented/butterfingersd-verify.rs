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

#[tokio::main]
async fn main() {
    // let context = FpContext::new();
    // let devices = context.devices();

    // let dev = devices.first().unwrap();
    // println!("Device name is {}", dev.name());
    // dev.open_sync(None).expect("Device could not be opened for verification");

    // let fpprint_file = OpenOptions::new().read(true).open("print/fprint").expect("Could not read the fingerprint file"); //changed from File::open to OpenOptions::open
    // let mut reader = BufReader::new(fpprint_file);
    // let mut buffer = Vec::new();

    // //read file into buffer vector
    // reader.read_to_end(&mut buffer).expect("Could not retrieve contents of file");

    // //deserialize the fingerprint stored in the file
    // let deserialized_print = FpPrint::deserialize(&buffer);

    // //retrieve the enrolled print from deserialized_print
    // let enrolled_print = deserialized_print.expect("Could not unwrap the deserialized print");

    // let match_res = dev.verify_sync(&enrolled_print, None, None, None::<()>, None).expect("Some error was encountered during verifying the fingerprint");

    // if match_res { //if fingerprint was found to be verified (the fingerprint is already stored)
    //     println!("Congratulations, the fingerprint is verified");
    // } else {
    //     println!("Huh... the fingerprint is not verified");
    // }

    let context = FpContext::new();
    let devices = context.devices();
    let fp_scanner = devices.first().expect("Devices could not be retrieved");
    
    fp_scanner.open_sync(None).expect("Device could not be opened");

    // Get a list of all entries in the folder
    let entries = fs::read_dir(dirs::home_dir().expect("Home directory could not be found").join("print/")).expect("Could not read the directory");

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

    // Print the list of files
    println!("File names: {:?}", file_names);

    // Iterate over the file names
    let fingerprints = file_names.iter().map(|filename| {
        let fpprint_file = OpenOptions::new().read(true).open(dirs::home_dir().expect("Home directory could not be found").join(format!("print/{}",filename))).expect("Could not read the fingerprint file"); //changed from File::open to OpenOptions::open
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
        println!("Username for Fingerprint #{}: {:?}", i, fingerprint.username().expect("Fingerprint username could not be retrieved"));
    }

    println!("Fingerprints retrieved");

    let mut number_of_tries = 0;

    loop {
        println!("Before new_print declaration");
        let mut new_print = FpPrint::new(fp_scanner); // The variable that will hold the new print
        println!("Please scan your fingerprint");
        let print_identified = fp_scanner.identify_sync(&fingerprints, None, Some(match_cb), None, Some(&mut new_print)).expect("Fingerprint could not be identified due to an error");
        if print_identified.is_some() {
            let fprint = print_identified.expect("Print could not be unwrapped");
            let uuid = fprint.username();
            match uuid {
                Some(uuid) => {
                    println!("UUID of the fingerprint: {}", uuid);
                    let result = record_attendance(&uuid).await;
                    if result.is_ok() {
                        println!("Attendance recorded for {}", employee_name_from_uuid(&uuid).await); //currently changing this to show the person whose fingerprint was scanned
                        number_of_tries = 0;
                    } else {
                        println!("Attendance could not be recorded");
                        number_of_tries += 1;
                    }
                },
                None => println!("UUID could not be retrieved"),
            }
            //println!("UUID of the fingerprint: {}", uuid);
        } else {
            println!("No matching fingerprint could be found");
            number_of_tries += 1;
        }
        if number_of_tries >= 3 {
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
                            println!("Attendance manually recorded for {}", employee_name_from_empid(&emp_id).await); //anything that comes out of an async function is a Future type which needs to be await-ed
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
        }
    }
    //fp_scanner.close_sync(None).expect("Device could not be closed");
}

async fn record_attendance(uuid: &str) -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv()?;
    let pool = MySqlPool::connect(&env::var("DATABASE_URL")?).await?; 
    let result = sqlx::query!("CALL record_attendance(?)", uuid)
        .execute(&pool)
        .await?;
    if result.rows_affected() > 0 {
        println!("Attendance recorded");
    }
    pool.close().await;
    Ok(())
}

async fn manual_attendance(emp_id: &u64) -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv()?;
    let pool = MySqlPool::connect(&env::var("DATABASE_URL")?).await?; 
    let result = sqlx::query!("CALL record_attendance_by_empid(?)", emp_id)
        .execute(&pool)
        .await?;
    if result.rows_affected() > 0 {
        println!("Attendance manually recorded");
    }
    pool.close().await;
    Ok(())
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

pub fn match_cb(
    _device: &FpDevice,
    matched_print: Option<FpPrint>,
    print: FpPrint,
    _error: Option<libfprint_rs::GError>, //Option<glib::Error>,
    _data: &Option<()>,
) {
    if let Some(matched_print) = &matched_print {
        println!("Matched print: {:#}", matched_print.username().expect("Fingerprint username could not be retrieved"));
        if print.username().is_some() {
            println!("Print: {:#}", &print.username().expect("Fingerprint username could not be retrieved"));
        } else {
            println!("Print does not have a username");
        }
        print.set_username(&matched_print.username().expect("Username could not be retrieved"));
        println!("Print username: {:#}", &print.username().expect("Fingerprint username could not be retrieved"));
    } else {
        println!("Not matched");
    }
}

// async fn get_employee_name(uuid: &str) -> Result<(), Box<dyn std::error::Error>> {
//     dotenvy::dotenv()?;
//     let pool = MySqlPool::connect(&env::var("DATABASE_URL")?).await?; 
//     let result = sqlx::query!("SELECT employee.fname, employee.lname FROM employee WHERE uuid = ?", uuid)
//         .fetch_one(&pool)
//         .await?;
//     pool.close().await;
//     Ok(())
// }

/*
expected fn pointer `for<'a, 'b> fn(&'a libfprint_rs::FpDevice, std::option::Option<_>, libfprint_rs::FpPrint, std::option::Option<libfprint_rs::GError>, &'b std::option::Option<_>)`
      found fn item `for<'a, 'b> fn(&'a libfprint_rs::FpDevice, std::option::Option<_>, libfprint_rs::FpPrint, std::option::Option<glib::Error>, &'b std::option::Option<()>) {match_cb}
*/

// fn get_enrolled_prints(file_names: Vec<String>) -> Vec<FpPrint> {
//     // Iterate over the file names
//     file_names.iter().map(|filename| {
//         let fpprint_file = OpenOptions::new().read(true).open("print/fprint").expect("Could not read the fingerprint file"); //changed from File::open to OpenOptions::open
//         let mut reader = BufReader::new(fpprint_file);
//         let mut buffer = Vec::new();

//         //read file into buffer vector
//         reader.read_to_end(&mut buffer).expect("Could not retrieve contents of file");

//         //deserialize the fingerprint stored in the file
//         let deserialized_print = FpPrint::deserialize(&buffer);

//         //retrieve the enrolled print from deserialized_print
//         let enrolled_print = deserialized_print.expect("Could not unwrap the deserialized print");
//     }).collect::<Vec<FpPrint>>()
// }

//infinite loop
/*
while true{
	scan for finger
	select the uuid of finger
	check if uuid exists in db
	if exists, call record_attendance(uuid)
	else error
}

*/
/*
loop {
    scan finger
    select uuid
    check if uuid exists in db
    SELECT emp_id from employee where uuid = ?, uuid.to_string()
    if exists, call record_attendance(uuid)
    else println!("Fingerprint does not exist in the database")
}

*/