use specs::{host_function::HostPlugin, types::ValueType};
use wasmi::{RuntimeArgs, RuntimeValue};

use crate::runtime::host::{ForeignContext, HostEnv};

use super::{EccHelperOp, ECC_FOREIGN_FUNCTION_NAME_ADD, ECC_FOREIGN_FUNCTION_NAME_MUL};

struct Context {}
impl ForeignContext for Context {}

fn ecc_add(args: RuntimeArgs) -> Option<RuntimeValue> {
    panic!("not implemented");
}

fn ecc_mul(args: RuntimeArgs) -> Option<RuntimeValue> {
    panic!("not implemented");
}

pub fn register_ecc_foreign(env: &mut HostEnv) {
    env.register_function(
        ECC_FOREIGN_FUNCTION_NAME_ADD,
        EccHelperOp::Add as usize,
        Box::new(Context {}),
        // TODO: Fix specs
        specs::host_function::Signature {
            params: vec![ValueType::I32, ValueType::I32, ValueType::I32],
            return_type: Some(specs::types::ValueType::I32),
        },
        Box::new(|_, args| ecc_add(args)),
        HostPlugin::Ecc,
    )
    .unwrap();

    env.register_function(
        ECC_FOREIGN_FUNCTION_NAME_MUL,
        EccHelperOp::Mul as usize,
        Box::new(Context {}),
        // TODO: Fix specs
        specs::host_function::Signature {
            params: vec![ValueType::I32, ValueType::I32, ValueType::I32],
            return_type: Some(specs::types::ValueType::I32),
        },
        Box::new(|_, args| ecc_mul(args)),
        HostPlugin::Ecc,
    )
    .unwrap();
}
