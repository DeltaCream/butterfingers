use libfprint_rs::{FpContext, FpPrint, FpDevice, FpFinger};

fn main() {
    let context = FpContext::new();
    let devices = context.devices();

    let dev = devices.get(0).unwrap();
    dev.open_sync(None).expect("Device could not be opened for verification");

    let enrolled_print = load_print_from_file();

    let match_res = dev.verify_sync(enrolled_print, None, None, None::<()>, None).expect("Some error was encountered during verifying the fingerprint");
}
