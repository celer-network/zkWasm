use super::{EccHelperEncode, EccHelperTableConfig};
use crate::foreign::ecc_helper::EccHelperOp;
use crate::{constant_from, curr, fixed_curr, nextn};
use halo2_proofs::{arithmetic::FieldExt, plonk::ConstraintSystem};
use strum::IntoEnumIterator;

impl<F: FieldExt> EccHelperTableConfig<F> {
    pub fn _configure(&self, meta: &mut ConstraintSystem<F>) {
        meta.create_gate("ecc helper op_bits sum equals to 1", |meta| {
            let sum = EccHelperOp::iter()
                .map(|op| nextn!(meta, self.op_bit.0, op as i32))
                .reduce(|acc, expr| acc + expr)
                .unwrap();

            // TODO: Fix constraint
            vec![fixed_curr!(meta, constant_from!(1))]
        });

        meta.lookup("ecc op lookup", |meta| {
            vec![(
                fixed_curr!(meta, self.sel)
                    * EccHelperEncode::encode_table_expr(
                        curr!(meta, self.op.0),
                        [
                            curr!(meta, self.args[1].0),
                            curr!(meta, self.args[2].0),
                            curr!(meta, self.args[3].0),
                        ],
                        curr!(meta, self.args[4].0),
                    ),
                self.op_valid_set,
            )]
        });

        // TODO: Configure ecc_chip?
    }
}
