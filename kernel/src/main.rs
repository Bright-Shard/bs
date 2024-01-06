#![no_std]
#![no_main]

use common::*;

#[no_mangle]
extern "C" fn main() {
    println!("HALLO FROM KERNEL");
}
