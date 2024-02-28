use std::{fs::OpenOptions, io::{BufReader, Read}};

use libfprint_rs::{FpContext, FpPrint};

fn main() {
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
    }
}

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