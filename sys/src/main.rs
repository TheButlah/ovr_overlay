use ovr_overlay_sys as sys;

fn main() {
    println!("{:?}", sys::EVRInitError::VRInitError_None as u8); // prints
}
