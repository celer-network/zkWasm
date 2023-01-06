use super::{
    circuits::EccHelperEncode, EccHelperOp, ECC_FOREIGN_FUNCTION_NAME_ADD,
    ECC_FOREIGN_FUNCTION_NAME_MUL, ECC_FOREIGN_TABLE_KEY,
};
use crate::{
    circuits::{
        etable_compact::{
            op_configure::{
                BitCell, ConstraintBuilder, EventTableCellAllocator, EventTableOpcodeConfig,
                MTableLookupCell, U64OnU8Cell,
            },
            EventTableCommonConfig, MLookupItem, StepStatus,
        },
        mtable_compact::encode::MemoryTableLookupEncode,
        utils::{bn_to_field, Context},
    },
    constant_from, constant_from_bn,
    foreign::{EventTableForeignCallConfigBuilder, ForeignCallInfo},
};
use halo2_proofs::{
    arithmetic::FieldExt,
    plonk::{Error, Expression, VirtualCells},
};
use num_bigint::BigUint;
use specs::step::StepInfo;
use specs::{
    etable::EventTableEntry,
    itable::{OpcodeClass, OPCODE_CLASS_SHIFT},
};
use specs::{host_function::HostPlugin, mtable::VarType};

pub struct ETableEccHelperTableConfig {
    foreign_call_id: u64,

    a: U64OnU8Cell,
    b: U64OnU8Cell,
    res: U64OnU8Cell,
    is_add: BitCell,
    is_mul: BitCell,

    lookup_stack_read_a: MTableLookupCell,
    lookup_stack_read_b: MTableLookupCell,
    lookup_stack_write: MTableLookupCell,
}

pub struct EccForeignCallInfo {}
impl ForeignCallInfo for EccForeignCallInfo {
    fn call_id(&self) -> usize {
        OpcodeClass::ForeignPluginStart as usize + HostPlugin::Ecc as usize
    }
}
pub struct ETableEccHelperTableConfigBuilder {}

impl<F: FieldExt> EventTableForeignCallConfigBuilder<F> for ETableEccHelperTableConfigBuilder {
    fn configure(
        common: &mut EventTableCellAllocator<F>,
        constraint_builder: &mut ConstraintBuilder<F>,
        info: &impl ForeignCallInfo,
    ) -> Box<dyn EventTableOpcodeConfig<F>> {
        let a = common.alloc_u64_on_u8();
        let b = common.alloc_u64_on_u8();
        let res = common.alloc_u64_on_u8();

        let is_add = common.alloc_bit_value();
        let is_mul = common.alloc_bit_value();

        let lookup_stack_read_a = common.alloc_mtable_lookup();
        let lookup_stack_read_b = common.alloc_mtable_lookup();
        let lookup_stack_write = common.alloc_mtable_lookup();

        constraint_builder.push(
            "ecchelper: is one of ops",
            Box::new(move |meta| vec![(is_add.expr(meta) + is_mul.expr(meta) - constant_from!(1))]),
        );

        constraint_builder.lookup(
            ECC_FOREIGN_TABLE_KEY,
            "ecc helper table lookup",
            Box::new(move |meta| {
                let op = is_add.expr(meta) * constant_from!(EccHelperOp::Add)
                    + is_mul.expr(meta) * constant_from!(EccHelperOp::Mul);
                EccHelperEncode::encode_opcode_expr(
                    op,
                    vec![a.expr(meta), b.expr(meta)],
                    res.expr(meta),
                )
            }),
        );

        Box::new(ETableEccHelperTableConfig {
            foreign_call_id: info.call_id() as u64,
            a,
            b,
            res,
            is_add,
            is_mul,
            lookup_stack_read_a,
            lookup_stack_read_b,
            lookup_stack_write,
        })
    }
}

impl<F: FieldExt> EventTableOpcodeConfig<F> for ETableEccHelperTableConfig {
    fn opcode(&self, meta: &mut VirtualCells<'_, F>) -> Expression<F> {
        let pick_one = self.is_add.expr(meta) * constant_from!(EccHelperOp::Add)
            + self.is_mul.expr(meta) * constant_from!(EccHelperOp::Mul);

        constant_from_bn!(&(BigUint::from(self.foreign_call_id) << OPCODE_CLASS_SHIFT)) + pick_one
    }

