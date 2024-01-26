//! `WindowManager` uses the [Extended Window Manager Hints (EWMH) specification](https://specifications.freedesktop.org/wm-spec/latest/)
//! as a way to integrate with EWMH compatible window managers. The EWHM spec builds on the lower
//! level Inter Client Communication Conventions Manual (ICCCM) to define interactions between
//! window managers, compositing managers and applications.
//!
//! [Root Window Properties](https://specifications.freedesktop.org/wm-spec/latest/ar01s03.html)  
//! The EWMH spec defines a number of properties that EWHM compliant window managers will maintain
//! and return to clients requesting information. `WindowManager` taps into the message queue to retrieve
//! details about a given window and to than manipulate the given window as desired.
//!
//! `wmcli` uses `WindowManager` with pre-defined shapes and positions to manipulate how a window should
//! be shaped and positioned on the screen in an ergonomic way; however `WindowManager` could be used
//! for a variety of reasons.
use crate::{
    atoms::AtomCollection, model::*, window::Window, ErrorWrapper, WindowManagerError, WindowManagerResult,
};
use std::{collections::HashMap, str, sync::Arc};
use tracing::{debug, trace};

use x11rb::{
    connection::Connection,
    protocol::xproto::{
        self, Atom, AtomEnum, ClientMessageEvent, ConnectionExt as _, EventMask, GetPropertyReply,
    },
    rust_connection::RustConnection,
};

// Define the second byte of the move resize flags 32bit value
// Used to indicate that the associated value has been changed and needs to be acted upon
pub type MoveResizeWindowFlags = u32;
pub const MOVE_RESIZE_WINDOW_X: MoveResizeWindowFlags = 1 << 8;
pub const MOVE_RESIZE_WINDOW_Y: MoveResizeWindowFlags = 1 << 9;
pub const MOVE_RESIZE_WINDOW_WIDTH: MoveResizeWindowFlags = 1 << 10;
pub const MOVE_RESIZE_WINDOW_HEIGHT: MoveResizeWindowFlags = 1 << 11;

pub type WindowStateAction = u32;
pub const WINDOW_STATE_ACTION_REMOVE: WindowStateAction = 0;
pub const WINDOW_STATE_ACTION_ADD: WindowStateAction = 1;

/// Window Manager control implements the EWMH protocol using x11rb to provide a simplified access
/// layer to EWHM compatible window managers.
pub struct WindowManager {
    conn: Arc<RustConnection>,     // x11 connection
    pub atoms: AtomCollection,     // atom cache
    supported: HashMap<u32, bool>, // cache for supported functions
    screen: usize,                 // screen number
    root: u32,                     // root window id
    width: u32,                    // screen width
    height: u32,                   // screen height
    work_width: u32,               // screen height
    work_height: u32,              // screen height
}

pub struct GetPropertyResult {
    boxed: WindowManagerResult<GetPropertyReply>,
}

impl TryInto<u32> for GetPropertyResult {
    type Error = ErrorWrapper;
    fn try_into(self) -> WindowManagerResult<u32> {
        self.boxed?.value32().and_then(|mut x| x.next()).ok_or(WindowManagerError::PropertyNotFound.into())
    }
}

impl TryInto<i32> for GetPropertyResult {
    type Error = ErrorWrapper;
    fn try_into(self) -> WindowManagerResult<i32> {
        Ok(TryInto::<u32>::try_into(self)? as i32)
    }
}

impl WindowManager {
    /// Create the window manager control instance and connect to the X11 server
    pub fn connect() -> WindowManagerResult<Self> {
        let (conn, screen) = x11rb::connect(None)?;

        // Get the screen size
        let (width, height, root) = {
            let screen = &conn.setup().roots[screen];
            (screen.width_in_pixels as u32, screen.height_in_pixels as u32, screen.root)
        };

        // Populate the supported functions cache
        let (atoms, supported) = WindowManager::init_caching(&conn, root)?;

        // Create the window manager object
        let mut wmcli = WindowManager {
            conn: Arc::new(conn),
            atoms,
            supported,
            screen,
            root,
            width,
            height,
            work_width: Default::default(),
            work_height: Default::default(),
        };

        // Get the work area
        let (width, height) = wmcli.workarea()?;
        wmcli.work_width = width as u32;
        wmcli.work_height = height as u32;

        debug!("connect: screen: {}, root: {}, w: {}, h: {}", screen, root, width, height);
        Ok(wmcli)
    }

