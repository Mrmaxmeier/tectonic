use std::ffi::CStr;
use libc;

mod papersize;
use self::papersize::PaperSize;

#[no_mangle]
pub extern fn paperinfo(ppformat: *const libc::c_char, width: *mut f64, height: *mut f64) -> usize {
  if ppformat.is_null() {
    return 1
  }

  let name = CStr::from_ptr(ppformat).to_string_lossy();
  if let Some(papersize) = PaperSize::from_str(&name) {
    let (pswidth, psheight) = papersize.dimensions();
    *width = pswidth;
    *height = psheight;
    0
  } else {
    1
  }
}
