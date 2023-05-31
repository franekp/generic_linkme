#![allow(clippy::needless_lifetimes, clippy::trivially_copy_pass_by_ref)]

use generic_linkme::{distributed_fn_slice, link};

#[distributed_fn_slice]
pub static SLICE1: [fn() -> u32] = [..];

#[distributed_fn_slice(SLICE1)]
fn foo() -> u32 { 4 }

#[distributed_fn_slice]
pub static SLICE2: [for<'a, 'b> fn(&'a &'b ())] = [..];

#[distributed_fn_slice(SLICE2)]
fn bar<'a, 'b>(_x: &'a &'b ()) {
    println!("adsf");
}

#[distributed_fn_slice]
pub static SLICE3: [unsafe extern "sysv64" fn() -> i32] = [..];

#[distributed_fn_slice(SLICE3)]
unsafe extern "sysv64" fn baz() -> i32 {
    42
}

#[test]
fn test_slices() {
    assert!(!SLICE1.is_empty());
    //assert!(!SLICE2.is_empty());
    assert!(!SLICE3.is_empty());
    unsafe {
        std::ptr::read_volatile(foo as *const u8);
        std::ptr::read_volatile(bar as *const u8);
        std::ptr::read_volatile(baz as *const u8);
    }
    link(foo);
    link(bar);
    link(|| unsafe { baz() });
}
