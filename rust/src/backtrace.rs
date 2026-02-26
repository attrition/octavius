use std::ffi::{c_char, c_int};

unsafe extern "C" {
    fn log_info(msg: *const c_char, param_str: *const c_char, param_int: c_int);
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn backtrace_print() {
    // In Rust we can use the backtrace crate if added to Cargo.toml,
    // or just leave it empty for now matching the non-GNUC/Windows paths in C.
    // For a real rewrite, we would use std::backtrace or backtrace crate.
}
