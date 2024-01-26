use libewmh::{prelude::*, window::WinOpt};

// Resize and move the active window
fn main() {
    WinOpt::new(None).shape(WinShape::Halfw).pos(WinPosition::Right).place().unwrap();
}
