use super::TestObj;

#[no_mangle]
pub extern "C" fn gtk_gl_shaders_test_obj_new() -> *mut glib::gobject_ffi::GObject {
    unsafe {
        <TestObj as glib::translate::IntoGlibPtr<
            *mut glib::subclass::basic::InstanceStruct<super::imp::TestObj>,
        >>::into_glib_ptr(TestObj::new()) as *mut glib::gobject_ffi::GObject
    }
}
