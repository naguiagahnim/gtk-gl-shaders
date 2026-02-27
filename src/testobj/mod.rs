pub mod ffi;
mod imp;

glib::wrapper! {
    pub struct TestObj(ObjectSubclass<imp::TestObj>);
}

impl Default for TestObj {
    fn default() -> Self {
        Self::new()
    }
}

impl TestObj {
    pub fn new() -> Self {
        glib::Object::new()
    }
}
