pub mod cranelift;
pub mod singlepass;

pub use wasmer_runtime_core::backend::Compiler;
use wasmer_runtime_core::vm::Ctx;

pub fn compiler_for_backend(backend: &str) -> Option<Box<dyn Compiler>> {
    match backend {
        #[cfg(any(feature = "cranelift", feature = "default-cranelift"))]
        "cranelift" => Some(cranelift::compiler()),

        #[cfg(any(feature = "singlepass", feature = "default-singlepass"))]
        "singlepass" => Some(singlepass::compiler()),

        _ => None,
    }
}

#[derive(Debug)]
pub struct InsufficientGasLeft;

/// Decreases gas left by the given amount.
/// If the amount exceeds the available gas, the remaining gas is set to 0 and
/// an InsufficientGasLeft error is returned.
pub fn decrease_gas_left(ctx: &mut Ctx, amount: u64) -> Result<(), InsufficientGasLeft> {
    let remaining = get_gas_left(ctx);
    if amount > remaining {
        set_gas_left(ctx, 0);
        Err(InsufficientGasLeft)
    } else {
        set_gas_left(ctx, remaining - amount);
        Ok(())
    }
}

#[cfg(feature = "default-cranelift")]
pub use cranelift::{compile, get_gas_left, set_gas_left, BACKEND_NAME};

#[cfg(feature = "default-singlepass")]
pub use singlepass::{compile, get_gas_left, set_gas_left, BACKEND_NAME};

#[cfg(test)]
#[cfg(feature = "default-singlepass")]
mod test {
    use super::*;
    use wasmer_runtime_core::{imports, Instance as WasmerInstance};

    fn instantiate(code: &[u8]) -> WasmerInstance {
        let module = compile(code).unwrap();
        let import_obj = imports! { "env" => {}, };
        module.instantiate(&import_obj).unwrap()
    }

    #[test]
    fn decrease_gas_left_works() {
        let wasm = wat::parse_str("(module)").unwrap();
        let mut instance = instantiate(&wasm);

        let before = get_gas_left(instance.context());
        decrease_gas_left(instance.context_mut(), 32).unwrap();
        let after = get_gas_left(instance.context());
        assert_eq!(after, before - 32);
    }

    #[test]
    fn decrease_gas_left_can_consume_all_gas() {
        let wasm = wat::parse_str("(module)").unwrap();
        let mut instance = instantiate(&wasm);

        let before = get_gas_left(instance.context());
        decrease_gas_left(instance.context_mut(), before).unwrap();
        let after = get_gas_left(instance.context());
        assert_eq!(after, 0);
    }

    #[test]
    fn decrease_gas_left_errors_for_amount_greater_than_remaining() {
        let wasm = wat::parse_str("(module)").unwrap();
        let mut instance = instantiate(&wasm);

        let before = get_gas_left(instance.context());
        let result = decrease_gas_left(instance.context_mut(), before + 1);
        match result.unwrap_err() {
            InsufficientGasLeft => {}
        }
        let after = get_gas_left(instance.context());
        assert_eq!(after, 0);
    }
}
