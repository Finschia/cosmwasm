#[cfg(target_arch = "wasm32")]
use crate::exports::make_dependencies;
use crate::memory::{consume_region, Region};
#[cfg(not(target_arch = "wasm32"))]
use crate::mock::mock_dependencies;
use crate::serde::from_slice;
use crate::{Deps, DepsMut, Env};

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

    // By existing design, ownership of Deps is intended to be held outside the contract logic.
    // So, it is not possible to provide a simple getter style without changing the existing design.
    pub fn with_deps<C, R>(callback: C) -> R
    where
        C: FnOnce(Deps) -> R,
    {
        /* FIXME:
        In actual product execution, deps is a stateless implementation.
        so even if we call make_dependencies in multiple places, there are no practical problems other than design violations.
        However, mock_dependencies in the test environment is different.
        Since it has an individual state, if it is created in several places, the test environment will be inconsistent with each different state.

        Therefore, it is currently not possible to develop logic that tests dynamic_call and the existing message driven.
        */
        #[cfg(target_arch = "wasm32")]
        let deps = make_dependencies();
        #[cfg(not(target_arch = "wasm32"))]
        let deps = mock_dependencies(&[]);

        callback(deps.as_ref())
    }

    pub fn with_deps_mut<C, R>(callback: C) -> R
    where
        C: FnOnce(DepsMut) -> R,
    {
        // FIXME: same above
        #[cfg(target_arch = "wasm32")]
        let mut deps = make_dependencies();
        #[cfg(not(target_arch = "wasm32"))]
        let mut deps = mock_dependencies(&[]);

        callback(deps.as_mut())
    }
}
