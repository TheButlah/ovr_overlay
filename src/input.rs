use crate::{errors::EVRInputError, pose, sys, Context};

use derive_more::{From, Into};
use enumset::{EnumSet, EnumSetType};
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

#[derive(From, Into /*, Debug, PartialEq, Eq, Clone, Copy*/)]
#[repr(transparent)]
// TODO: do we want to do something else to forward fields to the sys struct?
pub struct ActiveActionSet(pub sys::VRActiveActionSet_t);

#[derive(From, Into /*, Debug, PartialEq, Eq, Clone, Copy*/)]
#[repr(transparent)]
pub struct DigitalActionData(pub sys::InputDigitalActionData_t);

#[derive(From, Into /*, Debug, PartialEq, Eq, Clone, Copy*/)]
#[repr(transparent)]
pub struct PoseActionData(pub sys::InputPoseActionData_t);

#[derive(From, Into /*, Debug, PartialEq, Eq, Clone, Copy*/)]
#[repr(transparent)]
pub struct OriginInfo(pub sys::InputOriginInfo_t);

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

#[derive(EnumSetType, Debug)]
#[enumset(repr = "u32")]
pub enum InputString {
    Hand,
    ControllerType,
    InputSource,
    // TODO: openvr allows you to pass a u32 with all bits set to get a string that has all information, current and future.
    //       is there a good way to represent that with enumset? do we care?
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

    pub fn update_actions(&mut self, sets: &mut [ActiveActionSet]) -> Result<()> {
        let err = unsafe {
            self.inner.as_mut().UpdateActionState(
                // this should be fine because of repr(transparent)
                // TODO: have bytemuck say it's fine or something?
                sets.as_mut_ptr() as *mut sys::VRActiveActionSet_t,
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
    ) -> Result<DigitalActionData> {
        let mut data: MaybeUninit<sys::InputDigitalActionData_t> = MaybeUninit::uninit();
        let err = unsafe {
            self.inner.as_mut().GetDigitalActionData(
                action.0,
                data.as_mut_ptr(),
                std::mem::size_of::<sys::InputDigitalActionData_t>() as u32,
                restrict.0,
            )
        };
        EVRInputError::new(err)?;
        Ok(DigitalActionData(unsafe { data.assume_init() }))
    }

    pub fn get_pose_action_data_relative_to_now(
        &mut self,
        action: ActionHandle,
        universe: pose::TrackingUniverseOrigin,
        seconds_from_now: impl ToSeconds,
        restrict: InputValueHandle,
    ) -> Result<PoseActionData> {
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
        Ok(PoseActionData(unsafe { data.assume_init() }))
    }

    // ---- Action Origins ----

    pub fn get_origin_localized_name(
        &mut self,
        origin: InputValueHandle,
        bits: EnumSet<InputString>,
    ) -> Result<String> {
        let mut name = vec![0u8; 100];
        let err = unsafe {
            self.inner.as_mut().GetOriginLocalizedName(
                origin.0,
                name.as_mut_ptr() as *mut i8,
                name.len() as u32 - 1, // TODO: is there *actually* an off-by-one here?
                bits.as_repr() as i32,
            )
        };

        EVRInputError::new(err)?;
        Ok(CString::from_vec_with_nul(name)
            .expect("There should be a null byte left!")
            // This shouldn't copy
            .into_string()
            // This path should only copy once, and only if there is invalid utf8. wish there was an `into_string_lossy`.
            .unwrap_or_else(|err| err.into_cstring().to_string_lossy().into_owned()))
    }

    pub fn get_origin_tracked_device_info(
        &mut self,
        origin: InputValueHandle,
    ) -> Result<OriginInfo> {
        let mut data: MaybeUninit<sys::InputOriginInfo_t> = MaybeUninit::uninit();
        let err = unsafe {
            self.inner.as_mut().GetOriginTrackedDeviceInfo(
                origin.0,
                data.as_mut_ptr(),
                std::mem::size_of::<sys::InputOriginInfo_t>() as u32,
            )
        };

        EVRInputError::new(err)?;
        Ok(OriginInfo(unsafe { data.assume_init() }))
    }
}
