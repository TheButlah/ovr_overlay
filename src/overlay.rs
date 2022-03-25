pub use crate::errors::EVROverlayError;
use crate::{sys, ColorTint, Context};

use derive_more::From;
use std::marker::PhantomData;
use std::pin::Pin;

pub struct IOverlay<'c> {
    ctx: PhantomData<&'c Context>,
    inner: Pin<&'c mut sys::IVROverlay>,
}
impl<'c> IOverlay<'c> {
    pub(super) fn new(_ctx: &'c Context) -> Self {
        let inner = unsafe { Pin::new_unchecked(sys::VROverlay().as_mut::<'c>().unwrap()) };
        Self {
            ctx: Default::default(),
            inner,
        }
    }

    pub fn create_overlay(
        &mut self,
        key: &str,
        friendly_name: &str,
    ) -> Result<OverlayHandle, EVROverlayError> {
        let mut handle = sys::VROverlayHandle_t::default();
        let err = unsafe {
            self.inner.as_mut().CreateOverlay(
                key.as_ptr().cast(),
                friendly_name.as_ptr().cast(),
                &mut handle,
            )
        };

        EVROverlayError::new(err)?;
        Ok(OverlayHandle(handle))
    }

    pub fn show_overlay(&mut self, overlay: OverlayHandle) -> Result<(), EVROverlayError> {
        let err = unsafe { self.inner.as_mut().ShowOverlay(overlay.0) };
        EVROverlayError::new(err)
    }

    pub fn hide_overlay(&mut self, overlay: OverlayHandle) -> Result<(), EVROverlayError> {
        let err = unsafe { self.inner.as_mut().HideOverlay(overlay.0) };
        EVROverlayError::new(err)
    }

    pub fn is_visible(&mut self, overlay: OverlayHandle) -> bool {
        unsafe { self.inner.as_mut().IsOverlayVisible(overlay.0) }
    }

    /// Set the curvature of the overlay, with 0 being a quad and 1 being a cylinder.
    /// # Panics
    /// Panics if `curvature` is not in [0,1]
    pub fn set_curvature(
        &mut self,
        overlay: OverlayHandle,
        curvature: f32,
    ) -> Result<(), EVROverlayError> {
        // if !(0.0..=1.0).contains(&curvature) {
        //     panic!("`curvature` must lie in range [0,1]")
        // }
        let err = unsafe {
            self.inner
                .as_mut()
                .SetOverlayCurvature(overlay.0, curvature)
        };
        EVROverlayError::new(err)
    }

    pub fn curvature(&mut self, overlay: OverlayHandle) -> Result<f32, EVROverlayError> {
        let mut curvature = 0.0;
        let err = unsafe {
            self.inner
                .as_mut()
                .GetOverlayCurvature(overlay.0, &mut curvature)
        };
        EVROverlayError::new(err)?;
        Ok(curvature)
    }

    /// Sets the opacity of the overlay. `alpha` ranges from 0.0 (transparent) to 1.0 (opaque).
    /// # Panics
    /// Panics if `alpha` is not in [0,1]
    pub fn set_opacity(
        &mut self,
        overlay: OverlayHandle,
        alpha: f32,
    ) -> Result<(), EVROverlayError> {
        if !(0.0..=1.0).contains(&alpha) {
            panic!("`alpha` must be in range [0,1]");
        }
        let err = unsafe { self.inner.as_mut().SetOverlayAlpha(overlay.0, alpha) };
        EVROverlayError::new(err)
    }

    pub fn opacity(&mut self, overlay: OverlayHandle) -> Result<f32, EVROverlayError> {
        let mut alpha = 0.0;
        let err = unsafe { self.inner.as_mut().GetOverlayAlpha(overlay.0, &mut alpha) };
        EVROverlayError::new(err)?;
        Ok(alpha)
    }

    pub fn width(&mut self, overlay: OverlayHandle) -> Result<f32, EVROverlayError> {
        let mut width = 0.0;
        let err = unsafe {
            self.inner
                .as_mut()
                .GetOverlayWidthInMeters(overlay.0, &mut width)
        };
        EVROverlayError::new(err)?;
        Ok(width)
    }

    pub fn set_width(
        &mut self,
        overlay: OverlayHandle,
        width_in_meters: f32,
    ) -> Result<(), EVROverlayError> {
        let err = unsafe {
            self.inner
                .as_mut()
                .SetOverlayWidthInMeters(overlay.0, width_in_meters)
        };
        EVROverlayError::new(err)
    }

    pub fn tint(&mut self, overlay: OverlayHandle) -> Result<ColorTint, EVROverlayError> {
        let mut tint = ColorTint::default();
        let err = unsafe {
            self.inner
                .as_mut()
                .GetOverlayColor(overlay.0, &mut tint.r, &mut tint.g, &mut tint.b)
        };
        EVROverlayError::new(err)?;
        Ok(tint)
    }

    pub fn set_tint(
        &mut self,
        overlay: OverlayHandle,
        tint: ColorTint,
    ) -> Result<(), EVROverlayError> {
        let err = unsafe {
            self.inner
                .as_mut()
                .SetOverlayColor(overlay.0, tint.r, tint.g, tint.b)
        };
        EVROverlayError::new(err)
    }

    pub fn set_image(
        &mut self,
        overlay: OverlayHandle,
        img_path: &std::ffi::CStr,
    ) -> Result<(), EVROverlayError> {
        let err = unsafe {
            self.inner
                .as_mut()
                .SetOverlayFromFile(overlay.0, img_path.as_ptr())
        };
        EVROverlayError::new(err)
    }

    pub fn set_raw_data(
        &mut self,
        overlay: OverlayHandle,
        data: Vec<u8>,
        width: usize,
        height: usize,
        bytes_per_pixel: usize,
    ) -> Result<(), EVROverlayError> {
        // TODO: Don't leak this in the future, instead deallocate with rust drop
        let data = data.leak();
        let err = unsafe {
            self.inner.as_mut().SetOverlayRaw(
                overlay.0,
                data.as_mut_ptr().cast(),
                width as u32,
                height as u32,
                bytes_per_pixel as u32,
            )
        };
        EVROverlayError::new(err)
    }
}

#[derive(From, Debug, PartialEq, Eq, Clone, Copy)]
pub struct OverlayHandle(pub sys::VROverlayHandle_t);
