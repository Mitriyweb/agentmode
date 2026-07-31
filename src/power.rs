use anyhow::Result;

#[cfg(target_os = "macos")]
use anyhow::anyhow;
#[cfg(target_os = "macos")]
use core_foundation::base::TCFType;
#[cfg(target_os = "macos")]
use core_foundation::string::{CFString, CFStringRef};

#[cfg(target_os = "macos")]
type IOPMAssertionID = u32;
#[cfg(target_os = "macos")]
type IOReturn = i32;

#[cfg(target_os = "macos")]
const IOPM_ASSERTION_LEVEL_ON: u32 = 255;
#[cfg(target_os = "macos")]
const KERN_SUCCESS: i32 = 0;

// Prevent idle system sleep (works with lid closed, no external display needed)
#[cfg(target_os = "macos")]
const ASSERTION_TYPE: &str = "PreventUserIdleSystemSleep";
#[cfg(target_os = "macos")]
const ASSERTION_NAME: &str = "agentmode: AI agent is running";

#[cfg(target_os = "macos")]
#[link(name = "IOKit", kind = "framework")]
extern "C" {
    fn IOPMAssertionCreateWithName(
        assertion_type: CFStringRef,
        assertion_level: u32,
        assertion_name: CFStringRef,
        assertion_id: *mut IOPMAssertionID,
    ) -> IOReturn;

    fn IOPMAssertionRelease(assertion_id: IOPMAssertionID) -> IOReturn;
}

/// RAII guard — holds the IOKit power assertion alive.
/// When dropped, macOS is free to sleep again.
pub struct PowerAssertion {
    #[cfg(target_os = "macos")]
    id: IOPMAssertionID,
}

impl PowerAssertion {
    pub fn acquire() -> Result<Self> {
        #[cfg(target_os = "macos")]
        {
            let assertion_type = CFString::new(ASSERTION_TYPE);
            let assertion_name = CFString::new(ASSERTION_NAME);
            let mut id: IOPMAssertionID = 0;

            let rc = unsafe {
                IOPMAssertionCreateWithName(
                    assertion_type.as_concrete_TypeRef(),
                    IOPM_ASSERTION_LEVEL_ON,
                    assertion_name.as_concrete_TypeRef(),
                    &mut id,
                )
            };

            if rc != KERN_SUCCESS {
                return Err(anyhow!(
                    "IOPMAssertionCreateWithName failed with code {}",
                    rc
                ));
            }

            Ok(Self { id })
        }
        #[cfg(not(target_os = "macos"))]
        {
            println!("⚠ Power assertions are only supported on macOS.");
            Ok(Self {})
        }
    }
}

impl Drop for PowerAssertion {
    fn drop(&mut self) {
        #[cfg(target_os = "macos")]
        unsafe {
            IOPMAssertionRelease(self.id);
        }
    }
}
