use crate::core::IBusAttrList;
use crate::glib::gchar;
use std::ffi::CString;

extern "C" {
    pub fn ibus_text_new_from_string(str_: *const gchar) -> *mut IBusText;
    #[doc = " ibus_text_set_attributes:\n @text: An IBusText.\n @attrs: An IBusAttrList"]
    pub fn ibus_text_set_attributes(text: *mut IBusText, attrs: *mut IBusAttrList);
}

pub type IBusText = [u64; 9usize];

pub trait StringExt {
    fn to_ibus_text(&self) -> *mut IBusText;
}

impl StringExt for str {
    fn to_ibus_text(&self) -> *mut IBusText {
        unsafe {
            let text_c_str = CString::new(self).unwrap();
            ibus_text_new_from_string(text_c_str.as_ptr() as *const gchar)
        }
    }
}
