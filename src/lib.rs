pub mod overlay;

mod errors;

pub use self::errors::{EVRInitError, InitError};
pub use ovr_overlay_sys as sys;

use self::overlay::OverlayManager;

use lazy_static::lazy_static;
use std::sync::Mutex;

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
            let mut err = std::mem::MaybeUninit::<sys::EVRInitError>::uninit();
            let err = unsafe {
                let _ = sys::VR_Init(
                    err.as_mut_ptr(),
                    sys::EVRApplicationType::VRApplication_Overlay,
                    std::ptr::null(),
                );
                err.assume_init()
            };
            EVRInitError::new(err)?;
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

    pub fn overlay_mngr(&self) -> OverlayManager<'_> {
        OverlayManager::new(self)
    }
}

/// Tints each color channel by multiplying it with the given f32
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct ColorTint {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}
impl Default for ColorTint {
    fn default() -> Self {
        Self {
            r: 1.,
            g: 1.,
            b: 1.,
        }
    }
}
