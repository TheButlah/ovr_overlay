use crate::sys;

use derive_more::From;
use std::ffi::CStr;
use std::fmt::Display;

#[derive(Debug, Copy, Clone, PartialEq, Eq, thiserror::Error)]
pub struct EVRInitError(sys::EVRInitError);
impl EVRInitError {
    pub fn new(err: sys::EVRInitError) -> Option<Self> {
        if err == sys::EVRInitError::VRInitError_None {
            None
        } else {
            Some(Self(err))
        }
    }

    pub fn description(&self) -> &'static str {
        let desc: &'static CStr = unsafe { CStr::from_ptr(sys::VR_GetVRInitErrorAsSymbol(self.0)) };
        desc.to_str().unwrap()
    }

    pub fn inner(&self) -> sys::EVRInitError {
        self.0
    }
}
impl Display for EVRInitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let num = self.0 as u8;
        let desc = self.description();
        write!(f, "EVRInitError({num}): {desc}")
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, thiserror::Error)]
pub struct EVROverlayError(sys::EVROverlayError);
impl EVROverlayError {
    pub fn new(err: sys::EVROverlayError) -> Option<Self> {
        if err == sys::EVROverlayError::VROverlayError_None {
            None
        } else {
            Some(Self(err))
        }
    }

    pub fn description(&self) -> &'static str {
        "todo"
    }

    pub fn inner(&self) -> sys::EVROverlayError {
        self.0
    }
}
impl Display for EVROverlayError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let num = self.0 as u8;
        let desc = self.description();
        write!(f, "EVROverlayError({num}): {desc}")
    }
}

#[derive(Debug, From, thiserror::Error)]
pub enum InitError {
    #[error("OpenVR already initialized")]
    AlreadyInitialized,
    #[error("sys::{0}")]
    Sys(EVRInitError),
}
