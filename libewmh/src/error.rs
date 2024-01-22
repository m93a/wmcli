use std::error::Error as StdError;
use std::fmt;

/// `WmResult<T>` provides a simplified result type with a common error type
pub type WindowManagerResult<T> = std::result::Result<T, ErrorWrapper>;

/// WmcliError defines all the internal errors that `libewmh` might return
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum WindowManagerError {
    DesktopWinNotFound,
    InvalidAtom(String),
    InvalidWinGravity(u32),
    InvalidWinPosition(String),
    InvalidWinShape(String),
    InvalidWinClass(u32),
    InvalidWinMap(u32),
    InvalidWinState(u32),
    InvalidWinType(u32),
    PropertyNotFound(String),
    TaskbarNotFound,
    TaskbarReservationNotFound,
}
impl std::error::Error for WindowManagerError {}
impl fmt::Display for WindowManagerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            WindowManagerError::DesktopWinNotFound => write!(f, "desktop window was not found"),
            WindowManagerError::InvalidAtom(ref err) => write!(f, "invalid atom was given: {}", err),
            WindowManagerError::InvalidWinGravity(ref err) => write!(f, "invalid gravity was given: {}", err),
            WindowManagerError::InvalidWinPosition(ref err) => write!(f, "invalid position was given: {}", err),
            WindowManagerError::InvalidWinShape(ref err) => write!(f, "invalid shape was given: {}", err),
            WindowManagerError::InvalidWinClass(ref err) => write!(f, "invalid class was given: {}", err),
            WindowManagerError::InvalidWinMap(ref err) => write!(f, "invalid map was given: {}", err),
            WindowManagerError::InvalidWinState(ref err) => write!(f, "invalid state was given: {}", err),
            WindowManagerError::InvalidWinType(ref err) => write!(f, "invalid type was given: {}", err),
            WindowManagerError::PropertyNotFound(ref err) => write!(f, "property {} was not found", err),
            WindowManagerError::TaskbarNotFound => write!(f, "taskbar not found"),
            WindowManagerError::TaskbarReservationNotFound => write!(f, "taskbar reservation not found"),
        }
    }
}

/// ErrorWrapper provides wrapper around all the underlying library dependencys that `libewmh` uses
/// such that we can easily surface all errors from `libwmctdl` in a single easy way.
#[derive(Debug)]
pub enum ErrorWrapper {
    WindowManager(WindowManagerError),

    // std::str::Utf8Error
    Utf8(std::str::Utf8Error),

    // x11rb errors
    Connect(x11rb::errors::ConnectError),
    Connection(x11rb::errors::ConnectionError),
    Reply(x11rb::errors::ReplyError),
}
impl ErrorWrapper {
    /// Implemented directly on the `Error` type to reduce casting required
    pub fn is<T: StdError + 'static>(&self) -> bool {
        self.as_ref().is::<T>()
    }

    /// Implemented directly on the `Error` type to reduce casting required
    pub fn downcast_ref<T: StdError + 'static>(&self) -> Option<&T> {
        self.as_ref().downcast_ref::<T>()
    }

    /// Implemented directly on the `Error` type to reduce casting required
    pub fn downcast_mut<T: StdError + 'static>(&mut self) -> Option<&mut T> {
        self.as_mut().downcast_mut::<T>()
    }

    /// Implemented directly on the `Error` type to reduce casting required
    /// which allows for using as_ref to get the correct pass through.
    pub fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.as_ref().source()
    }
}
impl StdError for ErrorWrapper {}

impl fmt::Display for ErrorWrapper {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ErrorWrapper::WindowManager(ref err) => write!(f, "{}", err),
            ErrorWrapper::Utf8(ref err) => write!(f, "{}", err),
            ErrorWrapper::Connect(ref err) => write!(f, "{}", err),
            ErrorWrapper::Connection(ref err) => write!(f, "{}", err),
            ErrorWrapper::Reply(ref err) => write!(f, "{}", err),
        }
    }
}

impl AsRef<dyn StdError> for ErrorWrapper {
    fn as_ref(&self) -> &(dyn StdError + 'static) {
        match *self {
            ErrorWrapper::WindowManager(ref err) => err,
            ErrorWrapper::Utf8(ref err) => err,
            ErrorWrapper::Connect(ref err) => err,
            ErrorWrapper::Connection(ref err) => err,
            ErrorWrapper::Reply(ref err) => err,
        }
    }
}

impl AsMut<dyn StdError> for ErrorWrapper {
    fn as_mut(&mut self) -> &mut (dyn StdError + 'static) {
        match *self {
            ErrorWrapper::WindowManager(ref mut err) => err,
            ErrorWrapper::Utf8(ref mut err) => err,
            ErrorWrapper::Connect(ref mut err) => err,
            ErrorWrapper::Connection(ref mut err) => err,
            ErrorWrapper::Reply(ref mut err) => err,
        }
    }
}

impl From<WindowManagerError> for ErrorWrapper {
    fn from(err: WindowManagerError) -> ErrorWrapper {
        ErrorWrapper::WindowManager(err)
    }
}

impl From<std::str::Utf8Error> for ErrorWrapper {
    fn from(err: std::str::Utf8Error) -> ErrorWrapper {
        ErrorWrapper::Utf8(err)
    }
}

// x11rb errors
//--------------------------------------------------------------------------------------------------
impl From<x11rb::errors::ConnectError> for ErrorWrapper {
    fn from(err: x11rb::errors::ConnectError) -> ErrorWrapper {
        ErrorWrapper::Connect(err)
    }
}

impl From<x11rb::errors::ConnectionError> for ErrorWrapper {
    fn from(err: x11rb::errors::ConnectionError) -> ErrorWrapper {
        ErrorWrapper::Connection(err)
    }
}

impl From<x11rb::errors::ReplyError> for ErrorWrapper {
    fn from(err: x11rb::errors::ReplyError) -> ErrorWrapper {
        ErrorWrapper::Reply(err)
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_errors() {}
}
