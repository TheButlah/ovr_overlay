pub mod overlay;

mod errors;

pub use self::errors::EVRInitError;
pub use ovr_overlay_sys as sys;

use self::overlay::IOverlay;

use derive_more::From;
use lazy_static::lazy_static;
use std::sync::Mutex;
use thiserror::Error;

lazy_static! {
    // Mutex instead of atomic allows for blocking on lock
    static ref INITIALIZED: Mutex<bool> = Mutex::new(false);
}

/// Represents an active OpenVR context.
///
/// Shutting down this context is unsafe, so if this is dropped, the context will
/// remain active, as leaking resources is better than accidentally causing unsafe
/// behavior. To actually shut down, call [`Self::shutdown()`]..
pub struct Context {}
impl Context {
    pub fn init() -> Result<Self, InitError> {
        if let Ok(guard) = INITIALIZED.try_lock() {
            if *guard {
                return Err(InitError::AlreadyInitialized);
            }
            let mut error = std::mem::MaybeUninit::<sys::EVRInitError>::uninit();
            let error = unsafe {
                let _ = sys::VR_Init(
                    error.as_mut_ptr(),
                    sys::EVRApplicationType::VRApplication_Overlay,
                    std::ptr::null(),
                );
                EVRInitError(error.assume_init())
            };
            if error.0 != sys::EVRInitError::VRInitError_None {
                return Err(InitError::Sys(error));
            }
            Ok(Self {})
        } else {
            Err(InitError::AlreadyInitialized)
        }
    }

    // TODO: is this actually unsafe?
    // see https://docs.rs/openvr/latest/openvr/struct.Context.html#safety
    pub unsafe fn shutdown(self) {
        sys::VR_Shutdown()
    }

    pub fn overlay(&self) -> IOverlay<'_> {
        IOverlay::new(self)
    }
}

#[derive(Error, Debug, From)]
pub enum InitError {
    #[error("OpenVR already initialized")]
    AlreadyInitialized,
    #[error("sys::{0}")]
    Sys(EVRInitError),
}
