use libfprint_rs::{FpContext, FpPrint, FpDevice};

pub fn enroll_cb(device: &FpDevice, enroll_stage: i32, print: Option<FpPrint>, error: Option<libfprint_rs::GError>, data: &Option<i32>,) -> () {
    println!("Enroll stage: {}", enroll_stage);
}

fn main() {
    // let context = FpContext::new();
    // let devices = context.devices();

    //Enrolling a new fingerprint
    let context = FpContext::new();
    let devices = context.devices();

    let dev = devices.get(0).expect("No devices found");
    dev.open_sync(None).expect("Should open synchronously with no problems"); //?

    //adding a new fingerprint via enroll_sync
    let template = FpPrint::new(dev);
    let new_print = dev
                                            .enroll_sync(template, None,
                                                Some(enroll_cb), 
                                                Some(10));
    template.set_username("Bruce Banner");
    
    println!("{}", template.username().unwrap());


    //

    //Hello world for sanity check
    println!("Hello, world!");
    println!("Hello World");

    // //Verifying a fingerprint
    // let context = FpContext::new();
    // let devices = context.devices();

    // let dev = devices.get(0).expect("No devices found");
    // dev.open_sync(None).expect("Should work here");

    // // let enrolled_print = load_print_from_file();

    // // let match_res = dev.verify_sync(enrolled_print, None, None, None::<()>, None).expect("Some error encountered while verifying the fingerprint."); //?
}