    fn init_caching(
        conn: &RustConnection, root: u32,
    ) -> WindowManagerResult<(AtomCollection, HashMap<u32, bool>)> {
        debug!("initializing caching...");

        // Cache atoms
        let atoms = AtomCollection::new(conn)?.reply()?;

        // Cache supported functions
        let mut supported = HashMap::<u32, bool>::new();
        let reply = conn.get_property(false, root, atoms._NET_SUPPORTED, AtomEnum::ATOM, 0, u32::MAX)?.reply()?;
        for atom in reply.value32().ok_or(WindowManagerError::PropertyNotFound)? {
            trace!("supported: {}", atom);
            supported.insert(atom, true);
        }
        debug!("caching initialized");
        Ok((atoms, supported))
    }

    /// Get the default screen number
    pub fn screen(&self) -> usize {
        self.screen
    }

    /// Get the root window
    pub fn root(&self) -> u32 {
        self.root
    }

    /// Get the screen full width
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Get screen full height
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Get screen work width which is the full width minus any taskbars
    pub fn work_width(&self) -> u32 {
        self.work_width
    }

    /// Get screen work height which is the full width minus any taskbars
    pub fn work_height(&self) -> u32 {
        self.work_height
    }

    fn _get_window_property<A: Into<Atom>, B: Into<Atom>>(
        &self, window_id: u32, property: A, type_: B,
    ) -> Result<GetPropertyReply, ErrorWrapper> {
        Ok(self.conn.get_property(false, window_id, property, type_, 0, u32::MAX)?.reply()?)
    }

    pub fn get_window_property<A: Into<Atom>, B: Into<Atom>>(
        &self, window_id: u32, property: A, type_: B,
    ) -> GetPropertyResult {
        GetPropertyResult {
            boxed: self._get_window_property(window_id, property, type_),
        }
    }

    pub fn get_root_property<A: Into<Atom>, B: Into<Atom>>(&self, property: A, type_: B) -> GetPropertyResult {
        self.get_window_property(self.root, property, type_)
    }

    /// Get the active window id
    pub fn active_win(&self) -> WindowManagerResult<u32> {
        // Defined as: _NET_ACTIVE_WINDOW, WINDOW/32
        self.get_root_property(self.atoms._NET_ACTIVE_WINDOW, AtomEnum::WINDOW).try_into()
    }

    /// Check if a composit manager is running
    pub fn composite_manager(&self) -> WindowManagerResult<bool> {
        // Defined as: _NET_WM_CM_Sn
        // For each screen the compositing manager manages they MUST acquire ownership of a
        // selection named _NET_WM_CM_Sn, where the suffix `n` is the screen number.
        let atom = format!("_NET_WM_CM_S{}", self.screen);
        let atom = self.conn.intern_atom(false, atom.as_bytes())?.reply()?.atom;
        let reply = self.conn.get_selection_owner(atom)?.reply()?;
        let result = reply.owner != x11rb::NONE;
        debug!("composite_manager: {}", result);
        Ok(result)
    }

    /// Get number of desktops
    pub fn desktops(&self) -> WindowManagerResult<u32> {
        // Defined as: _NET_NUMBER_OF_DESKTOPS, CARDINAL/32
        self.get_root_property(self.atoms._NET_NUMBER_OF_DESKTOPS, AtomEnum::CARDINAL).try_into()
    }

