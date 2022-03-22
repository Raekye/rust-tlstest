use std::cell::Cell;

thread_local! {
	static LIB_TL: Cell<*mut u32> = const { Cell::new(std::ptr::null_mut()) };
}

#[inline(never)]
#[no_mangle]
pub fn get_lib_tl() -> *mut u32 {
	LIB_TL.with(|tl| tl.get())
}

// Prevent the compiler from optimizing `LIB_TL` to a constant.
pub fn set_lib_tl(ptr: *mut u32) {
	LIB_TL.with(|tl| tl.set(ptr));
}
