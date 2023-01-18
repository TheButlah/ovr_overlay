use crate::sys;

use derive_more::{From, Into};
use std::ffi::CStr;
use std::fmt::Display;

#[derive(Debug, Copy, Clone, PartialEq, Eq, thiserror::Error)]
pub struct EVRInitError(sys::EVRInitError);
impl EVRInitError {
    pub fn new(err: sys::EVRInitError) -> Result<(), Self> {
        if err == sys::EVRInitError::VRInitError_None {
            Ok(())
        } else {
            Err(Self(err))
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
    pub fn new(err: sys::EVROverlayError) -> Result<(), Self> {
        if err == sys::EVROverlayError::VROverlayError_None {
            Ok(())
        } else {
            Err(Self(err))
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

#[cfg(feature = "ovr_system")]
#[derive(From, Into, Debug, Copy, Clone, PartialEq, Eq, thiserror::Error)]
#[repr(transparent)]
pub struct ETrackedPropertyError(sys::ETrackedPropertyError);

#[cfg(feature = "ovr_system")]
impl ETrackedPropertyError {
    pub fn new(err: sys::ETrackedPropertyError) -> Result<(), Self> {
        if err == sys::ETrackedPropertyError::TrackedProp_Success {
            Ok(())
        } else {
            Err(Self(err))
        }
    }

    pub fn description(&self) -> &'static str {
        "todo"
    }

    pub fn inner(&self) -> sys::ETrackedPropertyError {
        self.0
    }
}

#[cfg(feature = "ovr_system")]
impl Display for ETrackedPropertyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let num = self.0 as u8;
        let desc = self.description();
        write!(f, "ETrackedPropertyError({num}): {desc}")
    }
}
#[cfg(feature = "ovr_input")]
#[derive(From, Into, Debug, Copy, Clone, PartialEq, Eq, thiserror::Error)]
#[repr(transparent)]
pub struct EVRInputError(sys::EVRInputError);

#[cfg(feature = "ovr_input")]
impl EVRInputError {
    pub fn new(err: sys::EVRInputError) -> Result<(), Self> {
        if err == sys::EVRInputError::VRInputError_None {
            Ok(())
        } else {
            Err(Self(err))
        }
    }

    pub fn description(&self) -> &'static str {
        use sys::EVRInputError::*;
        match self.0 {
            VRInputError_None => "None",
            VRInputError_NameNotFound => "NameNotFound",
            VRInputError_WrongType => "WrongType",
            VRInputError_InvalidHandle => "InvalidHandle",
            VRInputError_InvalidParam => "InvalidParam",
            VRInputError_NoSteam => "NoSteam",
            VRInputError_MaxCapacityReached => "MaxCapacityReached",
            VRInputError_IPCError => "IPCError",
            VRInputError_NoActiveActionSet => "NoActiveActionSet",
            VRInputError_InvalidDevice => "InvalidDevice",
            VRInputError_InvalidSkeleton => "InvalidSkeleton",
            VRInputError_InvalidBoneCount => "InvalidBoneCount",
            VRInputError_InvalidCompressedData => "InvalidCompressedData",
            VRInputError_NoData => "NoData",
            VRInputError_BufferTooSmall => "BufferTooSmall",
            VRInputError_MismatchedActionManifest => "MismatchedActionManifest",
            VRInputError_MissingSkeletonData => "MissingSkeletonData",
            VRInputError_InvalidBoneIndex => "InvalidBoneIndex",
            VRInputError_InvalidPriority => "InvalidPriority",
            VRInputError_PermissionDenied => "PermissionDenied",
            VRInputError_InvalidRenderModel => "InvalidRenderModel",
        }
    }

    pub fn inner(&self) -> sys::EVRInputError {
        self.0
    }
}

#[cfg(feature = "ovr_input")]
impl Display for EVRInputError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let num = self.0 as u8;
        let desc = self.description();
        write!(f, "EVRInputError({num}): {desc}")
    }
}

#[derive(Debug, From, thiserror::Error)]
pub enum InitError {
    #[error("OpenVR already initialized")]
    AlreadyInitialized,
    #[error("sys::{0}")]
    Sys(EVRInitError),
}
