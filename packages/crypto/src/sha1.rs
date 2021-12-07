use sha1::{Digest, Sha1};

use crate::errors::{CryptoError, CryptoResult};

/// Max count of a inputs for sha1.
/// A limit is set to prevent malicious excessive input.
/// Now, we limit ourselves to only small sizes for use cases in Uuid.
pub const INPUTS_MAX_CNT: usize = 4;

/// Max length of a input for sha1
/// After executing the crypto bench according to INPUT_MAX_LEN,
/// the gas factor is determined based on the result.
/// If you modify this value, you need to adjust the gas factor.
pub const INPUT_MAX_LEN: usize = 80;

pub fn sha1_calculate(hash_inputs: &[&[u8]]) -> CryptoResult<[u8; 20]> {
    check_hash_inputs(hash_inputs)?;

    let mut hasher = Sha1::new();
    for &hash_input in hash_inputs.iter() {
        hasher.update(hash_input);
    }
    let buffer: [u8; 20] = hasher.finalize().into();
    Ok(buffer)
}

fn check_hash_inputs(hash_inputs: &[&[u8]]) -> Result<(), CryptoError> {
    if hash_inputs.len() > INPUTS_MAX_CNT {
        return Err(CryptoError::inputs_too_larger(
            INPUTS_MAX_CNT,
            hash_inputs.len(),
        ));
    }

    for &hash_input in hash_inputs.iter() {
        if hash_input.len() > INPUT_MAX_LEN {
            return Err(CryptoError::input_too_long(INPUT_MAX_LEN, hash_input.len()));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha1_calculate() {
        let input1: &str = "input_data1";
        let input2: &str = "input_data2";
        let inputs = &[input1.as_bytes(), input2.as_bytes()];
        let calc_result = sha1_calculate(inputs).unwrap();
        assert_eq!(20, calc_result.len())
    }

    #[test]
    fn test_sha1_over_inputs_maximum_count() {
        let input: &str = "malformed data";
        let inputs: [&[u8]; INPUTS_MAX_CNT + 1] = [input.as_bytes(); INPUTS_MAX_CNT + 1];
        let calc_result = sha1_calculate(&inputs).unwrap_err();
        match calc_result {
            CryptoError::InputsTooLarger { limit, actual } => {
                assert_eq!(limit, INPUTS_MAX_CNT);
                assert_eq!(actual, INPUTS_MAX_CNT + 1);
            }
            _ => panic!("Wrong error message"),
        }
    }

    #[test]
    fn test_sha1_over_input_maximum_length() {
        let input: [u8; INPUT_MAX_LEN + 1] = [0; INPUT_MAX_LEN + 1];
        let calc_result = sha1_calculate(&[&input]).unwrap_err();
        match calc_result {
            CryptoError::InputTooLong { limit, actual } => {
                assert_eq!(limit, INPUT_MAX_LEN);
                assert_eq!(actual, INPUT_MAX_LEN + 1);
            }
            _ => panic!("Wrong error message"),
        }
    }
}