    /// Maximize the window both horizontally and vertiacally
    ///
    /// ### Arguments
    /// * `win` - id of the window to manipulate
    ///
    /// ### Examples
    /// ```ignore
    /// use libewmh::prelude::*;
    /// let wmcli = wmcli::connect().unwrap();
    /// wmcli.maximize_win(12345).unwrap();
    /// ```
    pub fn maximize_win(&self, win: xproto::Window) -> WindowManagerResult<()> {
        self.send_event(ClientMessageEvent::new(
            32,
            win,
            self.atoms._NET_WM_STATE,
            [
                WINDOW_STATE_ACTION_ADD,
                self.atoms._NET_WM_STATE_MAXIMIZED_HORZ,
                self.atoms._NET_WM_STATE_MAXIMIZED_VERT,
                0,
                0,
            ],
        ))?;
        debug!("maximize: id: {}", win);
        Ok(())
    }

    /// Move and resize the given window
    ///
    /// ### Arguments
    /// * `win` - id of the window to manipulate
    /// * `gravity` - gravity to use when resizing the window, defaults to NorthWest
    /// * `x` - x coordinate to use for the window during positioning
    /// * `y` - y coordinate to use for the window during positioning
    /// * `w` - width to resize the window to
    /// * `h` - height to resize the window to
    ///
    /// ### Examples
    /// ```ignore
    /// use libewmh::prelude::*;
    /// let wmcli = wmcli::connect().unwrap();
    /// wmcli.move_resize_win(12345, None, Some(0), Some(0), Some(500), Some(500)).unwrap();
    /// ```
    pub fn move_resize_win(
        &self, win: xproto::Window, gravity: Option<u32>, x: Option<u32>, y: Option<u32>, w: Option<u32>,
        h: Option<u32>,
    ) -> WindowManagerResult<()> {
        // Construct the move resize message
        //
        // Gravity is defined as the lower byte of the move resize flags 32bit value
        // https://tronche.com/gui/x/xlib/window/attributes/gravity.html
        // Defines how the window will shift as it grows or shrinks during a shape change operation.
        // The default value is NorthWest which means that the window will grow to the right and down
        // and will shrink up and left. By changing this to center you can get a more distributed growth
        // or shrink perception.
        let mut flags = gravity.unwrap_or(0);

        // Define the second byte of the move resize flags 32bit value
        // Used to indicate that the associated value has been changed and needs to be acted upon
        if x.is_some() {
            flags |= MOVE_RESIZE_WINDOW_X;
        }
        if y.is_some() {
            flags |= MOVE_RESIZE_WINDOW_Y;
        }
        if w.is_some() {
            flags |= MOVE_RESIZE_WINDOW_WIDTH;
        }
        if h.is_some() {
            flags |= MOVE_RESIZE_WINDOW_HEIGHT;
        }

        self.send_event(ClientMessageEvent::new(
            32,
            win,
            self.atoms._NET_MOVERESIZE_WINDOW,
            [flags, x.unwrap_or(0), y.unwrap_or(0), w.unwrap_or(0), h.unwrap_or(0)],
        ))?;

        debug!("move_resize_win: id: {}, g: {:?}, x: {:?}, y: {:?}, w: {:?}, h: {:?}", win, gravity, x, y, w, h);
        Ok(())
    }

    /// Send the event ensuring that a flush is called and that the message was precisely
    /// executed in the case of a resize/move.
    ///
    /// ### Arguments
    /// * `msg` - the client message event to send
    ///
    /// ### Examples
    /// ```ignore
    /// use libewmh::prelude::*;
    /// let wmcli = wmcli::connect().unwrap();
    /// let flags = MOVE_RESIZE_WINDOW_WIDTH | MOVE_RESIZE_WINDOW_HEIGHT;
    /// wmcli.send_event(ClientMessageEvent::new(32, win, wmcli.atoms._NET_MOVERESIZE_WINDOW,
    ///     [flags, 0, 0, 500, 500])).unwrap();
    /// ```
    pub fn send_event(&self, msg: ClientMessageEvent) -> WindowManagerResult<()> {
        let mask = EventMask::SUBSTRUCTURE_REDIRECT | EventMask::SUBSTRUCTURE_NOTIFY;
        self.conn.send_event(false, self.root, mask, &msg)?.check()?;
        self.conn.flush()?;
        debug!("send_event: win: {}", msg.window);

        // I've found that Xfwm4 does not precisely resize a window on the first request. It may be
        // this is a function of decorating the window during a redraw. At any rate because of this
        // unfortunate shortcoming we have to send the event a second time.
        if msg.type_ == self.atoms._NET_MOVERESIZE_WINDOW {
            std::thread::sleep(std::time::Duration::from_millis(50));
            self.conn.send_event(false, self.root, mask, &msg)?.check()?;
            self.conn.flush()?;
            debug!("send_event: win: {}", msg.window);
        }
        Ok(())
    }