    fn opcode_class(&self) -> OpcodeClass {
        unreachable!()
    }

    fn mops(&self, meta: &mut VirtualCells<'_, F>) -> Option<Expression<F>> {
        // TODO: Check
        Some(constant_from!(3))
    }

    fn assigned_extra_mops(
        &self,
        _ctx: &mut Context<'_, F>,
        _step: &StepStatus,
        entry: &EventTableEntry,
    ) -> u64 {
        match &entry.step_info {
            StepInfo::CallHost { function_name, .. } => 3,
            _ => unreachable!(),
        }
    }

    fn mtable_lookup(
        &self,
        meta: &mut VirtualCells<'_, F>,
        item: MLookupItem,
        common_config: &EventTableCommonConfig<F>,
    ) -> Option<Expression<F>> {
        match item {
            // TODO: Fix vtype
            MLookupItem::First => Some(MemoryTableLookupEncode::encode_stack_read(
                common_config.eid(meta),
                constant_from!(1),
                common_config.sp(meta) + constant_from!(2),
                constant_from!(VarType::I32),
                self.a.expr(meta),
            )),
            MLookupItem::Second => Some(MemoryTableLookupEncode::encode_stack_read(
                common_config.eid(meta),
                constant_from!(2),
                common_config.sp(meta) + constant_from!(1),
                constant_from!(VarType::I32),
                self.b.expr(meta),
            )),
            MLookupItem::Third => Some(MemoryTableLookupEncode::encode_stack_write(
                common_config.eid(meta),
                constant_from!(3),
                common_config.sp(meta) + constant_from!(2),
                constant_from!(VarType::I32),
                self.res.expr(meta),
            )),
            _ => None,
        }
    }

    fn sp_diff(&self, meta: &mut VirtualCells<'_, F>) -> Option<Expression<F>> {
        Some(constant_from!(1))
    }

    fn assign(
        &self,
        ctx: &mut Context<'_, F>,
        step_info: &StepStatus,
        entry: &EventTableEntry,
    ) -> Result<(), Error> {
        match &entry.step_info {
            StepInfo::CallHost {
                plugin,
                function_name,
                args,
                ret_val,
                ..
            } => {
                assert_eq!(*plugin, HostPlugin::Sha256);

                for (arg, v) in vec![&self.a, &self.b].into_iter().zip(args.iter()) {
                    arg.assign(ctx, *v)?;
                }

                self.res.assign(ctx, ret_val.unwrap())?;

                if function_name == ECC_FOREIGN_FUNCTION_NAME_ADD {
                    self.is_add.assign(ctx, true)?;
                }
                if function_name == ECC_FOREIGN_FUNCTION_NAME_MUL {
                    self.is_mul.assign(ctx, true)?;
                }

                for (i, (lookup, v)) in vec![&self.lookup_stack_read_a, &self.lookup_stack_read_b]
                    .into_iter()
                    .zip(args.iter())
                    .enumerate()
                {
                    lookup.assign(
                        ctx,
                        &MemoryTableLookupEncode::encode_stack_read(
                            BigUint::from(step_info.current.eid),
                            BigUint::from(1 + i as u64),
                            BigUint::from(step_info.current.sp + args.len() as u64 - i as u64),
                            BigUint::from(VarType::I32 as u64),
                            BigUint::from(*v),
                        ),
                    )?;
                }

                self.lookup_stack_write.assign(
                    ctx,
                    &MemoryTableLookupEncode::encode_stack_write(
                        BigUint::from(step_info.current.eid),
                        BigUint::from(1 + args.len() as u64),
                        BigUint::from(step_info.current.sp + args.len() as u64),
                        BigUint::from(VarType::I32 as u64),
                        BigUint::from(ret_val.unwrap()),
                    ),
                )?;
            }
            _ => unreachable!(),
        };
        Ok(())
    }
}
