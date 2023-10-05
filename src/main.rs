use libfprint_rs::{FpContext, FpPrint};

fn main() {
    // let context = FpContext::new();
    // let devices = context.devices();

    //Enrolling a new fingerprint
    let context = FpContext::new();
    let devices = context.devices();

    let dev = devices.get(0).expect("No devices found");
    dev.open_sync(None).expect("Should work with no problems"); //?

    let template = FpPrint::new(dev);
    template.set_username("Bruce Banner");
    
    println!("{}", template.username().unwrap());


    //

    //Hello world for sanity check
    println!("Hello, world!");
    println!("Hello World");

    //Verifying a fingerprint
    let context = FpContext::new();
    let devices = context.devices();

    let dev = devices.get(0).expect("No devices found");
    dev.open_sync(None).expect("Should work here");

    // let enrolled_print = load_print_from_file();

    // let match_res = dev.verify_sync(enrolled_print, None, None, None::<()>, None).expect("Some error encountered while verifying the fingerprint."); //?
}