    /// Determine if the given function is supported by the window manager
    ///
    /// ### Arguments
    /// * `atom` - atom to lookup to see if its supported
    ///
    /// ### Examples
    /// ```ignore
    /// use libewmh::prelude::*;
    /// let wmcli = wmcli::connect().unwrap();
    /// wmcli.supported(wmcli.atoms._NET_MOVERESIZE_WINDOW);
    /// ```
    #[allow(dead_code)]
    pub fn supported(&self, atom: u32) -> bool {
        self.supported.get(&atom).is_some()
    }

    /// Remove the MaxVert and MaxHorz states
    ///
    /// ### Arguments
    /// * `win` - id of the window to manipulate
    ///
    /// ### Examples
    /// ```ignore
    /// use libewmh::prelude::*;
    /// let wmcli = wmcli::connect().unwrap();
    /// wmcli.unmaximize_win(12345).unwrap();
    /// ```
    pub fn unmaximize_win(&self, win: xproto::Window) -> WindowManagerResult<()> {
        self.send_event(ClientMessageEvent::new(
            32,
            win,
            self.atoms._NET_WM_STATE,
            [
                WINDOW_STATE_ACTION_REMOVE,
                self.atoms._NET_WM_STATE_MAXIMIZED_HORZ,
                self.atoms._NET_WM_STATE_MAXIMIZED_VERT,
                0,
                0,
            ],
        ))?;
        debug!("unmaximize: id: {}", win);
        Ok(())
    }

    /// Get windows optionally all
    ///
    /// ### Arguments
    /// * `all` - default is to get all windows controlled by the window manager, when all is true get the super set of x11 windows
    ///
    /// ### Examples
    /// ```ignore
    /// use libewmh::prelude::*;
    /// let wmcli = wmcli::connect().unwrap();
    /// wmcli.windows(false).unwrap();
    /// ```
    pub fn get_windows(&self, all: bool) -> WindowManagerResult<Vec<Window>> {
        let mut windows = vec![];
        if all {
            // All windows in the X11 system
            let tree = self.conn.query_tree(self.root)?.reply()?;
            for win in tree.children {
                windows.push(Window { id: win });
            }
        } else {
            // Window manager client windows which is a subset of all windows that have been
            // reparented i.e. new ids and don't map to the same ids as their all windows selves.
            let reply = self
                .conn
                .get_property(false, self.root, self.atoms._NET_CLIENT_LIST, AtomEnum::WINDOW, 0, u32::MAX)?
                .reply()?;
            for win in reply.value32().ok_or(WindowManagerError::PropertyNotFound)? {
                windows.push(Window { id: win })
            }
        }
        Ok(windows)
    }

    /// Get window manager's window id and name
    pub fn winmgr(&self) -> WindowManagerResult<(u32, String)> {
        let win: u32 = self.get_root_property(self.atoms._NET_SUPPORTING_WM_CHECK, AtomEnum::WINDOW).try_into()?;
        let name = self.win_name(win)?;
        Ok((win, name))
    }

