//! Contains the error type of the library

use thiserror::Error;

use vigem_client_c_sys as ffi;

/// Represents all possible errors in the library
#[derive(Error, Debug, Clone, Copy)]
pub enum Error {
    #[error("Failed to allocate client")]
    NoVigemAlloc,

    #[error("Failed to allocate xbox 360 pad")]
    NoX360PadAlloc,

    #[error("Bus not found")]
    BusNotFound,

    #[error("No free slot")]
    NoFreeSlot,

    #[error("Invalid target")]
    InvalidTarget,

    #[error("Removal failed")]
    RemovalFailed,

    #[error("Already connected")]
    AlreadyConnected,

    #[error("Target uninitialized")]
    TargetUninitialized,

    #[error("Target not plugged in")]
    TargetNotPluggedIn,

    #[error("Bus version mismatch")]
    BusVersionMismatch,

    #[error("Bus access failed")]
    BusAccessFailed,

    #[error("The same callback already has been registered")]
    CallbackAlreadyRegistered,

    #[error("Another callback has already been registered")]
    AlreadyHasCallback,

    #[error("Callback not found")]
    CallbackNotFound,

    #[error("Bus already connected")]
    BusAlreadyConnected,

    #[error("Invalid bus handle")]
    BusInvalidHandle,

    #[error("User index out of range")]
    UserIndexOutOfRange,

    #[error("Invalid parameter")]
    InvalidParameter,

    #[error("Not supported")]
    NotSupported,

    #[error("Unknown error code {0:x}")]
    UnknownError(ffi::_VIGEM_ERRORS),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

pub(crate) fn check(error: ffi::_VIGEM_ERRORS) -> Result<()> {
    Err(match error {
        ffi::_VIGEM_ERRORS_VIGEM_ERROR_NONE => return Ok(()),

        ffi::_VIGEM_ERRORS_VIGEM_ERROR_BUS_NOT_FOUND => Error::BusNotFound,
        ffi::_VIGEM_ERRORS_VIGEM_ERROR_NO_FREE_SLOT => Error::NoFreeSlot,
        ffi::_VIGEM_ERRORS_VIGEM_ERROR_INVALID_TARGET => Error::InvalidTarget,
        ffi::_VIGEM_ERRORS_VIGEM_ERROR_REMOVAL_FAILED => Error::RemovalFailed,
        ffi::_VIGEM_ERRORS_VIGEM_ERROR_ALREADY_CONNECTED => Error::AlreadyConnected,
        ffi::_VIGEM_ERRORS_VIGEM_ERROR_TARGET_UNINITIALIZED => Error::TargetUninitialized,
        ffi::_VIGEM_ERRORS_VIGEM_ERROR_TARGET_NOT_PLUGGED_IN => Error::TargetNotPluggedIn,
        ffi::_VIGEM_ERRORS_VIGEM_ERROR_BUS_VERSION_MISMATCH => Error::BusVersionMismatch,
        ffi::_VIGEM_ERRORS_VIGEM_ERROR_BUS_ACCESS_FAILED => Error::BusAccessFailed,
        ffi::_VIGEM_ERRORS_VIGEM_ERROR_CALLBACK_ALREADY_REGISTERED => {
            Error::CallbackAlreadyRegistered
        }
        ffi::_VIGEM_ERRORS_VIGEM_ERROR_CALLBACK_NOT_FOUND => Error::CallbackNotFound,
        ffi::_VIGEM_ERRORS_VIGEM_ERROR_BUS_ALREADY_CONNECTED => Error::BusAlreadyConnected,
        ffi::_VIGEM_ERRORS_VIGEM_ERROR_BUS_INVALID_HANDLE => Error::BusInvalidHandle,
        ffi::_VIGEM_ERRORS_VIGEM_ERROR_XUSB_USERINDEX_OUT_OF_RANGE => Error::UserIndexOutOfRange,
        ffi::_VIGEM_ERRORS_VIGEM_ERROR_INVALID_PARAMETER => Error::InvalidParameter,
        ffi::_VIGEM_ERRORS_VIGEM_ERROR_NOT_SUPPORTED => Error::NotSupported,

        _ => Error::UnknownError(error),
    })
}
