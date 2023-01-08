use crate::{errors::EVRInputError, pose, sys, Context};

use derive_more::{From, Into};
use std::ffi::{CStr, CString};
use std::marker::PhantomData;
use std::mem::MaybeUninit;
use std::path::Path;
use std::pin::Pin;
use std::time::Duration;

pub struct InputManager<'c> {
    ctx: PhantomData<&'c Context>,
    inner: Pin<&'c mut sys::IVRInput>,
}

#[derive(From, Into, Debug, PartialEq, Eq, Clone, Copy)]
#[repr(transparent)]
pub struct ActionSetHandle(sys::VRActionSetHandle_t);

#[derive(From, Into, Debug, PartialEq, Eq, Clone, Copy)]
#[repr(transparent)]
pub struct ActionHandle(sys::VRActionHandle_t);

#[derive(From, Into, Debug, PartialEq, Eq, Clone, Copy)]
#[repr(transparent)]
pub struct InputValueHandle(sys::VRInputValueHandle_t);

type Result<T> = std::result::Result<T, EVRInputError>;

pub trait ToSeconds {
    fn to_seconds(self) -> f32;
}

impl ToSeconds for f32 {
    fn to_seconds(self) -> f32 {
        self
    }
}

impl ToSeconds for &f32 {
    fn to_seconds(self) -> f32 {
        *self
    }
}

impl ToSeconds for &Duration {
    fn to_seconds(self) -> f32 {
        self.as_secs_f32()
    }
}

impl<'c> InputManager<'c> {
    pub(super) fn new(_ctx: &'c Context) -> Self {
        let inner = unsafe { Pin::new_unchecked(sys::VRInput().as_mut::<'c>().unwrap()) };
        Self {
            ctx: Default::default(),
            inner,
        }
    }

    // ---- Handle Management ----

    pub fn set_action_manifest(&mut self, path: &Path) -> Result<()> {
        let path = if let Ok(s) = CString::new(path.to_string_lossy().as_bytes()) {
            s
        } else {
            return EVRInputError::new(sys::EVRInputError::VRInputError_InvalidParam);
        };
        self.set_action_manifest_raw(&path)
    }

    pub fn set_action_manifest_raw(&mut self, path: &CStr) -> Result<()> {
        let err = unsafe { self.inner.as_mut().SetActionManifestPath(path.as_ptr()) };
        EVRInputError::new(err)
    }

    pub fn get_action_set_handle(&mut self, name: &str) -> Result<ActionSetHandle> {
        let name = if let Ok(s) = CString::new(name) {
            s
        } else {
            return Err(sys::EVRInputError::VRInputError_InvalidParam.into());
        };

        self.get_action_set_handle_raw(&name)
    }

    pub fn get_action_set_handle_raw(&mut self, name: &CStr) -> Result<ActionSetHandle> {
        let mut handle: sys::VRActionSetHandle_t = 0;

        let err = unsafe {
            self.inner
                .as_mut()
                .GetActionSetHandle(name.as_ptr(), &mut handle)
        };

        EVRInputError::new(err)?;
        Ok(ActionSetHandle(handle))
    }

    pub fn get_action_handle(&mut self, name: &str) -> Result<ActionHandle> {
        let name = if let Ok(s) = CString::new(name) {
            s
        } else {
            return EVRInputError::new(sys::EVRInputError::VRInputError_InvalidParam)
                .map(|_| unreachable!());
        };

        self.get_action_handle_raw(&name)
    }

    pub fn get_action_handle_raw(&mut self, name: &CStr) -> Result<ActionHandle> {
        let mut handle: sys::VRActionHandle_t = 0;

        let err = unsafe {
            self.inner
                .as_mut()
                .GetActionHandle(name.as_ptr(), &mut handle)
        };

        EVRInputError::new(err)?;
        Ok(ActionHandle(handle))
    }

    pub fn get_input_source_handle(&mut self, name: &str) -> Result<InputValueHandle> {
        let name = if let Ok(s) = CString::new(name) {
            s
        } else {
            return EVRInputError::new(sys::EVRInputError::VRInputError_InvalidParam)
                .map(|_| unreachable!());
        };

        self.get_input_source_handle_raw(&name)
    }

    pub fn get_input_source_handle_raw(&mut self, name: &CStr) -> Result<InputValueHandle> {
        let mut handle: sys::VRInputValueHandle_t = 0;

        let err = unsafe {
            self.inner
                .as_mut()
                .GetInputSourceHandle(name.as_ptr(), &mut handle)
        };

        EVRInputError::new(err)?;
        Ok(InputValueHandle(handle))
    }

    // ---- Read Action State ----

    pub fn update_actions(&mut self, sets: &mut [sys::VRActiveActionSet_t]) -> Result<()> {
        let err = unsafe {
            self.inner.as_mut().UpdateActionState(
                sets.as_mut_ptr(),
                std::mem::size_of::<sys::VRActiveActionSet_t>() as u32,
                sets.len() as u32,
            )
        };

        EVRInputError::new(err)
    }

    pub fn get_digital_action_data(
        &mut self,
        action: ActionHandle,
        restrict: InputValueHandle,
    ) -> Result<sys::InputDigitalActionData_t> {
        let mut data: MaybeUninit<sys::InputDigitalActionData_t> = MaybeUninit::uninit();
        let err = unsafe {
            self.inner.as_mut().GetDigitalActionData(
                action.0,
                data.as_mut_ptr(),
                std::mem::size_of::<sys::VRActiveActionSet_t>() as u32,
                restrict.0,
            )
        };
        EVRInputError::new(err)?;
        Ok(unsafe { data.assume_init() })
    }

    pub fn get_pose_action_data_relative_to_now(
        &mut self,
        action: ActionHandle,
        universe: pose::TrackingUniverseOrigin,
        seconds_from_now: impl ToSeconds,
        restrict: InputValueHandle,
    ) -> Result<sys::InputPoseActionData_t> {
        let mut data: MaybeUninit<sys::InputPoseActionData_t> = MaybeUninit::uninit();
        let err = unsafe {
            self.inner.as_mut().GetPoseActionDataRelativeToNow(
                action.0,
                universe,
                seconds_from_now.to_seconds(),
                data.as_mut_ptr(),
                std::mem::size_of::<sys::InputPoseActionData_t>() as u32,
                restrict.0,
            )
        };

        EVRInputError::new(err)?;
        Ok(unsafe { data.assume_init() })
    }

    // ---- Action Origins ----

    // TODO: GetOriginLocalizedName -- this is gonna want a nice bitset UI

    pub fn get_origin_tracked_device_info(
        &mut self,
        origin: InputValueHandle,
    ) -> Result<sys::InputOriginInfo_t> {
        let mut data: MaybeUninit<sys::InputOriginInfo_t> = MaybeUninit::uninit();
        let err = unsafe {
            self.inner.as_mut().GetOriginTrackedDeviceInfo(
                origin.0,
                data.as_mut_ptr(),
                std::mem::size_of::<sys::InputOriginInfo_t>() as u32,
            )
        };

        EVRInputError::new(err)?;
        Ok(unsafe { data.assume_init() })
    }
}
