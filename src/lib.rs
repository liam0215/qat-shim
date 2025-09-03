mod ffi {
    #![allow(
        dead_code,
        non_camel_case_types,
        non_snake_case,
        non_upper_case_globals
    )]
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

pub mod qat {
    use std::ffi::CString;

    use super::ffi::*;

    pub const HASH_LEN: usize = 64; // SHA-512 output size (bytes)
    pub const DATA_LEN: usize = 32; // Ed25519 key size (bytes)

    #[repr(transparent)]
    #[derive(Debug, Clone)]
    pub struct Instance(*mut ::std::os::raw::c_void);

    unsafe impl Send for Instance {}
    unsafe impl Sync for Instance {}

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

    pub fn qae_mem_init() -> Result<(), Status> {
        let rc = unsafe { qat_qae_mem_init() };
        match Status::from(rc) {
            Status::Success => Ok(()),
            e => Err(e),
        }
    }

    pub fn qae_mem_destroy() {
        unsafe { qat_qae_mem_destroy() };
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

    pub fn hash_sha512(msg: &[u8]) -> Result<[u8; HASH_LEN], Status> {
        let mut hash = [0u8; HASH_LEN];
        let rc = unsafe {
            osalHashSHA512Full(
                msg.as_ptr() as *mut Cpa8U,
                hash.as_mut_ptr() as *mut Cpa8U,
                msg.len() as _,
            )
        };
        match Status::from(rc) {
            Status::Success => Ok(hash),
            e => Err(e),
        }
    }

    pub fn qat_alloc(size: usize, node: usize, align: usize) -> *mut u8 {
        let ptr: *mut ::std::os::raw::c_void =
            unsafe { qaeMemAllocNUMA(size, node as ::std::os::raw::c_int, align) };
        ptr as *mut u8
    }

    pub fn qat_free(ptr: *mut u8) {
        let ptr = ptr as *mut ::std::os::raw::c_void;
        let ptr = &ptr as *const *mut ::std::os::raw::c_void;
        unsafe { qaeMemFreeNUMA(ptr as _) };
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

        pub fn set_address_translation(&self) -> Result<(), Status> {
            let rc = unsafe { qat_set_address_translation(self.0) };
            match Status::from(rc) {
                Status::Success => Ok(()),
                e => Err(e),
            }
        }

        pub fn is_polled(&self) -> Result<bool, Status> {
            let mut info = CpaInstanceInfo2::default();
            let rc = unsafe { cpaCyInstanceGetInfo2(self.0, &mut info as *mut CpaInstanceInfo2) };
            match Status::from(rc) {
                Status::Success => Ok(info.isPolled == _CpaBoolean_CPA_TRUE),
                e => Err(e),
            }
        }

        pub fn poll_once(&self) -> Result<(), Status> {
            let rc = unsafe { icp_sal_CyPollInstance(self.0, 0) };
            match Status::from(rc) {
                Status::Success | Status::Retry => Ok(()),
                e => Err(e),
            }
        }

        pub fn eddsa_gen_public_key(
            &self,
            private_key: &[u8; DATA_LEN],
        ) -> Result<[u8; DATA_LEN], Status> {
            let mut public_key = [0u8; DATA_LEN];
            let rc = unsafe {
                edDsaGenPubKey(
                    private_key.as_ptr() as *mut Cpa8U,
                    public_key.as_mut_ptr() as *mut Cpa8U,
                    self.0,
                )
            };
            match Status::from(rc) {
                Status::Success => Ok(public_key),
                e => Err(e),
            }
        }

        pub fn eddsa_sign_msg(
            &self,
            private_key: &[u8; DATA_LEN],
            message_hash: &[u8; HASH_LEN],
        ) -> Result<[u8; DATA_LEN * 2], Status> {
            let mut signature = [0u8; DATA_LEN * 2];
            let rc = unsafe {
                edDsaSign(
                    private_key.as_ptr() as *mut Cpa8U,
                    message_hash.as_ptr() as *mut Cpa8U,
                    signature.as_mut_ptr() as *mut Cpa8U,
                    self.0,
                )
            };
            match Status::from(rc) {
                Status::Success => Ok(signature),
                e => Err(e),
            }
        }

        pub fn eddsa_verify_msg(
            &self,
            public_key: &[u8; DATA_LEN],
            message_hash: &[u8; HASH_LEN],
            signature: &[u8; DATA_LEN * 2],
        ) -> Result<(), Status> {
            let rc = unsafe {
                edDsaVerify(
                    public_key.as_ptr() as *mut Cpa8U,
                    message_hash.as_ptr() as *mut Cpa8U,
                    signature.as_ptr() as *mut Cpa8U,
                    self.0,
                )
            };
            match Status::from(rc) {
                Status::Success => Ok(()),
                e => Err(e),
            }
        }

        pub fn point_multiplication(
            &self,
            x: &[u8; DATA_LEN],
            y: &[u8; DATA_LEN],
            s: &[u8; DATA_LEN],
        ) -> Result<[u8; DATA_LEN], Status> {
            let mut product_x = [0u8; DATA_LEN];
            let mut product_y = [0u8; DATA_LEN];
            let mut product = [0u8; DATA_LEN];
            let rc = unsafe {
                pointMultiplication(
                    x.as_ptr() as *mut Cpa8U,
                    y.as_ptr() as *mut Cpa8U,
                    s.as_ptr() as *mut Cpa8U,
                    product_x.as_mut_ptr() as *mut Cpa8U,
                    product_y.as_mut_ptr() as *mut Cpa8U,
                    self.0,
                )
            };
            match Status::from(rc) {
                Status::Success => {
                    unsafe {
                        encodePoint(
                            product_x.as_ptr() as *mut Cpa8U,
                            product_y.as_ptr() as *mut Cpa8U,
                            product.as_mut_ptr() as *mut Cpa8U,
                        );
                    }
                    Ok(product)
                }
                e => Err(e),
            }
        }
        pub fn point_mul_unencoded(
            &self,
            x: &[u8; DATA_LEN],
            y: &[u8; DATA_LEN],
            s: &[u8; DATA_LEN],
        ) -> Result<([u8; DATA_LEN], [u8; DATA_LEN]), Status> {
            let mut product_x = [0u8; DATA_LEN];
            let mut product_y = [0u8; DATA_LEN];
            let rc = unsafe {
                pointMultiplication(
                    x.as_ptr() as *mut Cpa8U,
                    y.as_ptr() as *mut Cpa8U,
                    s.as_ptr() as *mut Cpa8U,
                    product_x.as_mut_ptr() as *mut Cpa8U,
                    product_y.as_mut_ptr() as *mut Cpa8U,
                    self.0,
                )
            };
            match Status::from(rc) {
                Status::Success => Ok((product_x, product_y)),
                e => Err(e),
            }
        }

        pub fn add_points(
            &self,
            x1: &[u8; DATA_LEN],
            y1: &[u8; DATA_LEN],
            x2: &[u8; DATA_LEN],
            y2: &[u8; DATA_LEN],
        ) -> Result<([u8; DATA_LEN], [u8; DATA_LEN]), Status> {
            let sum_x = [0u8; DATA_LEN];
            let sum_y = [0u8; DATA_LEN];
            let rc = unsafe {
                addPoints(
                    x1.as_ptr() as *mut Cpa8U,
                    y1.as_ptr() as *mut Cpa8U,
                    x2.as_ptr() as *mut Cpa8U,
                    y2.as_ptr() as *mut Cpa8U,
                    sum_x.as_ptr() as *mut Cpa8U,
                    sum_y.as_ptr() as *mut Cpa8U,
                )
            };
            match Status::from(rc) {
                Status::Success => Ok((sum_x, sum_y)),
                e => Err(e),
            }
        }

        pub fn encode_point(
            &self,
            x: &[u8; DATA_LEN],
            y: &[u8; DATA_LEN],
        ) -> Result<[u8; DATA_LEN], Status> {
            let encoded = [0u8; DATA_LEN];
            unsafe {
                encodePoint(
                    x.as_ptr() as *mut Cpa8U,
                    y.as_ptr() as *mut Cpa8U,
                    encoded.as_ptr() as *mut Cpa8U,
                );
            }
            Ok(encoded)
        }
    }
}
