#[cfg(target_arch = "wasm32")]
use crate::memory::{consume_region, Region};
#[cfg(not(target_arch = "wasm32"))]
use crate::mock::mock_env;
#[cfg(target_arch = "wasm32")]
use crate::serde::from_slice;
use crate::Env;

#[cfg(target_arch = "wasm32")]
extern "C" {
    fn global_env() -> u32;
}

pub struct GlobalApi {}
impl GlobalApi {
    #[cfg(target_arch = "wasm32")]
    pub fn env() -> Env {
        let env_ptr = unsafe { global_env() };
        let vec_env = unsafe { consume_region(env_ptr as *mut Region) };
        from_slice(&vec_env).unwrap()
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn env() -> Env {
        mock_env()
    }
}