    /// Get desktop work area
    ///
    /// ### Examples
    /// ```ignore
    /// use libewmh::prelude::*;
    /// let wmcli = wmcli::connect().unwrap();
    /// let (w, h) = wmcli.workarea().unwrap();
    /// ```
    pub fn workarea(&self) -> WindowManagerResult<(u16, u16)> {
        // Defined as: _NET_WORKAREA, x, y, width, height CARDINAL[][4]/32
        // which means when retrieving the value via `get_property` that we need to use a `self.atoms._NET_WORKAREA`
        // request message with a `AtomEnum::CARDINAL` type response and we can use the `reply.value32()` accessor to
        // retrieve the values of which there will be 4 for each desktop as defined (x, y, width, height).
        let reply = self
            .conn
            .get_property(false, self.root, self.atoms._NET_WORKAREA, AtomEnum::CARDINAL, 0, u32::MAX)?
            .reply()?;
        let mut values = reply.value32().ok_or(WindowManagerError::PropertyNotFound)?;
        let x = values.next().ok_or(WindowManagerError::PropertyNotFound)?;
        let y = values.next().ok_or(WindowManagerError::PropertyNotFound)?;
        let w = values.next().ok_or(WindowManagerError::PropertyNotFound)?;
        let h = values.next().ok_or(WindowManagerError::PropertyNotFound)?;
        debug!("work_area: x: {}, y: {}, w: {}, h: {}", x, y, w, h);

        // x and y are always zero so dropping them
        Ok((w as u16, h as u16))
    }

    /// Get window attribrtes
    ///
    /// ### Arguments
    /// * `win` - id of the window to manipulate
    ///
    /// ### Examples
    /// ```ignore
    /// use libewmh::prelude::*;
    /// let wmcli = wmcli::connect().unwrap();
    /// let (class, state) = wmcli.win_attributes(12345).unwrap();
    /// ```
    #[allow(dead_code)]
    pub fn win_attributes(&self, win: xproto::Window) -> WindowManagerResult<(WinClass, WinMap)> {
        let attr = self.conn.get_window_attributes(win)?.reply()?;
        debug!(
            "win_attributes: id: {}, win_gravity: {:?}, bit_gravity: {:?}",
            win, attr.win_gravity, attr.bit_gravity
        );
        Ok((WinClass::from(attr.class.into())?, WinMap::from(attr.map_state.into())?))
    }

    /// Get window class which ends up being the applications name
    ///
    /// ### Arguments
    /// * `win` - id of the window to manipulate
    ///
    /// ### Examples
    /// ```ignore
    /// use libewmh::prelude::*;
    /// let wmcli = wmcli::connect().unwrap();
    /// let class = wmcli.win_class(12345).unwrap();
    /// ```
    pub fn win_class(&self, win: xproto::Window) -> WindowManagerResult<String> {
        let reply =
            self.conn.get_property(false, win, AtomEnum::WM_CLASS, AtomEnum::STRING, 0, u32::MAX)?.reply()?;

        // Skip the first null terminated string and extract the second
        let iter = reply.value.into_iter().skip_while(|x| *x != 0).skip(1).take_while(|x| *x != 0);

        // Extract the second null terminated string
        let class = str::from_utf8(&iter.collect::<Vec<_>>())?.to_owned();
        debug!("win_class: id: {}, class: {}", win, class);
        Ok(class)
    }

    /// Get window desktop
    ///
    /// ### Arguments
    /// * `win` - id of the window to manipulate
    /// ```
    pub fn win_desktop(&self, win: xproto::Window) -> WindowManagerResult<i32> {
        // Defined as: _NET_WM_DESKTOP desktop, CARDINAL/32
        // FIXME why i32?!!
        self.get_window_property(win, self.atoms._NET_WM_DESKTOP, AtomEnum::CARDINAL).try_into()
    }

