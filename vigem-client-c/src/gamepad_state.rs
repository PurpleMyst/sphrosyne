//! Contains structures and enums needed to represent a gamepad state

use bitflags::bitflags;

use vigem_client_c_sys as ffi;

bitflags! {
    /// Represents an xbox 360 controller's buttons
    #[derive(Default)]
    pub struct X360Buttons: u16 {
        const DPAD_UP = 0x0001;
        const DPAD_DOWN = 0x0002;
        const DPAD_LEFT = 0x0004;
        const DPAD_RIGHT = 0x0008;
        const START = 0x0010;
        const BACK = 0x0020;
        const LEFT_THUMB = 0x0040;
        const RIGHT_THUMB = 0x0080;
        const LEFT_SHOULDER = 0x0100;
        const RIGHT_SHOULDER = 0x0200;
        const A = 0x1000;
        const B = 0x2000;
        const X = 0x4000;
        const Y = 0x8000;
    }
}

/// Represents an xbox 360 controller's state
#[derive(Debug, Copy, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct X360State {
    /// The controller's buttons
    pub buttons: X360Buttons,

    /// The controller's left analog trigger's value, ranging from 0 to 255
    pub left_trigger: u8,

    /// The controller's right analog trigger's value, ranging from 0 to 255
    pub right_trigger: u8,

    /// The controller's left thumbstick axes.
    /// The first element of the tuple is the X axis, while the second one is the Y AXis.
    pub left_thumbstick: (i16, i16),

    /// The controller's right thumbstick axes.
    /// The first element of the tuple is the X axis, while the second one is the Y AXis.
    pub right_thumbstick: (i16, i16),
}

#[cfg(feature = "serde")]
impl serde::Serialize for X360Buttons {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.bits().serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for X360Buttons {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <u16 as serde::Deserialize<'de>>::deserialize(deserializer)?;

        Self::from_bits(value)
            .ok_or_else(|| serde::de::Error::custom(format!("Invalid X360Buttons: {:#x}", value)))
    }
}

impl X360State {
    pub(crate) fn to_xusb_report(self) -> ffi::_XUSB_REPORT {
        ffi::_XUSB_REPORT {
            wButtons: self.buttons.bits(),
            bLeftTrigger: self.left_trigger,
            bRightTrigger: self.right_trigger,
            sThumbLX: self.left_thumbstick.0,
            sThumbLY: self.left_thumbstick.1,
            sThumbRX: self.right_thumbstick.0,
            sThumbRY: self.right_thumbstick.1,
        }
    }
}
