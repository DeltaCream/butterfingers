use std::{fs::{File, self}, io::{BufRead, BufReader, Read}};

use libfprint_rs::{FpContext, FpPrint, FpDevice, FpFinger};

fn main() {
    let context = FpContext::new();
    let devices = context.devices();

    let dev = devices.get(0).unwrap();
    dev.open_sync(None).expect("Device could not be opened for verification");

    let fpprint_file = File::open("/print/fprint.txt").expect("Could not read the fingerprint file");
    let mut reader = BufReader::new(fpprint_file);
    let mut buffer = Vec::new();

    //read file into buffer vector
    reader.read_to_end(&mut buffer).expect("Could not retrieve contents of file");

    //deserialize the fingerprint stored in the file
    let deserialized_print = FpPrint::deserialize(&buffer);

    //retrieve the enrolled print from deserialized_print
    let enrolled_print = deserialized_print.expect("Could not unwrap the deserialized print");

    let match_res = dev.verify_sync(&enrolled_print, None, None, None::<()>, None).expect("Some error was encountered during verifying the fingerprint");

    if match_res { //if fingerprint was found to be verified (the fingerprint is already stored)
        println!("Congratulations, the fingerprint is verified");
    } else {
        println!("Huh... the fingerprint is not verified");
    }
}