    /// Get window frame border values added by the window manager
    ///
    /// ### Arguments
    /// * `win` - id of the window to manipulate
    ///
    /// ### Examples
    /// ```ignore
    /// use libewmh::prelude::*;
    /// let wmcli = wmcli::connect().unwrap();
    /// let (l, r, t, b) = wmcli.win_borders(12345).unwrap();
    /// ```
    pub fn win_borders(&self, win: xproto::Window) -> WindowManagerResult<(u32, u32, u32, u32)> {
        // Defined as: _NET_FRAME_EXTENTS, left, right, top, bottom, CARDINAL[4]/32
        // which means when retrieving the value via `get_property` that we need to use a `self.atoms._NET_FRAME_EXTENTS`
        // request message with a `AtomEnum::CARDINAL` type response and we can use the `reply.value32()` accessor to
        // retrieve the values of which there will be...
        let reply = self
            .conn
            .get_property(false, win, self.atoms._NET_FRAME_EXTENTS, AtomEnum::CARDINAL, 0, u32::MAX)?
            .reply()?;
        let mut values = reply.value32().ok_or(WindowManagerError::PropertyNotFound)?;
        let l = values.next().ok_or(WindowManagerError::PropertyNotFound)?;
        let r = values.next().ok_or(WindowManagerError::PropertyNotFound)?;
        let t = values.next().ok_or(WindowManagerError::PropertyNotFound)?;
        let b = values.next().ok_or(WindowManagerError::PropertyNotFound)?;
        debug!("win_borders: id: {}, l: {}, r: {}, t: {}, b: {}", win, l, r, t, b);
        Ok((l, r, t, b))
    }

    /// Get window geometry
    ///
    /// ### Arguments
    /// * `win` - id of the window to manipulate
    ///
    /// ### Examples
    /// ```ignore
    /// use libewmh::prelude::*;
    /// let wmcli = wmcli::connect().unwrap();
    /// let (x, y, w, h) = wmcli.win_geometry(12345).unwrap();
    /// ```
    pub fn win_geometry(&self, win: xproto::Window) -> WindowManagerResult<(i32, i32, u32, u32)> {
        // The returned x, y location is relative to its parent window making the values completely
        // useless. However using `translate_coordinates` we can have the window manager map those
        // useless values into real world cordinates by passing it the root as the relative window.

        // Get width and heith and useless relative location values
        let g = self.conn.get_geometry(win)?.reply()?;

        // Translate the useless retative location values to to real world values
        let t = self.conn.translate_coordinates(win, self.root, g.x, g.y)?.reply()?;

        let (x, y, w, h) = (t.dst_x, t.dst_y, g.width, g.height);
        debug!("win_geometry: id: {}, x: {}, y: {}, w: {}, h: {}", win, x, y, w, h);
        Ok((x as i32, y as i32, w as u32, h as u32))
    }

    /// Get window name
    ///
    /// ### Arguments
    /// * `win` - id of the window to manipulate
    ///
    /// ### Examples
    /// ```ignore
    /// use libewmh::prelude::*;
    /// let wmcli = wmcli::connect().unwrap();
    /// let name = wmcli.win_name(12345).unwrap();
    /// ```
    pub fn win_name(&self, win: xproto::Window) -> WindowManagerResult<String> {
        // Defined as: _NET_WM_NAME, UTF8_STRING
        // which means when retrieving the value via `get_property` that we need to use a `self.atoms._NET_WM_NAME`
        // request message with a `AtomEnum::UTF8_STRING` type response and we can use the `reply.value` accessor to
        // retrieve the value.

        // First try the _NET_WM_VISIBLE_NAME
        let reply = self
            .conn
            .get_property(false, win, self.atoms._NET_WM_VISIBLE_NAME, self.atoms.UTF8_STRING, 0, u32::MAX)?
            .reply()?;
        if reply.type_ != x11rb::NONE {
            if let Ok(value) = str::from_utf8(&reply.value) {
                if value != "" {
                    debug!("win_name: using _NET_WM_VISIBLE_NAME for: {}", value);
                    return Ok(value.to_owned());
                }
            }
        }

        // Next try the _NET_WM_NAME
        let reply = self
            .conn
            .get_property(false, win, self.atoms._NET_WM_NAME, self.atoms.UTF8_STRING, 0, u32::MAX)?
            .reply()?;
        if reply.type_ != x11rb::NONE {
            if let Ok(value) = str::from_utf8(&reply.value) {
                if value != "" {
                    debug!("win_name: using _NET_WM_NAME for: {}", value);
                    return Ok(value.to_owned());
                }
            }
        }

        // Fall back on the WM_NAME
        let reply =
            self.conn.get_property(false, win, AtomEnum::WM_NAME, AtomEnum::STRING, 0, u32::MAX)?.reply()?;
        if reply.type_ != x11rb::NONE {
            if let Ok(value) = str::from_utf8(&reply.value) {
                if value != "" {
                    debug!("win_name: using WM_NAME for: {}", value);
                    return Ok(value.to_owned());
                }
            }
        }

        // No valid name was found
        Err(WindowManagerError::PropertyNotFound.into())
    }

