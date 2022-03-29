use crate::sys;

/// Row-major 3x4 matrix
#[repr(C)]
pub struct Matrix3x4(pub [[f32; 4]; 3]);
impl From<&Matrix3x4> for &sys::HmdMatrix34_t {
    fn from(other: &Matrix3x4) -> Self {
        let other = other as *const Matrix3x4;
        // safety: C++ POD types have same memory layout as order of fields
        //   so it is safe to cast it this way
        unsafe { &*other.cast() }
    }
}
impl From<&sys::HmdMatrix34_t> for &Matrix3x4 {
    fn from(other: &sys::HmdMatrix34_t) -> Self {
        let other = other as *const sys::HmdMatrix34_t;
        // safety: C++ POD types have same memory layout as order of fields
        //   so it is safe to cast it this way
        unsafe { &*other.cast() }
    }
}
impl From<sys::HmdMatrix34_t> for Matrix3x4 {
    fn from(other: sys::HmdMatrix34_t) -> Self {
        // Get shrekt, autocxx 😎
        unsafe { std::mem::transmute(other) }
    }
}
impl From<Matrix3x4> for sys::HmdMatrix34_t {
    fn from(other: Matrix3x4) -> Self {
        unsafe { std::mem::transmute(other) }
    }
}

#[cfg(feature = "nalgebra")]
impl From<&Matrix3x4> for nalgebra::Matrix3x4<f32> {
    fn from(other: &Matrix3x4) -> Self {
        use slice_of_array::SliceFlatExt;
        Self::from_row_slice(other.0.flat())
    }
}

#[cfg(feature = "nalgebra")]
impl<RStride, CStride> From<nalgebra::MatrixSlice3x4<'_, f32, RStride, CStride>> for Matrix3x4
where
    RStride: nalgebra::base::dimension::Dim,
    CStride: nalgebra::base::dimension::Dim,
{
    fn from(other: nalgebra::MatrixSlice3x4<'_, f32, RStride, CStride>) -> Self {
        Self(other.transpose().data.0)
    }
}

pub use sys::ETrackingUniverseOrigin as TrackingUniverseOrigin;
