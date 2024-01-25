//! `wmcli` implements the [Extended Window Manager Hints (EWMH) specification](https://specifications.freedesktop.org/wm-spec/latest/)
//! as a way to work along side EWMH compatible window managers as a companion. `wmcli` provides the
//! ability to precisely define how windows should be shaped and placed and can fill in gaps for
//! window managers lacking some shaping or placement features. Mapping `wmcli` commands to user
//! defined hot key sequences will allow for easy window manipulation beyond what your favorite EWMH
//! window manager provides.
//!
//! ## Command line examples
//!
//! ### Shape a window
//! Shape the active window using the pre-defined `small` shape which is a quarter of the screen.
//! ```bash
//! wmcli shape small
//! ```
//!
//! ### Move a window
//! Move the active window to the bottom left corner of the screen.
//! ```bash
//! wmcli move bottom-left
//! ```
//!
//! ### Place a window
//! Shape the active window using the pre-defined `small` shape which is a quarter of the screen
//! and then position it in the bottom left corner of the screen.
//! ```bash
//! wmcli place small bottom-left
//! ```
use std::env;

use clap::{crate_description, crate_version, Command};
use libewmh::WinOpt;

fn cli() -> Command {
    Command::new("wmcli")
        .about(crate_description!())
        .subcommand_required(true)
        .arg_required_else_help(true)
        .version(crate_version!())
        .subcommand(
            Command::new("window")
                .visible_alias("w")
                .about("Control individual windows.")
                .subcommand_required(true)
                .arg_required_else_help(true)
                .subcommand(Command::new("list").visible_alias("l").about("List out all windows"))
                .subcommand(Command::new("move").visible_alias("m").about("Move a window"))
                .subcommand(Command::new("shape").visible_alias("s").about("Resize a window"))
                .subcommand(Command::new("close").visible_alias("c").about("Close a window")),
        )
        .subcommand(
            Command::new("desktop")
                .visible_alias("d")
                .about("Manage desktops (also known as workspaces)")
                .subcommand_required(true)
                .arg_required_else_help(true)
                .subcommand(Command::new("list").visible_alias("l").about("List all desktops"))
                .subcommand(Command::new("switch").visible_alias("s").about("Switch to a desktop"))
                .subcommand(Command::new("close").visible_alias("c").about("Close a desktop")),
        )
}

fn main() {
    let _ = match cli().get_matches().subcommand() {
        Some(("window", sub)) => match sub.subcommand() {
            Some(("list", _)) => libewmh::list(false),
            Some(("move", _)) => WinOpt::new(None).pos(libewmh::WinPosition::Bottom).place(),
            _ => unreachable!(),
        },
        _ => unreachable!(),
    };
}

// fn foo() {
//     // Determine the target window
//     let win = { matches.value_of("window").and_then(|x| x.parse::<u32>().ok()) };

//     // info
//     else if let Some(_) = matches.subcommand_matches("info") {
//         libewmh::info(win).pass()?;

//     // list
//     } else if let Some(matches) = matches.subcommand_matches("list") {
//         libewmh::list(matches.is_present("all")).pass()?;

//     // move
//     } else if let Some(ref matches) = matches.subcommand_matches("move") {
//         let pos = WinPosition::try_from(matches.value_of("POSITION").unwrap()).pass()?;
//         WinOpt::new(win).pos(pos).place().pass()?;

//     // place
//     } else if let Some(ref matches) = matches.subcommand_matches("place") {
//         let shape = WinShape::try_from(matches.value_of("SHAPE").unwrap()).pass()?;
//         let pos = WinPosition::try_from(matches.value_of("POSITION").unwrap()).pass()?;
//         WinOpt::new(win).shape(shape).pos(pos).place().pass()?;

//     // static
//     } else if let Some(ref matches) = matches.subcommand_matches("static") {
//         let w = matches.value_of("WIDTH").unwrap().parse::<u32>().pass()?;
//         let h = matches.value_of("HEIGHT").unwrap().parse::<u32>().pass()?;
//         let mut win = WinOpt::new(win).size(w, h);
//         if matches.value_of("X").is_some() && matches.value_of("Y").is_some() {
//             let x = matches.value_of("X").unwrap().parse::<u32>().pass()?;
//             let y = matches.value_of("Y").unwrap().parse::<u32>().pass()?;
//             win = win.location(x, y);
//         }
//         win.place().pass()?;

//     // shape
//     } else if let Some(ref matches) = matches.subcommand_matches("shape") {
//         let shape = WinShape::try_from(matches.value_of("SHAPE").unwrap()).pass()?;
//         WinOpt::new(win).shape(shape).place().pass()?;
//     }
// }
