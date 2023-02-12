//! Create a [`Context`] to get started.
//!
//! **This library makes no semver guarantees until version 0.1.0 or greater.**

#[cfg(feature = "ovr_chaperone_setup")]
pub mod chaperone_setup;
#[cfg(feature = "ovr_input")]
pub mod input;
pub mod overlay;
pub mod pose;

mod errors;

pub use self::errors::{EVRInitError, InitError};
pub use ovr_overlay_sys as sys;

use self::overlay::OverlayManager;

#[cfg(feature = "ovr_chaperone_setup")]
use self::chaperone_setup::ChaperoneSetupManager;
#[cfg(feature = "ovr_input")]
use self::input::InputManager;

use derive_more::{From, Into};
use lazy_static::lazy_static;
use std::fmt::Debug;
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

    #[cfg(feature = "ovr_chaperone_setup")]
    pub fn chaperone_setup_mngr(&self) -> ChaperoneSetupManager<'_> {
        ChaperoneSetupManager::new(self)
    }

    #[cfg(feature = "ovr_input")]
    pub fn input_mngr(&self) -> InputManager<'_> {
        InputManager::new(self)
    }
}

/// Tints each color channel by multiplying it with the given f32
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct ColorTint {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}
impl Default for ColorTint {
    fn default() -> Self {
        Self {
            r: 1.,
            g: 1.,
            b: 1.,
            a: 1.,
        }
    }
}

#[derive(From, Into)]
pub struct TextureBounds(pub sys::VRTextureBounds_t);
impl Clone for TextureBounds {
    fn clone(&self) -> Self {
        Self(sys::VRTextureBounds_t {
            uMin: self.0.uMin,
            vMin: self.0.vMin,
            uMax: self.0.uMax,
            vMax: self.0.vMax,
        })
    }
}
impl Debug for TextureBounds {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TextureBounds")
            .field("uMin", &self.0.uMin)
            .field("vMin", &self.0.vMin)
            .field("uMax", &self.0.uMax)
            .field("vMax", &self.0.vMax)
            .finish()
    }
}

#[derive(Clone, Copy, From, Into)]
#[repr(transparent)]
pub struct TrackedDeviceIndex(pub sys::TrackedDeviceIndex_t);
impl TrackedDeviceIndex {
    pub const fn new(index: sys::TrackedDeviceIndex_t) -> Result<Self, ()> {
        if index == sys::k_unTrackedDeviceIndexInvalid {
            // TODO: Is this ever going to come up from an otherwise successful result?
            Err(())
        } else {
            Ok(Self(index))
        }
    }

    /// Device index for the HMD
    const HMD: Self = Self(sys::k_unTrackedDeviceIndex_Hmd);

    /// Maximum number of Tracked Devices
    const MAX: usize = sys::k_unMaxTrackedDeviceCount as usize;

    // Please open an issue on the github repository if you need this.
    // pub const fn is_other(&self) -> bool {
    //     self.0 == sys::k_unTrackedDeviceIndexOther
    // }
}

#[cfg(test)]
mod tests {
    #[test]
    fn ensure_testing_optional_features() {
        macro_rules! helper {
            ($($feature:literal),+ $(,)?) => {
                $(assert!(cfg!(feature = $feature), "use `cargo test --all-features` instead!"));+
            };
        }

        helper!("nalgebra");
    }
}
