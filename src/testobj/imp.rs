use glib::subclass::prelude::*;

#[derive(Default)]
pub struct TestObj;

#[glib::object_subclass]
impl ObjectSubclass for TestObj {
    const NAME: &'static str = "TestObj";
    type Type = super::TestObj;
}

impl ObjectImpl for TestObj {}
