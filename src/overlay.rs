pub use crate::errors::EVROverlayError;
use crate::pose::Matrix3x4;
use crate::pose::TrackingUniverseOrigin;
use crate::TextureBounds;
use crate::{sys, ColorTint, Context, TrackedDeviceIndex};

use derive_more::From;
use std::marker::PhantomData;
use std::pin::Pin;

pub struct OverlayManager<'c> {
	ctx: PhantomData<&'c Context>,
	inner: Pin<&'c mut sys::IVROverlay>,
}
impl<'c> OverlayManager<'c> {
	pub(super) fn new(_ctx: &'c Context) -> Self {
		let inner =
			unsafe { Pin::new_unchecked(sys::VROverlay().as_mut::<'c>().unwrap()) };
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

	pub fn set_visibility(
		&mut self,
		overlay: OverlayHandle,
		is_visible: bool,
	) -> Result<(), EVROverlayError> {
		let pinned = self.inner.as_mut();
		let err = if is_visible {
			unsafe { pinned.ShowOverlay(overlay.0) }
		} else {
			unsafe { pinned.HideOverlay(overlay.0) }
		};
		EVROverlayError::new(err)
	}

	pub fn is_visible(&mut self, overlay: OverlayHandle) -> bool {
		unsafe { self.inner.as_mut().IsOverlayVisible(overlay.0) }
	}

	/// Set the curvature of the overlay, with 0 being a quad and 1 being a cylinder.
	/// # Panics
	/// Panics if `curvature` is not in `[0,1]`
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

	pub fn curvature(
		&mut self,
		overlay: OverlayHandle,
	) -> Result<f32, EVROverlayError> {
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
	/// Panics if `alpha` is not in `[0,1]`
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

	pub fn tint(
		&mut self,
		overlay: OverlayHandle,
	) -> Result<ColorTint, EVROverlayError> {
		let mut tint = ColorTint::default();
		unsafe {
			let err = self.inner.as_mut().GetOverlayColor(
				overlay.0,
				&mut tint.r,
				&mut tint.g,
				&mut tint.b,
			);
			EVROverlayError::new(err)?;
			let err = self.inner.as_mut().GetOverlayAlpha(overlay.0, &mut tint.a);
			EVROverlayError::new(err)?
		};
		Ok(tint)
	}

	pub fn set_tint(
		&mut self,
		overlay: OverlayHandle,
		tint: ColorTint,
	) -> Result<(), EVROverlayError> {
		unsafe {
			let err = self
				.inner
				.as_mut()
				.SetOverlayColor(overlay.0, tint.r, tint.g, tint.b);
			EVROverlayError::new(err)?;
			let err = self.inner.as_mut().SetOverlayAlpha(overlay.0, tint.a);
			EVROverlayError::new(err)?;
		}
		Ok(())
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
		data: &[u8],
		width: usize,
		height: usize,
		bytes_per_pixel: usize,
	) -> Result<(), EVROverlayError> {
		let err = unsafe {
			let ptr: *const std::ffi::c_void = data.as_ptr().cast();
			// I think there is a typo in the API, and it actually needs a const
			// ptr. *IF* this is true, the following line is safe.
			let ptr = ptr as *mut std::ffi::c_void;

			self.inner.as_mut().SetOverlayRaw(
				overlay.0,
				ptr.cast(),
				width as u32,
				height as u32,
				bytes_per_pixel as u32,
			)
		};
		EVROverlayError::new(err)
	}

	/// Get aspect ratio, with aspect expressed as width / height.
	pub fn texel_aspect(
		&mut self,
		overlay: OverlayHandle,
	) -> Result<f32, EVROverlayError> {
		let mut aspect = 0.0;
		let err = unsafe {
			self.inner
				.as_mut()
				.GetOverlayTexelAspect(overlay.0, &mut aspect)
		};
		EVROverlayError::new(err)?;
		Ok(aspect)
	}

	/// Set aspect ratio, with aspect expressed as width / height.
	///
	/// Note that too extreme of an aspect ratio will cause an error to be returned.
	pub fn set_texel_aspect(
		&mut self,
		overlay: OverlayHandle,
		aspect: f32,
	) -> Result<(), EVROverlayError> {
		let err =
			unsafe { self.inner.as_mut().SetOverlayTexelAspect(overlay.0, aspect) };
		EVROverlayError::new(err)
	}

	/// Sets an absolute transform for this overlay.
	///
	/// Wraps c++ `SetOverlayTransformAbsolute`.
	pub fn set_transform_absolute(
		&mut self,
		overlay: OverlayHandle,
		origin: TrackingUniverseOrigin,
		origin_to_overlay: &Matrix3x4,
	) -> Result<(), EVROverlayError> {
		let origin_to_overlay: &sys::HmdMatrix34_t = origin_to_overlay.into();
		let err = unsafe {
			self.inner.as_mut().SetOverlayTransformAbsolute(
				overlay.0,
				origin,
				origin_to_overlay,
			)
		};
		EVROverlayError::new(err)
	}

	/// Gets the absolute transform for this overlay.
	///
	/// Wraps c++ `GetOverlayTransformAbsolute`.
	pub fn get_transform_absolute(
		&mut self,
		overlay: OverlayHandle,
		origin_to_overlay: &mut Matrix3x4,
	) -> Result<TrackingUniverseOrigin, EVROverlayError> {
		// Some random value just to initialize the data
		let mut origin = TrackingUniverseOrigin::TrackingUniverseStanding;
		let origin_to_overlay: &mut sys::HmdMatrix34_t = origin_to_overlay.into();
		let err = unsafe {
			self.inner.as_mut().GetOverlayTransformAbsolute(
				overlay.0,
				&mut origin,
				origin_to_overlay,
			)
		};
		EVROverlayError::new(err).map(|_| origin)
	}

	/// Sets the transform for this overlay, relative to a tracked device.
	///
	/// Wraps c++ `SetOverlayTransformTrackedDeviceRelative`.
	pub fn set_transform_tracked_device_relative(
		&mut self,
		overlay: OverlayHandle,
		index: TrackedDeviceIndex,
		device_to_overlay: &Matrix3x4,
	) -> Result<(), EVROverlayError> {
		let device_to_overlay: &sys::HmdMatrix34_t = device_to_overlay.into();
		let err = unsafe {
			self.inner
				.as_mut()
				.SetOverlayTransformTrackedDeviceRelative(
					overlay.0,
					index.0,
					device_to_overlay,
				)
		};
		EVROverlayError::new(err)
	}

	/// Gets the transform for this overlay, relative to a tracked device.
	///
	/// Wraps c++ `GetOverlayTransformTrackedDeviceRelative`.
	pub fn get_transform_tracked_device_relative(
		&mut self,
		overlay: OverlayHandle,
		device_to_overlay: &mut Matrix3x4,
	) -> Result<TrackedDeviceIndex, EVROverlayError> {
		let mut index = sys::TrackedDeviceIndex_t::default();
		let device_to_overlay: &mut sys::HmdMatrix34_t = device_to_overlay.into();
		let err = unsafe {
			self.inner
				.as_mut()
				.GetOverlayTransformTrackedDeviceRelative(
					overlay.0,
					&mut index,
					device_to_overlay,
				)
		};
		EVROverlayError::new(err)?;
		// TODO: is the error ever really going to be delayed to here? (Can we successfully return an invalid handle?)
		TrackedDeviceIndex::new(index).or_else(|_| {
			EVROverlayError::new(sys::EVROverlayError::VROverlayError_RequestFailed)
				.map(|_| unreachable!())
		})
	}

	/// Sets the transform for this overlay, relative to another overlay.
	///
	/// Wraps c++ `SetOverlayTransformOverlayRelative`.
	pub fn set_transform_overlay_relatve(
		&mut self,
		child_overlay: OverlayHandle,
		parent_overlay: OverlayHandle,
		parent_to_child: &Matrix3x4,
	) -> Result<(), EVROverlayError> {
		let parent_to_child: &sys::HmdMatrix34_t = parent_to_child.into();
		let err = unsafe {
			self.inner.as_mut().SetOverlayTransformOverlayRelative(
				child_overlay.0,
				parent_overlay.0,
				parent_to_child,
			)
		};
		EVROverlayError::new(err)
	}

	/// Gets the transform for this overlay, relative to another overlay.
	///
	/// Wraps c++ `GetOverlayTransformOverlayRelative`.
	pub fn get_transform_overlay_relative(
		&mut self,
		child_overlay: OverlayHandle,
		parent_to_child: &mut Matrix3x4,
	) -> Result<OverlayHandle, EVROverlayError> {
		let mut parent_overlay = sys::VROverlayHandle_t::default();
		let parent_to_child: &mut sys::HmdMatrix34_t = parent_to_child.into();
		let err = unsafe {
			self.inner.as_mut().GetOverlayTransformOverlayRelative(
				child_overlay.0,
				&mut parent_overlay,
				parent_to_child,
			)
		};
		EVROverlayError::new(err).map(|_| parent_overlay.into())
	}

	pub fn set_texture_bounds(
		&mut self,
		overlay: OverlayHandle,
		bounds: &TextureBounds,
	) -> Result<(), EVROverlayError> {
		let err = unsafe {
			self.inner
				.as_mut()
				.SetOverlayTextureBounds(overlay.0, &bounds.0)
		};
		EVROverlayError::new(err)
	}
}
unsafe impl Send for OverlayManager<'_> {}
unsafe impl Sync for OverlayManager<'_> {}

#[derive(From, Debug, PartialEq, Eq, Clone, Copy)]
pub struct OverlayHandle(pub sys::VROverlayHandle_t);
