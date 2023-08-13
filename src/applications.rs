use std::ffi::{CStr, CString};
use std::marker::PhantomData;
use std::path::Path;
use std::pin::Pin;

use crate::{errors::EVRApplicationError, sys, Context};

pub struct ApplicationsManager<'c> {
	ctx: PhantomData<&'c Context>,
	inner: Pin<&'c mut sys::IVRApplications>,
}

type Result<T> = std::result::Result<T, EVRApplicationError>;

impl<'c> ApplicationsManager<'c> {
	pub(super) fn new(_ctx: &'c Context) -> Self {
		let inner = unsafe {
			Pin::new_unchecked(sys::VRApplications().as_mut::<'c>().unwrap())
		};
		Self {
			ctx: Default::default(),
			inner,
		}
	}

	// ---- Handle Management ----

	pub fn add_application_manifest(
		&mut self,
		path: &Path,
		temporary: bool,
	) -> Result<()> {
		let path = if let Ok(s) = CString::new(path.to_string_lossy().as_bytes()) {
			s
		} else {
			return EVRApplicationError::new(
				sys::EVRApplicationError::VRApplicationError_InvalidParameter,
			);
		};
		self.add_application_manifest_raw(&path, temporary)
	}

	pub fn add_application_manifest_raw(
		&mut self,
		path: &CStr,
		temporary: bool,
	) -> Result<()> {
		let err = unsafe {
			self.inner
				.as_mut()
				.AddApplicationManifest(path.as_ptr(), temporary)
		};
		EVRApplicationError::new(err)
	}

	pub fn remove_application_manifest(&mut self, path: &Path) -> Result<()> {
		let path = if let Ok(s) = CString::new(path.to_string_lossy().as_bytes()) {
			s
		} else {
			return EVRApplicationError::new(
				sys::EVRApplicationError::VRApplicationError_InvalidParameter,
			);
		};
		self.remove_application_manifest_raw(&path)
	}

	pub fn remove_application_manifest_raw(&mut self, path: &CStr) -> Result<()> {
		let err =
			unsafe { self.inner.as_mut().RemoveApplicationManifest(path.as_ptr()) };
		EVRApplicationError::new(err)
	}

	pub fn is_application_installed(&mut self, key: &str) -> Result<bool> {
		let name = if let Ok(s) = CString::new(key) {
			s
		} else {
			return EVRApplicationError::new(
				sys::EVRApplicationError::VRApplicationError_InvalidParameter,
			)
			.map(|_| unreachable!());
		};

		self.is_application_installed_raw(&name)
	}

	pub fn is_application_installed_raw(&mut self, key: &CStr) -> Result<bool> {
		let installed =
			unsafe { self.inner.as_mut().IsApplicationInstalled(key.as_ptr()) };

		Ok(installed)
	}
}
