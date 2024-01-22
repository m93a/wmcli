# wmcli
[![license-badge](https://img.shields.io/crates/l/fungus.svg)](https://opensource.org/licenses/MIT)
[![crates.io](https://img.shields.io/crates/v/wmcli.svg)](https://crates.io/crates/wmcli)
[![Minimum rustc](https://img.shields.io/badge/rustc-1.30+-lightgray.svg)](https://github.com/phR0ze/gory#rustc-requirements)

***Rust X11 automation***

`wmcli` implements the [Extended Window Manager Hints (EWMH) specification](https://specifications.freedesktop.org/wm-spec/latest/)
as a way to work along side EWMH compatible window managers as a companion. `wmcli` provides the 
ability to precisely define how windows should be shaped and placed and can fill in gaps for window 
managers lacking some shaping or placement features. Mapping wmcli commands to user defined hot key 
sequences will allow for easy window manipulation beyond what your favorite EWMH window manager 
provides.

### Disclaimer
***wmcli*** comes with absolutely no guarantees or support of any kind. It is to be used at
your own risk. Any damages, issues, losses or problems caused by the use of ***wmcli*** are
strictly the responsiblity of the user and not the developer/creator of ***wmcli***.

### Quick links
* [Usage](#usage)
  * [Shape window](#shape-window)
  * [Move window](#move-window)
  * [Place window](#place-window)
* [Contribute](#contribute)
  * [Git-Hook](#git-hook)
* [License](#license)
  * [Contribution](#contribution)
* [Backlog](#backlog)
* [Changelog](#changelog)

## Usage <a name="usage"/></a>
***rustc >= 1.30*** is required due to the 
[tracing\_subscriber](https://docs.rs/tracing-subscriber/0.2.15/tracing_subscriber) requirements

`$ wmcli -h` for cli help
![help image](docs/images/help.png)

### Shape window <a name="shape-window"/></a>
Shape the active window using the pre-defined `small` shape which is a quarter of the screen.
```bash
$ wmcli shape small
```

### Move window <a name="move-window"/></a>
Move the active window to the bottom left corner of the screen.
```bash
$ wmcli move bottom-left
```

### Place window <a name="place-window"/></a>
Combine the shape and move into a single command by placing the window. First the window is shaped 
using the pre-defined `small` shape then it is moved to the bottom left of the screen in a single 
operation.
```bash
$ wmcli place small bottom-left
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
 * MIT license [LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT
 * Apache License, Version 2.0 [LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0

### Contribution <a name="contribution"/></a>
Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in
this project by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without
any additional terms or conditions.

---

## Backlog <a name="backlog"/></a>

## Changelog <a name="changelog"/></a>
* 12/18/2021
  * Add Arch Linux packaging
  * Added public documentation
  * Fix to precisely place windows with Xfwm4
  * Completed move, shape and place implementation
