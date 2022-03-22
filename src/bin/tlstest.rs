use std::cell::Cell;

use tlstest::*;

thread_local! {
	static BIN_TL: Cell<*mut u32> = const { Cell::new(std::ptr::null_mut()) };
}

#[inline(never)]
#[no_mangle]
pub fn get_bin_tl() -> *mut u32 {
	BIN_TL.with(|tl| tl.get())
}

fn main() {
	// Prevent the compiler from optimizing `BIN_TL` to a constant.
	if std::env::var("test").is_ok() {
		BIN_TL.with(|tl| tl.set(1 as *mut u32));
		set_lib_tl(1 as *mut u32);
	}
	unsafe {
		// Just use the values somehow so they don't get optimized out.
		libc::free(get_bin_tl().cast());
		libc::free(get_lib_tl().cast());
	}
}
