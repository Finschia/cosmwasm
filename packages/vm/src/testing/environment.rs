use super::{MockQuerier, MockStorage};
use crate::conversion::ref_to_u32;
use crate::memory::{read_region, write_region};
use crate::BackendApi;
use crate::Environment;
use crate::VmResult;
use crate::WasmerVal;

pub fn write_data_to_mock_env<A: BackendApi>(
    env: &Environment<A, MockStorage, MockQuerier>,
    data: &[u8],
) -> VmResult<u32> where {
    let result = env.call_function1("allocate", &[(data.len() as u32).into()])?;
    let region_ptr = ref_to_u32(&result)?;
    write_region(&env.memory(), region_ptr, data)?;
    Ok(region_ptr)
}

pub fn read_data_from_mock_env<A: BackendApi>(
    env: &Environment<A, MockStorage, MockQuerier>,
    wasm_ptr: &WasmerVal,
    size: usize,
) -> VmResult<Vec<u8>> {
    let region_ptr = ref_to_u32(wasm_ptr)?;
    read_region(&env.memory(), region_ptr, size)
}
