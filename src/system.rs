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

mod private {
    pub trait Sealed {}
    // TODO: this may be overkill, but it encodes exactly how restrictive I want the sealing to be.
    pub trait SealedProperty<T> {}
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
impl TrackedDeviceProperty for CString {
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
// impl<'a> TrackedDeviceProperty for &'a CStr {
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

pub trait TrackedDevicePropertyName<Output: TrackedDeviceProperty>:
    private::SealedProperty<Output> + Into<sys::ETrackedDeviceProperty>
{
    fn get(self, index: TrackedDeviceIndex, system: &mut SystemManager) -> PropResult<Output> {
        Output::get(index, system, self.into())
    }
}

pub mod props {
    use crate::pose::Matrix3x4;
    use macro_rules_attribute::apply;

    use super::*;

    macro_rules! props_enum {
        (
            $( #[$meta:meta] )*
            $vis:vis enum $type:ident {
                $($variant:ident),+ $(,)?
            }
        ) => {
            ::paste::paste! {
                #[repr(i32)]
                $( #[$meta] )*
                $vis enum $type {
                    $($variant = crate::sys::ETrackedDeviceProperty::[<Prop_ $variant _ $type>] as i32,)+
                }
            }
        };
    }

    // first s/^\s+Prop_[a-zA-Z0-9_]+_(?!Bool)[a-zA-Z0-9]+ .*\n//\
    // remove Prop_ParentContainer
    // then s/Prop_([a-zA-Z0-9_]+)_Bool/$1/
    // TODO: update regex to match macro approach

    #[apply(props_enum!)]
    #[allow(non_camel_case_types)]
    pub enum Bool {
        WillDriftInYaw,
        DeviceIsWireless,
        DeviceIsCharging,
        Firmware_UpdateAvailable,
        Firmware_ManualUpdate,
        BlockServerShutdown,
        CanUnifyCoordinateSystemWithHmd,
        ContainsProximitySensor,
        DeviceProvidesBatteryStatus,
        DeviceCanPowerOff,
        HasCamera,
        Firmware_ForceUpdateRequired,
        ViveSystemButtonFixRequired,
        NeverTracked,
        Identifiable,
        Firmware_RemindUpdate,
        ReportsTimeSinceVSync,
        IsOnDesktop,
        DisplaySuppressed,
        DisplayAllowNightMode,
        DriverDirectModeSendsVsyncEvents,
        DisplayDebugMode,
        DoNotApplyPrediction,
        DriverIsDrawingControllers,
        DriverRequestsApplicationPause,
        DriverRequestsReducedRendering,
        ConfigurationIncludesLighthouse20Features,
        DriverProvidedChaperoneVisibility,
        CameraSupportsCompatibilityModes,
        SupportsRoomViewDepthProjection,
        DisplaySupportsMultipleFramerates,
        DisplaySupportsRuntimeFramerateChange,
        DisplaySupportsAnalogGain,
        Hmd_SupportsHDCP14LegacyCompat,
        Hmd_SupportsMicMonitoring,
        Audio_SupportsDualSpeakerAndJackOutput,
        CanWirelessIdentify,
        HasDisplayComponent,
        HasControllerComponent,
        HasCameraComponent,
        HasDriverDirectModeComponent,
        HasVirtualDisplayComponent,
        HasSpatialAnchorsSupport,
    }

    #[apply(props_enum!)]
    #[allow(non_camel_case_types)]
    pub enum Int32 {
        DeviceClass,
        NumCameras,
        CameraFrameLayout,
        CameraStreamFormat,
        EstimatedDeviceFirstUseTime,
        DisplayMCType,
        EdidVendorID,
        EdidProductID,
        DisplayGCType,
        CameraCompatibilityMode,
        DisplayMCImageWidth,
        DisplayMCImageHeight,
        DisplayMCImageNumChannels,
        ExpectedTrackingReferenceCount,
        ExpectedControllerCount,
        DistortionMeshResolution,
        HmdTrackingStyle,
        DriverRequestedMuraCorrectionMode,
        DriverRequestedMuraFeather_InnerLeft,
        DriverRequestedMuraFeather_InnerRight,
        DriverRequestedMuraFeather_InnerTop,
        DriverRequestedMuraFeather_InnerBottom,
        DriverRequestedMuraFeather_OuterLeft,
        DriverRequestedMuraFeather_OuterRight,
        DriverRequestedMuraFeather_OuterTop,
        DriverRequestedMuraFeather_OuterBottom,
        Axis0Type,
        Axis1Type,
        Axis2Type,
        Axis3Type,
        Axis4Type,
        ControllerRoleHint,
        Nonce,
        ControllerHandSelectionPriority,
    }

    #[apply(props_enum!)]
    #[allow(non_camel_case_types)]
    pub enum Uint64 {
        HardwareRevision,
        FirmwareVersion,
        FPGAVersion,
        VRCVersion,
        RadioVersion,
        DongleVersion,
        ParentDriver,
        BootloaderVersion,
        PeripheralApplicationVersion,
        CurrentUniverseId,
        PreviousUniverseId,
        DisplayFirmwareVersion,
        CameraFirmwareVersion,
        DisplayFPGAVersion,
        DisplayBootloaderVersion,
        DisplayHardwareVersion,
        AudioFirmwareVersion,
        GraphicsAdapterLuid,
        AudioBridgeFirmwareVersion,
        ImageBridgeFirmwareVersion,
        AdditionalRadioFeatures,
        SupportedButtons,
        OverrideContainer,
    }

    #[apply(props_enum!)]
    #[allow(non_camel_case_types)]
    pub enum Matrix34 {
        StatusDisplayTransform,
        CameraToHeadTransform,
        ImuToHeadTransform,
    }

    #[apply(props_enum!)]
    #[allow(non_camel_case_types)]
    pub enum String {
        TrackingSystemName,
        ModelNumber,
        SerialNumber,
        RenderModelName,
        ManufacturerName,
        TrackingFirmwareVersion,
        HardwareRevision,
        AllWirelessDongleDescriptions,
        ConnectedWirelessDongle,
        Firmware_ManualUpdateURL,
        Firmware_ProgrammingTarget,
        DriverVersion,
        ResourceRoot,
        RegisteredDeviceType,
        InputProfilePath,
        AdditionalDeviceSettingsPath,
        AdditionalSystemReportData,
        CompositeFirmwareVersion,
        ManufacturerSerialNumber,
        ComputedSerialNumber,
        DisplayMCImageLeft,
        DisplayMCImageRight,
        DisplayGCImage,
        CameraFirmwareDescription,
        DriverProvidedChaperonePath,
        NamedIconPathControllerLeftDeviceOff,
        NamedIconPathControllerRightDeviceOff,
        NamedIconPathTrackingReferenceDeviceOff,
        ExpectedControllerType,
        HmdColumnCorrectionSettingPrefix,
        Audio_DefaultPlaybackDeviceId,
        Audio_DefaultRecordingDeviceId,
        AttachedDeviceId,
        ModeLabel,
        IconPathName,
        NamedIconPathDeviceOff,
        NamedIconPathDeviceSearching,
        NamedIconPathDeviceSearchingAlert,
        NamedIconPathDeviceReady,
        NamedIconPathDeviceReadyAlert,
        NamedIconPathDeviceNotReady,
        NamedIconPathDeviceStandby,
        NamedIconPathDeviceAlertLow,
        NamedIconPathDeviceStandbyAlert,
        UserConfigPath,
        InstallPath,
        ControllerType,
    }

    // TODO: The following properties are not included here yet:
    //  - Prop_ParentContainer (no type mentioned, but is supposed to be opaque to us anyway)
    // The rest are arrays
    //  - Prop_CameraToHeadTransforms_Matrix34_Array
    //  - Prop_CameraWhiteBalance_Vector4_Array
    //  - Prop_CameraDistortionFunction_Int32_Array
    //  - Prop_CameraDistortionCoefficients_Float_Array
    //  - Prop_DisplayAvailableFrameRates_Float_Array

    macro_rules! impl_property {
        ($enum:ty; $($ty:ty),+) => {
            pub use $enum::*;
            impl From<$enum> for sys::ETrackedDeviceProperty {
                fn from(t: $enum) -> Self {
                    unsafe { std::mem::transmute(t) } // TODO: Into+FromPrimitive?
                }
            }
            $(
                impl private::SealedProperty<$ty> for $enum {}
                impl TrackedDevicePropertyName<$ty> for $enum {}
            )+
        }
    }

    impl_property!(Bool; bool);
    impl_property!(Int32; i32);
    impl_property!(Uint64; u64);
    impl_property!(Matrix34; Matrix3x4);
    impl_property!(self::String; CString);

    impl<T: private::Sealed> private::SealedProperty<T> for sys::ETrackedDeviceProperty {}
    impl<T: TrackedDeviceProperty> TrackedDevicePropertyName<T> for sys::ETrackedDeviceProperty {}
}

impl<'c> SystemManager<'c> {
    pub(super) fn new(_ctx: &'c Context) -> Self {
        let inner = unsafe { Pin::new_unchecked(sys::VRSystem().as_mut::<'c>().unwrap()) };
        Self {
            ctx: Default::default(),
            inner,
        }
    }

    pub fn get_tracked_device_property<
        T: TrackedDeviceProperty,
        N: TrackedDevicePropertyName<T>,
    >(
        &mut self,
        index: TrackedDeviceIndex,
        prop: N,
    ) -> PropResult<T> {
        N::get(prop, index, self)
    }
}
unsafe impl Send for SystemManager<'_> {}
unsafe impl Sync for SystemManager<'_> {}

#[cfg(test)]
mod test {
    use super::*;
    fn _compile_test(mut system: SystemManager) {
        let _bootloader_version = system
            .get_tracked_device_property(TrackedDeviceIndex::HMD, props::DisplayBootloaderVersion);
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
