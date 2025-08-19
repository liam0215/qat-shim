mod ffi {
    #![allow(
        dead_code,
        non_camel_case_types,
        non_snake_case,
        non_upper_case_globals
    )]
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

// Safe-ish helpers around the raw C API.
pub mod qat {
    use std::ffi::CString;

    use super::ffi::*;

    #[derive(Debug)]
    pub struct Instance(*mut ::std::os::raw::c_void);

    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    pub enum Status {
        Success,
        Retry,
        Resource,
        Fail(i32),
    }
    impl From<i32> for Status {
        fn from(v: i32) -> Self {
            match v {
                x if x == CPA_STATUS_SUCCESS as i32 => Status::Success,
                x if x == CPA_STATUS_RETRY as i32 => Status::Retry,
                x if x == CPA_STATUS_RESOURCE as i32 => Status::Resource,
                other => Status::Fail(other),
            }
        }
    }

    pub fn start_session<T: AsRef<str>>(process_name: T) -> Result<(), Status> {
        let c_process_name = CString::new(process_name.as_ref()).expect("no interior NULs");
        let rc = unsafe { qat_start_session(c_process_name.as_ptr()) };
        match Status::from(rc) {
            Status::Success => Ok(()),
            e => Err(e),
        }
    }

    pub fn stop_session() -> Result<(), Status> {
        let rc = unsafe { qat_stop_session() };
        match Status::from(rc) {
            Status::Success => Ok(()),
            e => Err(e),
        }
    }

    pub fn get_first_instance() -> Result<Instance, Status> {
        let mut h: *mut ::std::os::raw::c_void = std::ptr::null_mut();
        let rc = unsafe { qat_get_instance(&mut h as *mut _) };
        let st = Status::from(rc);
        match st {
            Status::Success if !h.is_null() => Ok(Instance(h)),
            Status::Success => Err(Status::Fail(-1)),
            _ => Err(st),
        }
    }

    impl Instance {
        pub fn start(&self) -> Result<(), Status> {
            let rc = unsafe { qat_cy_start_instance(self.0) };
            match Status::from(rc) {
                Status::Success => Ok(()),
                e => Err(e),
            }
        }
        pub fn stop(&self) -> Result<(), Status> {
            let rc = unsafe { qat_cy_stop_instance(self.0) };
            match Status::from(rc) {
                Status::Success => Ok(()),
                e => Err(e),
            }
        }
    }
}
