pub use crate::errors::EVROverlayError;
use crate::{sys, Context};

pub struct IOverlay<'c> {
    ctx: &'c Context,
    inner: &'c sys::IVROverlay,
}
impl<'c> IOverlay<'c> {
    pub(super) fn new(ctx: &'c Context) -> Self {
        let inner = unsafe { sys::VROverlay().as_ref::<'c>() }.unwrap();
        Self { ctx, inner }
    }

    pub fn create_overlay(&self, key: &str, friendly_name: &str) -> OverlayHandle {
        // let err = self.inner.CreateOverlay(key.as_ptr(), friendly_name.as_ptr(), VROverlayHandle_t * pOverlayHandle )
        todo!()
    }
}

pub struct OverlayHandle();
