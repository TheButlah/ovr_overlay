use crate::sys;

use derive_more::From;
use std::{ffi::CStr, fmt::Display};

#[derive(From, Debug, Copy, Clone, PartialEq, Eq)]
pub struct EVRInitError(pub sys::EVRInitError);
impl EVRInitError {
    pub fn description(&self) -> &'static str {
        let desc: &'static CStr = unsafe { CStr::from_ptr(sys::VR_GetVRInitErrorAsSymbol(self.0)) };
        desc.to_str().unwrap()
    }
}
impl Display for EVRInitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let num = self.0 as u8;
        let desc = self.description();
        write!(f, "EVRInitError({num}): {desc}")
    }
}

#[derive(From, Debug, Copy, Clone, PartialEq, Eq)]
pub struct EVROverlayError(pub sys::EVROverlayError);
impl EVROverlayError {
    pub fn description(&self) -> &'static str {
        "todo"
    }
}
impl Display for EVROverlayError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let num = self.0 as u8;
        let desc = self.description();
        write!(f, "EVRInitError({num}): {desc}")
    }
}
