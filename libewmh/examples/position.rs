use libewmh::{prelude::*, window::WinOpt};

// Move the active window
fn main() {
    WinOpt::new(None).pos(WinPosition::Left).place().unwrap();
}
