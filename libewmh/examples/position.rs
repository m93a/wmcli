use libewmh::prelude::*;

// Move the active window
fn main() {
    WinOpt::new(None).pos(WinPosition::Left).place().unwrap();
}
