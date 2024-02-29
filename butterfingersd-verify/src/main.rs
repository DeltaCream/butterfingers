use std::{
    fs::{
        OpenOptions,
        self,
    }, 
    io::{
        BufReader,
        Read
    },
    env,
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
    
    loop {
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

        println!("Fingerprints retrieved");

        println!("Before new_print declaration");
        let mut new_print = FpPrint::new(fp_scanner); // The variable that will hold the new print
        println!("Please scan your fingerprint");
        let print_identified = fp_scanner.identify_sync(&fingerprints, None, Some(match_cb), None, Some(&mut new_print)).expect("Fingerprint could not be identified due to an error");
        if print_identified.is_some() {
            let fprint = print_identified.expect("Print could not be unwrapped");
            let uuid = fprint.username().expect("UUID (Username) could not be retrieved");
            println!("UUID of the fingerprint: {}", uuid);
            let result = record_attendance(&uuid).await;
            if result.is_ok() {
                println!("Attendance recorded for {}", uuid);
            } else {
                println!("Attendance could not be recorded");
            }
        } else {
            println!("No matching fingerprint could be found")
        }

        fp_scanner.close_sync(None).expect("Device could not be closed");
    }

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

pub fn match_cb(
    _device: &FpDevice,
    matched_print: Option<FpPrint>,
    _print: FpPrint,
    _error: Option<libfprint_rs::GError>, //Option<glib::Error>,
    _data: &Option<()>,
) -> () {
    if matched_print.is_some() {
        println!("Matched");
    } else {
        println!("Not matched");
    }
}

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