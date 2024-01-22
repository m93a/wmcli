# libewmh

[![license-badge](https://img.shields.io/crates/l/fungus.svg)](https://opensource.org/licenses/MIT)
[![crates.io](https://img.shields.io/crates/v/libewmh.svg)](https://crates.io/crates/libewmh)
[![Minimum rustc](https://img.shields.io/badge/rustc-1.30+-lightgray.svg)](https://github.com/phR0ze/gory#rustc-requirements)

**_Rust X11 automation_**

`libewmh` implements the [Extended Window Manager Hints (EWMH) specification](https://specifications.freedesktop.org/wm-spec/latest/)
as a way to work along side EWMH compatible window managers as a companion. `libewmh` provides the
ability to precisely define how windows should be shaped and placed and can fill in gaps for window
managers lacking some shaping or placement features. `libewmh` exposes X11 details in a simple
consumable way opening the door to window manipulation beyond what your favorite EWMH window manager
provides.

### Quick links

- [Usage](#usage)
  - [Shape window](#shape-window)
  - [Move window](#move-window)
  - [Place window](#place-window)
  - [Window Manager info](#window-manager-info)
- [Contribute](#contribute)
  - [Git-Hook](#git-hook)
- [License](#license)
  - [Contribution](#contribution)
- [Backlog](#backlog)
- [Changelog](#changelog)

## Usage <a name="usage"/></a>

This minimum rustc requirement is driven by the
[tracing_subscriber](https://docs.rs/tracing-subscriber/0.2.15/tracing_subscriber) requirements

### Shape window <a name="shape-window"/></a>

Shape the active window using the pre-defined `WinShape::Small` shape which is a quarter of the
screen.

```rust
use libewmh::prelude::*;

fn main() {
    WinOpt::new(None).shape(WinShape::Max).place().unwrap();
}
```

### Move window <a name="move-window"/></a>

Move the active window to the bottom left corner of the screen using the pre-defined
`WinPosition::BottomLeft` position.

```rust
use libewmh::prelude::*;

fn main() {
    WinOpt::new(None).pos(WinPosition::BottomLeft).place().unwrap();
}
```

### Place window <a name="place-window"/></a>

Combine the shape and move into a single command by placing the window. First the window is shaped
using the pre-defined `WinShap::Small` shape then it is moved to the bottom left using the
pre-defined `WinPosition:BottomLeft` position in a single operation.

```rust
use libewmh::prelude::*;

fn main() {
    WinOpt::new(None).shape(WinShape::Small).pos(WinPosition::BottomLeft).place().unwrap();
}
```

### Window Manager info <a name="window-manager-info"/></a>

```rust
use libewmh::prelude::*;

fn main() {
    let wmcli = wmcli::connect().unwrap();
    let (_, wm_name) = wmcli.winmgr().unwrap();
    let win = wmcli.active_win().unwrap();
    println!("X11 Information");
    println!("-----------------------------------------------------------------------");
    println!("Window Manager:    {}", wm_name);
    println!("Composite Manager: {}", wmcli.composite_manager().unwrap());
    println!("Root Window:       {}", wmcli.root());
    println!("Work area:         {}x{}", wmcli.work_width(), wmcli.work_height());
    println!("Screen Size:       {}x{}", wmcli.width(), wmcli.height());
    println!("Desktops:          {}", wmcli.desktops().unwrap());
    println!();
    println!("Active Window");
    println!("{:-<120}", "");

    println!("{:<8} {:<3} {:<6} {:<5} {:<5} {:<4} {:<4} {:<8} {:<7} {:<18} {:<18} {}", "ID", "DSK", "PID", "X", "Y", "W", "H", "BORDERS", "TYPE", "STATE", "CLASS", "NAME");

    let pid = wmcli.win_pid(win).unwrap_or(-1);
    let desktop = wmcli.win_desktop(win).unwrap_or(-1);
    let typ = wmcli.win_type(win).unwrap_or(WinType::Invalid);
    let states = wmcli.win_state(win).unwrap_or(vec![WinState::Invalid]);
    let (x, y, w, h) = wmcli.win_geometry(win).unwrap_or((0,0,0,0));
    let (l, r, t, b) = wmcli.win_borders(win).unwrap_or((0, 0, 0, 0));
    let class = wmcli.win_class(win).unwrap_or("".to_owned());
    let name = wmcli.win_name(win).unwrap_or("".to_owned());
    println!("{:<8} {:<3} {:<6} {:<5} {:<5} {:<4} {:<4} {:<8} {:<7} {:<18} {:<18} {}",
        format!("{:0>8}", win), format!("{:>2}", desktop), pid,
        format!("{:<4}", x), format!("{:<4}", y), format!("{:<4}", w), format!("{:<4}", h),
        format!("{},{},{},{}", l, r, t, b),
        typ.to_string(), format!("{:?}", states), class, name);
}
```

## Contribute <a name="Contribute"/></a>

Pull requests are always welcome. However understand that they will be evaluated purely on whether
or not the change fits with my goals/ideals for the project.

### Git-Hook <a name="git-hook"/></a>

Enable the git hooks to have automatic version increments

```bash
cd ~/Projects/wmcli
git config core.hooksPath .githooks
```

## License <a name="license"/></a>

This project is licensed under either of:

- MIT license [LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT
- Apache License, Version 2.0 [LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0

### Contribution <a name="contribution"/></a>

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in
this project by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without
any additional terms or conditions.

---

## Backlog <a name="backlog"/></a>

## Changelog <a name="changelog"/></a>
