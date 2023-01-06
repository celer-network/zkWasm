use super::EccHelperOp;
use crate::{
    constant_from, fixed_curr,
    foreign::ForeignTableConfig,
    traits::circuits::bit_range_table::{
        BitColumn, BitRangeTable, U4Column, U8Column, U8PartialColumn,
    },
};
use halo2_proofs::{
    arithmetic::FieldExt,
    plonk::{Column, ConstraintSystem, Expression, Fixed, TableColumn, VirtualCells},
};
use std::marker::PhantomData;

pub mod assign;
pub mod config;
pub mod expr;

const OP_ARGS_NUM: usize = 5;

pub struct EccHelperEncode();

impl EccHelperEncode {
    pub(super) fn encode_opcode_expr<F: FieldExt>(
        op: Expression<F>,
        args: Vec<Expression<F>>,
        ret: Expression<F>,
    ) -> Expression<F> {
        assert!(args.len() < OP_ARGS_NUM);
        let mut acc = op * constant_from!(1 << (OP_ARGS_NUM * 4));
        for (i, v) in args.into_iter().enumerate() {
            acc = acc + v * constant_from!(1 << (i * 4 + 4));
        }
        acc = acc + ret;
        acc
    }

    pub(super) fn encode_opcode_f<F: FieldExt>(op: EccHelperOp, args: &Vec<u32>, ret: u32) -> F {
        assert!(args.len() < OP_ARGS_NUM);
        let mut acc = F::from(op as u64) * F::from(1u64 << (OP_ARGS_NUM * 4));
        for (i, v) in args.into_iter().enumerate() {
            acc = acc + F::from(*v as u64) * F::from(1u64 << (i * 4 + 4));
        }
        acc = acc + F::from(ret as u64);
        acc
    }

    pub(super) fn encode_table_f<F: FieldExt>(op: EccHelperOp, args: [u32; 2], ret: u32) -> F {
        let mut acc = F::from(op as u64) * F::from(1u64 << (OP_ARGS_NUM * 4));
        for (i, v) in args.into_iter().enumerate() {
            acc = acc + F::from(v as u64) * F::from(1u64 << (i * 4 + 4));
        }
        acc = acc + F::from(ret as u64);
        acc
    }

    pub(super) fn encode_table_expr<F: FieldExt>(
        op: Expression<F>,
        args: [Expression<F>; 3],
        ret: Expression<F>,
    ) -> Expression<F> {
        let mut acc = op * constant_from!(1u64 << (OP_ARGS_NUM * 4));
        for (i, v) in args.into_iter().enumerate() {
            acc = acc + v * constant_from!(1u64 << (i * 4 + 4));
        }
        acc = acc + ret;
        acc
    }
}

#[derive(Clone)]
pub struct EccHelperTableConfig<F: FieldExt> {
    sel: Column<Fixed>,

    op_bit: BitColumn,
    op: U8Column,
    args: [U4Column; OP_ARGS_NUM],
    aux: U8PartialColumn, // limited to u8 except for block first line

    op_valid_set: TableColumn,
    mark: PhantomData<F>,
}

impl<F: FieldExt> EccHelperTableConfig<F> {
    fn new(meta: &mut ConstraintSystem<F>, rtable: &impl BitRangeTable<F>) -> Self {
        let sel = meta.fixed_column();
        let block_first_line_sel = meta.fixed_column();
        let op = rtable.u8_column(meta, "ecc helper op", |meta| fixed_curr!(meta, sel));
        let op_bit = rtable.bit_column(meta, "ecc helper op_bit", |meta| fixed_curr!(meta, sel));
        let args = [0; OP_ARGS_NUM]
            .map(|_| rtable.u4_column(meta, "ecc helper args", |meta| fixed_curr!(meta, sel)));
        let aux = rtable.u8_partial_column(meta, "ecc aux", |meta| {
            fixed_curr!(meta, sel) * (constant_from!(1) - fixed_curr!(meta, block_first_line_sel))
        });
        let op_valid_set = meta.lookup_table_column();

        Self {
            sel,
            op_bit,
            op,
            args,
            aux,
            op_valid_set,
            mark: PhantomData,
        }
    }

    pub fn configure(meta: &mut ConstraintSystem<F>, rtable: &impl BitRangeTable<F>) -> Self {
        let config = Self::new(meta, rtable);
        config._configure(meta);
        config
    }
}

impl<F: FieldExt> ForeignTableConfig<F> for EccHelperTableConfig<F> {
    fn configure_in_table(
        &self,
        meta: &mut ConstraintSystem<F>,
        key: &'static str,
        expr: &dyn Fn(&mut VirtualCells<'_, F>) -> Expression<F>,
    ) {
        // TODO: Fill in
    }
}
