//! `libewmh` implements the [Extended Window Manager Hints (EWMH) specification](https://specifications.freedesktop.org/wm-spec/latest/)
//! as a way to integrate with EWMH compatible window managers. The EWHM spec builds on the lower
//! level Inter Client Communication Conventions Manual (ICCCM) to define interactions between
//! window managers, compositing managers and applications.
//!
//! [Root Window Properties](https://specifications.freedesktop.org/wm-spec/latest/ar01s03.html)  
//! The EWMH spec defines a number of properties that EWHM compliant window managers will maintain
//! and return to clients requesting information. `libewmh` taps into the message queue to retrieve
//! details about a given window and to than manipulate the given window as desired.
//!
//! `wmcli` uses `libewmh` with pre-defined shapes and positions to manipulate how a window should
//! be shaped and positioned on the screen in an ergonomic way; however `libewmh` could be used
//! for a variety of reasons.
mod atoms;
mod error;
mod model;
pub mod window;
mod wm;
pub use error::*;
pub use model::*;
pub use wm::WindowManager;

/// All essential symbols in a simple consumable form
///
/// ### Examples
/// ```
/// use libewmh::prelude::*;
/// ```
pub mod prelude {
    pub use crate::*;
}

/// Window option provides an ergonomic way to manipulate a window

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
