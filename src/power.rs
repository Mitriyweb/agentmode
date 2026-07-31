use anyhow::{anyhow, Result};
use core_foundation::base::TCFType;
use core_foundation::string::{CFString, CFStringRef};

type IOPMAssertionID = u32;
type IOReturn = i32;

const IOPM_ASSERTION_LEVEL_ON: u32 = 255;
const KERN_SUCCESS: i32 = 0;

// Prevent idle system sleep (works with lid closed, no external display needed)
const ASSERTION_TYPE: &str = "PreventUserIdleSystemSleep";
const ASSERTION_NAME: &str = "agentmode: AI agent is running";

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
    id: IOPMAssertionID,
}

impl PowerAssertion {
    pub fn acquire() -> Result<Self> {
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
}

impl Drop for PowerAssertion {
    fn drop(&mut self) {
        unsafe {
            IOPMAssertionRelease(self.id);
        }
    }
}
