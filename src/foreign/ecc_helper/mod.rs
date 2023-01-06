use strum_macros::EnumIter;

pub mod circuits;
pub mod etable_op_configure;
pub mod runtime;

pub const ECC_FOREIGN_TABLE_KEY: &'static str = "ecc-helper-table";
pub const ECC_FOREIGN_FUNCTION_NAME_ADD: &'static str = "zkwasm_ecc_add";
pub const ECC_FOREIGN_FUNCTION_NAME_MUL: &'static str = "zkwasm_ecc_mul";

#[derive(Clone, Copy, EnumIter, PartialEq)]
pub enum EccHelperOp {
    Add = 1,
    Mul = 2,
}

impl From<&String> for EccHelperOp {
    fn from(function_name: &String) -> Self {
        match function_name.as_str() {
            ECC_FOREIGN_FUNCTION_NAME_ADD => EccHelperOp::Add,
            ECC_FOREIGN_FUNCTION_NAME_MUL => EccHelperOp::Mul,
            _ => unreachable!(),
        }
    }
}
