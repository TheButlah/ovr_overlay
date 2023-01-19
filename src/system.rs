use crate::errors::ETrackedPropertyError;
use crate::{sys, Context};

use std::ffi::CString;
use std::marker::PhantomData;
use std::pin::Pin;
use std::ptr::{null, null_mut};

pub struct SystemManager<'c> {
    ctx: PhantomData<&'c Context>,
    inner: Pin<&'c mut sys::IVRSystem>,
}

mod sealed {
    pub trait SealedPropertyType {}
    pub trait SealedProperty<T: SealedPropertyType> {}
}

use sealed::*;

type PropResult<T> = std::result::Result<T, ETrackedPropertyError>;

pub trait PropertyType: SealedPropertyType + Sized {
    fn get(
        index: sys::TrackedDeviceIndex_t,
        system: &mut SystemManager,
        prop: sys::ETrackedDeviceProperty,
    ) -> PropResult<Self>;
}

macro_rules! impl_property_type {
    ($ty:ty, $method:ident) => {
        impl SealedPropertyType for $ty {}
        impl PropertyType for $ty {
            fn get(
                index: sys::TrackedDeviceIndex_t,
                system: &mut SystemManager,
                prop: sys::ETrackedDeviceProperty,
            ) -> PropResult<Self> {
                let mut err = sys::ETrackedPropertyError::TrackedProp_Success;
                let res = unsafe { system.inner.as_mut().$method(index, prop, &mut err) };
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
        index: sys::TrackedDeviceIndex_t,
        system: &mut SystemManager,
        prop: sys::ETrackedDeviceProperty,
    ) -> PropResult<Self> {
        let mut err = sys::ETrackedPropertyError::TrackedProp_Success;
        let res = unsafe {
            system
                .inner
                .as_mut()
                .GetMatrix34TrackedDeviceProperty(index, prop, &mut err)
        };
        ETrackedPropertyError::new(err)?;
        Ok(res.into())
    }
}

impl sealed::SealedPropertyType for CString {}
impl PropertyType for CString {
    fn get(
        index: sys::TrackedDeviceIndex_t,
        system: &mut SystemManager,
        prop: sys::ETrackedDeviceProperty,
    ) -> PropResult<Self> {
        let mut err = sys::ETrackedPropertyError::TrackedProp_Success;
        let len = unsafe {
            system.inner.as_mut().GetStringTrackedDeviceProperty(
                index,
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
                index,
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
        index: sys::TrackedDeviceIndex_t,
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

// TODO: array and string. I don't feel like dealing with them right now.

pub trait Property<Output: PropertyType>: SealedProperty<Output> + Into<sys::ETrackedDeviceProperty> {
    fn get(
        self,
        index: sys::TrackedDeviceIndex_t,
        system: &mut SystemManager,
    ) -> PropResult<Output> {
        Output::get(index, system, self.into())
    }
}

mod props {
    use crate::pose::Matrix3x4;

    use super::*;

    // first s/^\s+Prop_[a-zA-Z0-9_]+_(?!Int32)[a-zA-Z0-9]+ .*\n//
    // then s/Prop_([a-zA-Z0-9_]+)_Bool/$1/
    #[repr(i32)]
    pub enum Bool {
        WillDriftInYaw = 1004,
        DeviceIsWireless = 1010,
        DeviceIsCharging = 1011,
        Firmware_UpdateAvailable = 1014,
        Firmware_ManualUpdate = 1015,
        BlockServerShutdown = 1023,
        CanUnifyCoordinateSystemWithHmd = 1024,
        ContainsProximitySensor = 1025,
        DeviceProvidesBatteryStatus = 1026,
        DeviceCanPowerOff = 1027,
        HasCamera = 1030,
        Firmware_ForceUpdateRequired = 1032,
        ViveSystemButtonFixRequired = 1033,
        NeverTracked = 1038,
        Identifiable = 1043,
        Firmware_RemindUpdate = 1047,
        ReportsTimeSinceVSync = 2000,
        IsOnDesktop = 2007,
        DisplaySuppressed = 2036,
        DisplayAllowNightMode = 2037,
        DriverDirectModeSendsVsyncEvents = 2043,
        DisplayDebugMode = 2044,
        DoNotApplyPrediction = 2054,
        DriverIsDrawingControllers = 2057,
        DriverRequestsApplicationPause = 2058,
        DriverRequestsReducedRendering = 2059,
        ConfigurationIncludesLighthouse20Features = 2069,
        DriverProvidedChaperoneVisibility = 2076,
        CameraSupportsCompatibilityModes = 2078,
        SupportsRoomViewDepthProjection = 2079,
        DisplaySupportsMultipleFramerates = 2081,
        DisplaySupportsRuntimeFramerateChange = 2084,
        DisplaySupportsAnalogGain = 2085,
        Hmd_SupportsHDCP14LegacyCompat = 2102,
        Hmd_SupportsMicMonitoring = 2103,
        Audio_SupportsDualSpeakerAndJackOutput = 2303,
        CanWirelessIdentify = 4007,
        //Prop_ParentContainer = 5151,
        HasDisplayComponent = 6002,
        HasControllerComponent = 6003,
        HasCameraComponent = 6004,
        HasDriverDirectModeComponent = 6005,
        HasVirtualDisplayComponent = 6006,
        HasSpatialAnchorsSupport = 6007,
    }

    #[repr(i32)]
    pub enum I32 {
        DeviceClass = 1029,
        NumCameras = 1039,
        CameraFrameLayout = 1040,
        CameraStreamFormat = 1041,
        EstimatedDeviceFirstUseTime = 1051,
        DisplayMCType = 2008,
        EdidVendorID = 2011,
        EdidProductID = 2015,
        DisplayGCType = 2017,
        CameraCompatibilityMode = 2033,
        DisplayMCImageWidth = 2038,
        DisplayMCImageHeight = 2039,
        DisplayMCImageNumChannels = 2040,
        ExpectedTrackingReferenceCount = 2049,
        ExpectedControllerCount = 2050,
        DistortionMeshResolution = 2056,
        HmdTrackingStyle = 2075,
        DriverRequestedMuraCorrectionMode = 2200,
        DriverRequestedMuraFeather_InnerLeft = 2201,
        DriverRequestedMuraFeather_InnerRight = 2202,
        DriverRequestedMuraFeather_InnerTop = 2203,
        DriverRequestedMuraFeather_InnerBottom = 2204,
        DriverRequestedMuraFeather_OuterLeft = 2205,
        DriverRequestedMuraFeather_OuterRight = 2206,
        DriverRequestedMuraFeather_OuterTop = 2207,
        DriverRequestedMuraFeather_OuterBottom = 2208,
        Axis0Type = 3002,
        Axis1Type = 3003,
        Axis2Type = 3004,
        Axis3Type = 3005,
        Axis4Type = 3006,
        ControllerRoleHint = 3007,
        Nonce = 4008,
        //Prop_ParentContainer = 5151,
        ControllerHandSelectionPriority = 7002,
    }

    #[repr(i32)]
    pub enum U64 {
        HardwareRevision = 1017,
        FirmwareVersion = 1018,
        FPGAVersion = 1019,
        VRCVersion = 1020,
        RadioVersion = 1021,
        DongleVersion = 1022,
        ParentDriver = 1034,
        BootloaderVersion = 1044,
        PeripheralApplicationVersion = 1048,
        CurrentUniverseId = 2004,
        PreviousUniverseId = 2005,
        DisplayFirmwareVersion = 2006,
        CameraFirmwareVersion = 2027,
        DisplayFPGAVersion = 2029,
        DisplayBootloaderVersion = 2030,
        DisplayHardwareVersion = 2031,
        AudioFirmwareVersion = 2032,
        GraphicsAdapterLuid = 2045,
        AudioBridgeFirmwareVersion = 2061,
        ImageBridgeFirmwareVersion = 2062,
        AdditionalRadioFeatures = 2070,
        SupportedButtons = 3001,
        //Prop_ParentContainer = 5151,
        OverrideContainer = 5152,
    }

    #[repr(i32)]
    pub enum Matrix {
        StatusDisplayTransform = 1013,
        CameraToHeadTransform = 2016,
        ImuToHeadTransform = 2063,
    }

    #[repr(i32)]
    pub enum String {
        TrackingSystemName = 1000,
        ModelNumber = 1001,
        SerialNumber = 1002,
        RenderModelName = 1003,
        ManufacturerName = 1005,
        TrackingFirmwareVersion = 1006,
        HardwareRevision = 1007,
        AllWirelessDongleDescriptions = 1008,
        ConnectedWirelessDongle = 1009,
        Firmware_ManualUpdateURL = 1016,
        Firmware_ProgrammingTarget = 1028,
        DriverVersion = 1031,
        ResourceRoot = 1035,
        RegisteredDeviceType = 1036,
        InputProfilePath = 1037,
        AdditionalDeviceSettingsPath = 1042,
        AdditionalSystemReportData = 1045,
        CompositeFirmwareVersion = 1046,
        ManufacturerSerialNumber = 1049,
        ComputedSerialNumber = 1050,
        DisplayMCImageLeft = 2012,
        DisplayMCImageRight = 2013,
        DisplayGCImage = 2021,
        CameraFirmwareDescription = 2028,
        DriverProvidedChaperonePath = 2048,
        NamedIconPathControllerLeftDeviceOff = 2051,
        NamedIconPathControllerRightDeviceOff = 2052,
        NamedIconPathTrackingReferenceDeviceOff = 2053,
        ExpectedControllerType = 2074,
        HmdColumnCorrectionSettingPrefix = 2077,
        Audio_DefaultPlaybackDeviceId = 2300,
        Audio_DefaultRecordingDeviceId = 2301,
        AttachedDeviceId = 3000,
        ModeLabel = 4006,
        IconPathName = 5000,
        NamedIconPathDeviceOff = 5001,
        NamedIconPathDeviceSearching = 5002,
        NamedIconPathDeviceSearchingAlert = 5003,
        NamedIconPathDeviceReady = 5004,
        NamedIconPathDeviceReadyAlert = 5005,
        NamedIconPathDeviceNotReady = 5006,
        NamedIconPathDeviceStandby = 5007,
        NamedIconPathDeviceAlertLow = 5008,
        NamedIconPathDeviceStandbyAlert = 5009,
        // Prop_ParentContainer = 5151,
        UserConfigPath = 6000,
        InstallPath = 6001,
        ControllerType = 7000,
    }

    // TODO: Arrays
    // a lot of the array types are one-offs so maybe we could use empty structs for them?

    macro_rules! impl_property {
        ($enum:ty, $($ty:ty),+) => {
            pub use $enum::*;
            impl From<$enum> for sys::ETrackedDeviceProperty {
                fn from(t: $enum) -> Self {
                    unsafe { std::mem::transmute(t) } // TODO: Into+FromPrimitive?
                }
            }
            $(
                impl SealedProperty<$ty> for $enum {}
                impl Property<$ty> for $enum {}
            )+
        }
    }

    impl_property!(Bool, bool);
    impl_property!(I32, i32);
    impl_property!(U64, u64);
    impl_property!(Matrix, Matrix3x4);
    impl_property!(self::String, std::string::String, CString);
}

impl<'c> SystemManager<'c> {
    pub(super) fn new(_ctx: &'c Context) -> Self {
        let inner = unsafe { Pin::new_unchecked(sys::VRSystem().as_mut::<'c>().unwrap()) };
        Self {
            ctx: Default::default(),
            inner,
        }
    }

    pub fn get_property<R: PropertyType, T: Property<R>>(
        &mut self,
        index: sys::TrackedDeviceIndex_t,
        prop: T,
    ) -> PropResult<R> {
        prop.get(index, self)
    }

    pub fn get_property_sys<T: PropertyType>(
        &mut self,
        index: sys::TrackedDeviceIndex_t,
        prop: sys::ETrackedDeviceProperty,
    ) -> PropResult<T> {
        T::get(index, self, prop)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    fn test(mut system: SystemManager) {
        let bootloader_version = system.get_property(0, props::DisplayBootloaderVersion);
        let display_version: u64 = system.get_property_sys(
            0,
            sys::ETrackedDeviceProperty::Prop_DisplayHardwareVersion_Uint64,
        ).unwrap();
        let gc_image_string: String = system.get_property(0, props::DisplayGCImage).unwrap();
        let gc_image_cstring: CString = system.get_property(0, props::DisplayGCImage).unwrap();
    }
}