    /// Get window parent
    ///
    /// ### Arguments
    /// * `win` - id of the window to manipulate
    ///
    /// ### Examples
    /// ```ignore
    /// use libewmh::prelude::*;
    /// let wmcli = wmcli::connect().unwrap();
    /// let parent = wmcli.win_parent(12345).unwrap();
    /// ```
    #[allow(dead_code)]
    pub fn win_parent(&self, win: xproto::Window) -> WindowManagerResult<u32> {
        let tree = self.conn.query_tree(win)?.reply()?;
        let id = tree.parent;
        debug!("win_parent: id: {}, parent: {:?}", win, id);
        Ok(id)
    }

    /// Get window pid
    ///
    /// ### Arguments
    /// * `win` - id of the window to manipulate
    ///
    /// ### Examples
    /// ```ignore
    /// use libewmh::prelude::*;
    /// let wmcli = wmcli::connect().unwrap();
    /// let pid = wmcli.win_pid(12345).unwrap();
    /// ```
    pub fn win_pid(&self, win: xproto::Window) -> WindowManagerResult<i32> {
        // Defined as: _NET_WM_PID, CARDINAL/32
        self.get_window_property(win, self.atoms._NET_WM_PID, AtomEnum::CARDINAL).try_into()
    }

    /// Get window state
    ///
    /// ### Arguments
    /// * `win` - id of the window to manipulate
    ///
    /// ### Examples
    /// ```ignore
    /// use libewmh::prelude::*;
    /// let wmcli = wmcli::connect().unwrap();
    /// let state = wmcli.win_state(12345).unwrap();
    /// ```
    pub fn win_state(&self, win: xproto::Window) -> WindowManagerResult<Vec<WinState>> {
        // Defined as: _NET_WM_STATE, ATOM[]
        // which means when retrieving the value via `get_property` that we need to use a `self.atoms._NET_WM_STATE`
        // request message with a `AtomEnum::ATOM` type response and we can use the `reply.value32()` accessor to
        // retrieve the values of which there will be a single value.
        let mut states = vec![];
        let reply =
            self.conn.get_property(false, win, self.atoms._NET_WM_STATE, AtomEnum::ATOM, 0, u32::MAX)?.reply()?;
        for state in reply.value32().ok_or(WindowManagerError::PropertyNotFound)? {
            let state = WinState::from(&self.atoms, state);
            debug!("win_state: id: {}, state: {}", win, state);
            states.push(state);
        }
        Ok(states)
    }

    /// Get window type
    ///
    /// ### Arguments
    /// * `win` - id of the window to manipulate
    ///
    /// ### Examples
    /// ```ignore
    /// use libewmh::prelude::*;
    /// let wmcli = wmcli::connect().unwrap();
    /// let type_ = wmcli.win_type(12345).unwrap();
    /// ```
    pub fn win_type(&self, win: xproto::Window) -> WindowManagerResult<WinType> {
        // Defined as: _NET_WM_WINDOW_TYPE, ATOM[]/32
        // which means when retrieving the value via `get_property` that we need to use a `self.atoms._NET_WM_WINDOW_TYPE`
        // request message with a `AtomEnum::ATOM` type response and we can use the `reply.value32()` accessor to
        // retrieve the value.
        Ok(WinType::from(
            &self.atoms,
            self.get_window_property(win, self.atoms._NET_WM_WINDOW_TYPE, AtomEnum::ATOM).try_into()?,
        ))
    }

    // Helper method to print out the data type
    // println!("DataType NET: {:?}", AtomEnum::from(reply.type_ as u8));
    #[allow(dead_code)]
    fn print_data_type(reply: &GetPropertyReply) {
        println!("DataType: {:?}", AtomEnum::from(reply.type_ as u8));
    }
}
