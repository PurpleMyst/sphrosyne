//! The main entry point of the library, contains the [Client] necessary for all usage.

// As far as I can see, `Client` can be passed around as a shared reference as it is
// nothing more than a pointer to a device handle. The same can not be said for
// `Target`, as that struct actually contains data that is manipulated by the different
// functions.

use std::{
    ffi::c_void,
    marker::PhantomData,
    mem::forget,
    panic::{catch_unwind, RefUnwindSafe},
    ptr::NonNull,
};

use vigem_client_c_sys as ffi;

use crate::{
    error::{check, Error, Result},
    gamepad_state::X360State,
};

/// A connection to the bus
#[derive(Debug)]
pub struct Client {
    vigem: NonNull<ffi::_VIGEM_CLIENT_T>,
}

/// A marker type representing a target being an xbox 360 controller
#[derive(Debug, Clone, Copy)]
pub enum X360 {}

impl Client {
    /// Allocate a new client, connect it and return it.
    pub fn new() -> Result<Self> {
        let vigem = NonNull::new(unsafe { ffi::vigem_alloc() }).ok_or(Error::NoVigemAlloc)?;
        check(unsafe { ffi::vigem_connect(vigem.as_ptr()) })?;
        Ok(Self { vigem })
    }

    /// Create and add a new xbox 360 gamepad target
    pub fn connect_x360_pad(&self) -> Result<Target<'_, X360>> {
        let target =
            NonNull::new(unsafe { ffi::vigem_target_x360_alloc() }).ok_or(Error::NoX360PadAlloc)?;
        check(unsafe { ffi::vigem_target_add(self.vigem.as_ptr(), target.as_ptr()) })?;
        Ok(Target {
            client: self,
            target,
            has_notification: false,
            _marker: PhantomData,
        })
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        unsafe {
            ffi::vigem_disconnect(self.vigem.as_ptr());
            ffi::vigem_free(self.vigem.as_ptr());
        }
    }
}

/// A target. Could be an xbox 360 controller or a dualshock depending on the marker type.
#[derive(Debug)]
pub struct Target<'client, Type> {
    client: &'client Client,
    target: NonNull<ffi::_VIGEM_TARGET_T>,
    has_notification: bool,
    _marker: PhantomData<Type>,
}

impl<Type> Drop for Target<'_, Type> {
    fn drop(&mut self) {
        let _ = self.remove_internal();
    }
}

impl<Type> Target<'_, Type> {
    /// Get this target's vendor id
    pub fn vendor_id(&self) -> u16 {
        unsafe { ffi::vigem_target_get_vid(self.target.as_ptr()) }
    }

    /// Set this target's vendor id
    pub fn set_vendor_id(&mut self, vendor_id: u16) {
        unsafe { ffi::vigem_target_set_vid(self.target.as_ptr(), vendor_id) }
    }

    /// Get this target's product id
    pub fn product_id(&self) -> u16 {
        unsafe { ffi::vigem_target_get_pid(self.target.as_ptr()) }
    }

    /// Set this target's product id
    pub fn set_product_id(&mut self, product_id: u16) {
        unsafe { ffi::vigem_target_set_pid(self.target.as_ptr(), product_id) }
    }

    fn remove_internal(&mut self) -> Result<()> {
        check(unsafe {
            ffi::vigem_target_remove(self.client.vigem.as_ptr(), self.target.as_ptr())
        })?;
        unsafe {
            ffi::vigem_target_free(self.target.as_ptr());
        }
        Ok(())
    }

    /// Remove and deallocate this target
    pub fn remove(mut self) -> Result<()> {
        self.remove_internal()?;
        forget(self);
        Ok(())
    }
}

/// Represents a notification from an xbox 360 controller
#[derive(Debug, Clone, Copy)]
pub struct X360NotificationData {
    /// How much the large motor should be vibrating
    pub large_motor: u8,

    /// How much the small motor should be vibrating
    pub small_motor: u8,

    /// What player are we?
    pub led_number: u8,
}

/// The handle to a notification callback
///
/// This has no special usage, its usage is just to track the type and a pointer to the
/// notification so that we can deallocate it properly once it is no longer needed.
#[derive(Debug)]
pub struct NotificationHandle<F>(*mut F);

unsafe extern "C" fn x360_notification_handler<F>(
    _client: *mut ffi::_VIGEM_CLIENT_T,
    _target: *mut ffi::_VIGEM_TARGET_T,
    large_motor: u8,
    small_motor: u8,
    led_number: u8,
    userdata: *mut c_void,
) where
    F: RefUnwindSafe + Fn(X360NotificationData),
{
    if let Some(f) = unsafe { (userdata as *mut F).as_ref() } {
        let data = X360NotificationData {
            large_motor,
            small_motor,
            led_number,
        };
        let _ = catch_unwind(move || f(data));
    }
}

impl Target<'_, X360> {
    /// Update this controller's state
    pub fn update(&mut self, state: X360State) -> Result<()> {
        check(unsafe {
            ffi::vigem_target_x360_update(
                self.client.vigem.as_ptr(),
                self.target.as_ptr(),
                state.to_xusb_report(),
            )
        })
    }

    /// Get this controller's user index
    pub fn user_index(&self) -> Result<u32> {
        let mut index: u32 = 0xDEADBEEF;
        check(unsafe {
            ffi::vigem_target_x360_get_user_index(
                self.client.vigem.as_ptr(),
                self.target.as_ptr(),
                (&mut index) as *mut _,
            )
        })?;
        Ok(index)
    }

    /// Register a notification callback for this target.
    /// It will be called anytime there is a vibration request and/or the led number changes.
    ///
    /// The callback must be [RefUnwindSafe] since we utilize [catch_unwind] to avoid
    /// panicking over the FFI boundary. This means that any panics in your handler will simply be eaten up.
    ///
    /// The callback must also be [Sync] as it will be called, by reference, in another
    /// thread spawned by ViGEmClient.
    ///
    /// Only one notification callback may be registered at a time.
    /// You can unregister via [unregister_notification](Self::unregister_notification). Make sure to do so before dropping a target or memory may be leaked.
    pub fn register_notification<F>(&mut self, func: F) -> Result<NotificationHandle<F>>
    where
        F: Fn(X360NotificationData) + RefUnwindSafe + Sync,
    {
        if self.has_notification {
            return Err(Error::AlreadyHasCallback);
        }

        let ptr = Box::leak(Box::new(func));
        check(unsafe {
            ffi::vigem_target_x360_register_notification(
                self.client.vigem.as_ptr(),
                self.target.as_ptr(),
                Some(x360_notification_handler::<F>),
                ptr as *mut _ as *mut _,
            )
        })?;
        self.has_notification = true;
        Ok(NotificationHandle(ptr))
    }

    /// Unregister the current notification callback.
    pub fn unregister_notification<F>(&mut self, handle: NotificationHandle<F>) {
        unsafe {
            ffi::vigem_target_x360_unregister_notification(self.target.as_ptr());
            let _ = Box::from_raw(handle.0);
        }
        self.has_notification = false;
    }
}
