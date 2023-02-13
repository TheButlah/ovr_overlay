use crate::errors::ETrackedPropertyError;
use crate::{sys, Context, TrackedDeviceIndex};

use std::ffi::CString;
use std::marker::PhantomData;
use std::pin::Pin;
use std::ptr::null_mut;

pub struct SystemManager<'c> {
    ctx: PhantomData<&'c Context>,
    inner: Pin<&'c mut sys::IVRSystem>,
}

mod sealed {
    pub trait SealedPropertyType {}
}

use sealed::*;

type PropResult<T> = std::result::Result<T, ETrackedPropertyError>;

pub trait PropertyType: SealedPropertyType + Sized {
    fn get(
        index: TrackedDeviceIndex,
        system: &mut SystemManager,
        prop: sys::ETrackedDeviceProperty,
    ) -> PropResult<Self>;
}

macro_rules! impl_property_type {
    ($ty:ty, $method:ident) => {
        impl SealedPropertyType for $ty {}
        impl PropertyType for $ty {
            fn get(
                index: TrackedDeviceIndex,
                system: &mut SystemManager,
                prop: sys::ETrackedDeviceProperty,
            ) -> PropResult<Self> {
                let mut err = sys::ETrackedPropertyError::TrackedProp_Success;
                let res = unsafe { system.inner.as_mut().$method(index.0, prop, &mut err) };
                ETrackedPropertyError::new(err)?;
                Ok(res)
            }
        }
    };
}

impl_property_type!(bool, GetBoolTrackedDeviceProperty);
impl_property_type!(f32, GetFloatTrackedDeviceProperty);
impl_property_type!(i32, GetInt32TrackedDeviceProperty);
impl_property_type!(u64, GetUint64TrackedDeviceProperty);

// thought: other matrix types?
impl sealed::SealedPropertyType for crate::pose::Matrix3x4 {}
impl PropertyType for crate::pose::Matrix3x4 {
    fn get(
        index: TrackedDeviceIndex,
        system: &mut SystemManager,
        prop: sys::ETrackedDeviceProperty,
    ) -> PropResult<Self> {
        let mut err = sys::ETrackedPropertyError::TrackedProp_Success;
        let res = unsafe {
            system
                .inner
                .as_mut()
                .GetMatrix34TrackedDeviceProperty(index.0, prop, &mut err)
        };
        ETrackedPropertyError::new(err)?;
        Ok(res.into())
    }
}

impl sealed::SealedPropertyType for CString {}
impl PropertyType for CString {
    fn get(
        index: TrackedDeviceIndex,
        system: &mut SystemManager,
        prop: sys::ETrackedDeviceProperty,
    ) -> PropResult<Self> {
        let mut err = sys::ETrackedPropertyError::TrackedProp_Success;
        let len = unsafe {
            system.inner.as_mut().GetStringTrackedDeviceProperty(
                index.0,
                prop,
                null_mut(),
                0,
                &mut err,
            )
        };
        ETrackedPropertyError::new(err)?;
        let mut data = vec![0; len as usize];
        let _len = unsafe {
            system.inner.as_mut().GetStringTrackedDeviceProperty(
                index.0,
                prop,
                data.as_mut_ptr() as *mut i8,
                len,
                &mut err,
            )
        };
        ETrackedPropertyError::new(err)?;

        Ok(CString::from_vec_with_nul(data).expect("missing nul byte from openvr!"))
    }
}

impl sealed::SealedPropertyType for String {}
impl PropertyType for String {
    fn get(
        index: TrackedDeviceIndex,
        system: &mut SystemManager,
        prop: sys::ETrackedDeviceProperty,
    ) -> PropResult<Self> {
        // might want to make a helper function for this concept
        // or be fancy like this: <https://www.reddit.com/r/rust/comments/7n1oz2/comment/drzqn9d/?utm_source=share&utm_medium=web2x&context=3>
        CString::get(index, system, prop).map(|s| {
            s.into_string()
                .unwrap_or_else(|s| s.into_cstring().to_string_lossy().into_owned())
        })
    }
}

// TODO: arrays. I don't feel like dealing with them right now.

impl<'c> SystemManager<'c> {
    pub(super) fn new(_ctx: &'c Context) -> Self {
        let inner = unsafe { Pin::new_unchecked(sys::VRSystem().as_mut::<'c>().unwrap()) };
        Self {
            ctx: Default::default(),
            inner,
        }
    }

    pub fn get_property_sys<T: PropertyType>(
        &mut self,
        index: TrackedDeviceIndex,
        prop: sys::ETrackedDeviceProperty,
    ) -> PropResult<T> {
        T::get(index, self, prop)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    fn _compile_test(mut system: SystemManager) {
        // let _bootloader_version =
        //     system.get_property(TrackedDeviceIndex::HMD, props::DisplayBootloaderVersion);
        let _display_version: u64 = system
            .get_property_sys(
                TrackedDeviceIndex::HMD,
                sys::ETrackedDeviceProperty::Prop_DisplayHardwareVersion_Uint64,
            )
            .unwrap();
        // let _gc_image_string: String = system
        //     .get_property(TrackedDeviceIndex::HMD, props::DisplayGCImage)
        //     .unwrap();
        // let _gc_image_cstring: CString = system
        //     .get_property(TrackedDeviceIndex::HMD, props::DisplayGCImage)
        //     .unwrap();
    }
}
