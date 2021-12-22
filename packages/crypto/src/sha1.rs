use sha1::{Digest, Sha1};

use crate::errors::CryptoResult;

pub fn sha1_calculate(hash_inputs: &[&[u8]]) -> CryptoResult<[u8; 20]> {
    let mut hasher = Sha1::new();
    for &hash_input in hash_inputs.iter() {
        hasher.update(hash_input);
    }
    let buffer: [u8; 20] = hasher.finalize().into();
    Ok(buffer)
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
}
