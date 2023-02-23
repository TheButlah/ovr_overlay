use crate::errors::ETrackedPropertyError;
use crate::{sys, Context, TrackedDeviceIndex};

use std::ffi::{CString, CStr};
use std::marker::PhantomData;
use std::pin::Pin;
use std::ptr::null_mut;

pub struct SystemManager<'c> {
    ctx: PhantomData<&'c Context>,
    inner: Pin<&'c mut sys::IVRSystem>,
    string_buf: Vec<u8>,
}

mod private {
    pub trait Sealed {}
}


type PropResult<T> = Result<T, ETrackedPropertyError>;

/// Trait implemented by types that represent storage types of properties.
pub trait TrackedDeviceProperty: private::Sealed + Sized {
    fn get(
        index: TrackedDeviceIndex,
        system: &mut SystemManager,
        prop: sys::ETrackedDeviceProperty,
    ) -> PropResult<Self>;
}


macro_rules! impl_property_type {
    ($ty:ty, $method:ident) => {
        impl private::Sealed for $ty {}
        impl TrackedDeviceProperty for $ty {
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

// TODO: Decide if we want to support matrix types from other libraries, like nalgebra
impl private::Sealed for crate::pose::Matrix3x4 {}
impl TrackedDeviceProperty for crate::pose::Matrix3x4 {
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

impl private::Sealed for CString {}
impl TrackedDevicePropertyValue for CString {
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

// This would probably be easer if it were a method on SystemManager,
//  as-is this implementation doesn't match the trait definition.
// impl private::Sealed for &CStr {}
// impl<'a> TrackedDevicePropertyValue for &'a CStr {
//     fn get(
//         index: TrackedDeviceIndex,
//         system: &'a mut SystemManager,
//         prop: sys::ETrackedDeviceProperty,
//     ) -> PropResult<&'a CStr> {
//         let mut err = sys::ETrackedPropertyError::TrackedProp_Success;
//         let len = unsafe {
//             system.inner.as_mut().GetStringTrackedDeviceProperty(
//                 index.0,
//                 prop,
//                 system.string_buf.as_mut_ptr() as *mut i8,
//                 sys::k_unMaxPropertyStringSize,
//                 &mut err,
//             )
//         };
//         ETrackedPropertyError::new(err)?;

//         Ok(CStr::from_bytes_with_nul(&system.string_buf[..len as usize]).unwrap())
//     }
// }

// TODO: arrays. I don't feel like dealing with them right now.

impl<'c> SystemManager<'c> {
    pub(super) fn new(_ctx: &'c Context) -> Self {
        let inner = unsafe { Pin::new_unchecked(sys::VRSystem().as_mut::<'c>().unwrap()) };
        Self {
            ctx: Default::default(),
            inner,
            string_buf: vec![0u8; sys::k_unMaxPropertyStringSize as usize],
        }
    }

    pub fn get_tracked_device_property<T: TrackedDevicePropertyValue>(
        &mut self,
        index: TrackedDeviceIndex,
        prop: sys::ETrackedDeviceProperty,
    ) -> PropResult<T> {
        T::get(index, self, prop)
    }
}
unsafe impl Send for SystemManager<'_> {}
unsafe impl Sync for SystemManager<'_> {}

#[cfg(test)]
mod test {
    use super::*;
    fn _compile_test(mut system: SystemManager) {
        // let _bootloader_version =
        //     system.get_tracked_device_property(TrackedDeviceIndex::HMD, props::DisplayBootloaderVersion);
        let _display_version: u64 = system
            .get_tracked_device_property(
                TrackedDeviceIndex::HMD,
                sys::ETrackedDeviceProperty::Prop_DisplayHardwareVersion_Uint64,
            )
            .unwrap();
        // let _gc_image_cstring = system
        //     .get_tracked_device_property(TrackedDeviceIndex::HMD, props::DisplayGCImage)
        //     .unwrap();
    }
}
