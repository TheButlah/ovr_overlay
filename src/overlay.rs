pub use crate::errors::EVROverlayError;
use crate::{sys, Context};

use derive_more::From;
use std::mem::MaybeUninit;
use std::pin::Pin;

pub struct IOverlay<'c> {
    ctx: &'c Context,
    inner: Pin<&'c mut sys::IVROverlay>,
}
impl<'c> IOverlay<'c> {
    pub(super) fn new(ctx: &'c Context) -> Self {
        let inner = unsafe { Pin::new_unchecked(sys::VROverlay().as_mut::<'c>().unwrap()) };
        Self { ctx, inner }
    }

    pub fn create_overlay(
        &mut self,
        key: &str,
        friendly_name: &str,
    ) -> Result<OverlayHandle, EVROverlayError> {
        let mut handle = MaybeUninit::<sys::VROverlayHandle_t>::uninit();
        let err = unsafe {
            self.inner.as_mut().CreateOverlay(
                key.as_ptr().cast(),
                friendly_name.as_ptr().cast(),
                handle.as_mut_ptr(),
            )
        };

        if let Some(err) = EVROverlayError::new(err) {
            return Err(err);
        }
        Ok(OverlayHandle(unsafe { handle.assume_init() }))
    }
}

#[derive(From)]
pub struct OverlayHandle(pub sys::VROverlayHandle_t);
