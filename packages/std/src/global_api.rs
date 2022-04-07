use crate::memory::{consume_region, Region};
use crate::serde::from_slice;
use crate::Env;

extern "C" {
    fn global_env() -> u32;
}

pub struct GlobalApi {}
impl GlobalApi {
    pub fn env() -> Env {
        let env_ptr = unsafe { global_env() };
        let vec_env = unsafe { consume_region(env_ptr as *mut Region) };
        from_slice(&vec_env).unwrap()
    }
}
