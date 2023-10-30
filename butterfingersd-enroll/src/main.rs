use std::{sync::{Arc, Mutex}, io::{stdin, self}};

use libfprint_rs::{FpContext, FpPrint, FpDevice, FpFinger};


fn main() {

    // println!("For whom will you scan the fingerprint?");

    // let mut user = String::new();
    // io::stdin().read_line(&mut user).expect("Failed to get username");

    let context = FpContext::new();

    let devices = context.devices();

    let dev = devices.get(0).expect("Devices could not be retrieved");

    println!("{:#?}", dev.scan_type()); //print the scan type of the device
    println!("{:#?}", dev.features());  //print the features of the device


    dev.open_sync(None).expect("Device could not be opened");

    let template = FpPrint::new(dev);    //&dev); //&dev because dev might be used later
    template.set_finger(FpFinger::RightIndex);
    template.set_username("test");  //&user);

    println!("Username of the fingerprint: {}", template.username().expect("Fingerprint username could not be retrieved"));

    let counter = Arc::new(Mutex::new(0));

    let _new_print = dev
        .enroll_sync(template, None, Some(progress_cb), Some(counter.clone()))
        .unwrap();

    println!("new_print contents: {:#?}",_new_print);   //print the FpPrint struct
    println!("new_print username: {:#?}",_new_print.username().unwrap());   //print the username of the FpPrint

    println!("Total enroll stages: {}", counter.lock().unwrap());

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

pub fn progress_cb(
    _device: &FpDevice,
    enroll_stage: i32,
    _print: Option<FpPrint>,
    _error: Option<glib::Error>,
    data: &Option<Arc<Mutex<i32>>>,
) -> () {
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
) -> () {
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
