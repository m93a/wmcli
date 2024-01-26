use libewmh::{prelude::*, window::WinOpt};

// Move the active window
fn main() {
    WinOpt::new(None).shape(WinShape::Max).place().unwrap();
}
