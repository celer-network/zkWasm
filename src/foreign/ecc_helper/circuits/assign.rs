use super::{EccHelperEncode, EccHelperTableConfig};
use crate::foreign::ecc_helper::EccHelperOp;
use halo2_proofs::{arithmetic::FieldExt, circuit::Layouter, plonk::Error};
use specs::etable::EventTableEntry;

pub struct EccHelperTableChip<F: FieldExt> {
    pub(crate) config: EccHelperTableConfig<F>,
}

impl<F: FieldExt> EccHelperTableChip<F> {
    pub fn new(config: EccHelperTableConfig<F>) -> Self {
        Self { config }
    }
    pub fn assign(
        &self,
        layouter: &mut impl Layouter<F>,
        entry: &Vec<EventTableEntry>,
    ) -> Result<(), Error> {
        layouter.assign_region(
            || "ecc helper assign",
            |mut region| {
                // TODO: assign helper?
                // TODO: asign op args ret
                // TODO: assign add and mul

                Ok(())
            },
        )?;
        Ok(())
    }

    pub fn init(&self, layouter: &mut impl Layouter<F>) -> Result<(), Error> {
        layouter.assign_table(
            || "ecc helper table",
            |mut table| {
                table.assign_cell(
                    || "ecc helper table",
                    self.config.op_valid_set,
                    0,
                    || Ok(F::zero()),
                )?;
                let mut index = 1;

                for a in 0..1 << 4 {
                    for b in 0..1 << 4 {
                        table.assign_cell(
                            || "ecc helper table",
                            self.config.op_valid_set,
                            index,
                            || {
                                // TODO: Fix this
                                Ok(EccHelperEncode::encode_table_f::<F>(
                                    EccHelperOp::Add,
                                    [a, b],
                                    (a & b) & 0xf,
                                ))
                            },
                        )?;
                        index += 1;

                        table.assign_cell(
                            || "ecc helper table",
                            self.config.op_valid_set,
                            index,
                            || {
                                // TODO: Fix this
                                Ok(EccHelperEncode::encode_table_f::<F>(
                                    EccHelperOp::Mul,
                                    [a, b],
                                    (a & b) & 0xf,
                                ))
                            },
                        )?;
                        index += 1;
                    }
                }

                Ok(())
            },
        )?;

        Ok(())
    }
}
