use super::*;

impl<'a> Codegen<'a> {
    pub(super) fn emit_store(&self, ty: &Type, offset: u32, code: &mut Vec<u8>) {
        match ty.strip_const() {
            Type::Bool | Type::Char | Type::SignedChar => {
                code.push(OP_I32_STORE8);
                write_u32(code, 0);
                write_u32(code, offset);
            }
            Type::UnsignedChar => {
                code.push(OP_I32_STORE8);
                write_u32(code, 0);
                write_u32(code, offset);
            }
            Type::Short | Type::UnsignedShort => {
                code.push(OP_I32_STORE16);
                write_u32(code, 1);
                write_u32(code, offset);
            }
            Type::Int
            | Type::UnsignedInt
            | Type::Long
            | Type::UnsignedLong
            | Type::Pointer(_)
            | Type::Reference(_)
            | Type::Enum(_) => {
                code.push(OP_I32_STORE);
                write_u32(code, 2);
                write_u32(code, offset);
            }
            Type::LongLong | Type::UnsignedLongLong => {
                code.push(OP_I64_STORE);
                write_u32(code, 3);
                write_u32(code, offset);
            }
            Type::Float => {
                code.push(OP_F32_STORE);
                write_u32(code, 2);
                write_u32(code, offset);
            }
            Type::Double => {
                code.push(OP_F64_STORE);
                write_u32(code, 3);
                write_u32(code, offset);
            }
            _ => {
                code.push(OP_I32_STORE);
                write_u32(code, 2);
                write_u32(code, offset);
            }
        }
    }

    pub(super) fn emit_conversion(&self, from: &Type, to: &Type, code: &mut Vec<u8>) {
        let from = from.strip_const().strip_reference();
        let to = to.strip_const().strip_reference();
        if to == &Type::Bool && from != &Type::Bool {
            match from {
                Type::Float => {
                    code.push(OP_F32_CONST);
                    code.extend_from_slice(&0.0f32.to_le_bytes());
                    code.push(OP_F32_NE);
                }
                Type::Double => {
                    code.push(OP_F64_CONST);
                    code.extend_from_slice(&0.0f64.to_le_bytes());
                    code.push(OP_F64_NE);
                }
                Type::LongLong | Type::UnsignedLongLong => {
                    code.push(OP_I64_EQZ);
                    code.push(OP_I32_EQZ);
                }
                _ => {
                    code.push(OP_I32_CONST);
                    write_i32(code, 0);
                    code.push(OP_I32_NE);
                }
            }
            return;
        }
        if from == to {
            return;
        }
        if from.to_wasm_type() == to.to_wasm_type() {
            emit_integer_narrowing(to, code);
            return;
        }
        if matches!(to, Type::LongLong | Type::UnsignedLongLong) {
            if from == &Type::Double {
                code.push(if to.is_signed() {
                    OP_I64_TRUNC_F64_S
                } else {
                    OP_I64_TRUNC_F64_U
                });
            } else if from == &Type::Float {
                code.push(if to.is_signed() {
                    OP_I64_TRUNC_F32_S
                } else {
                    OP_I64_TRUNC_F32_U
                });
            } else if from.is_integer() || from.is_pointer() || from.is_reference() {
                if from.is_signed() {
                    code.push(OP_I64_EXTEND_I32_S);
                } else {
                    code.push(OP_I64_EXTEND_I32_U);
                }
            }
        } else if matches!(from, Type::LongLong | Type::UnsignedLongLong) {
            if to.is_integer() || to.is_pointer() || to.is_reference() {
                code.push(OP_I32_WRAP_I64);
            } else if to == &Type::Double {
                code.push(if from.is_signed() {
                    OP_F64_CONVERT_I64_S
                } else {
                    OP_F64_CONVERT_I64_U
                });
            } else if to == &Type::Float {
                code.push(if from.is_signed() {
                    OP_F32_CONVERT_I64_S
                } else {
                    OP_F32_CONVERT_I64_U
                });
            }
        } else if from == &Type::Float && to.is_integer() {
            code.push(if to.is_signed() {
                OP_I32_TRUNC_F32_S
            } else {
                OP_I32_TRUNC_F32_U
            });
        } else if from == &Type::Double && to.is_integer() {
            code.push(if to.is_signed() {
                OP_I32_TRUNC_F64_S
            } else {
                OP_I32_TRUNC_F64_U
            });
        } else if from.is_integer() && to == &Type::Float {
            code.push(if from.is_signed() {
                OP_F32_CONVERT_I32_S
            } else {
                OP_F32_CONVERT_I32_U
            });
        } else if from.is_integer() && to == &Type::Double {
            code.push(if from.is_signed() {
                OP_F64_CONVERT_I32_S
            } else {
                OP_F64_CONVERT_I32_U
            });
        } else if from == &Type::Float && to == &Type::Double {
            code.push(OP_F64_PROMOTE_F32);
        } else if from == &Type::Double && to == &Type::Float {
            code.push(OP_F32_DEMOTE_F64);
        }
        emit_integer_narrowing(to, code);
    }

    pub(super) fn emit_load(&self, ty: &Type, offset: u32, code: &mut Vec<u8>) {
        match ty.strip_const() {
            Type::Char | Type::SignedChar => {
                code.push(OP_I32_LOAD8_S);
                write_u32(code, 0);
                write_u32(code, offset);
            }
            Type::Bool | Type::UnsignedChar => {
                code.push(OP_I32_LOAD8_U);
                write_u32(code, 0);
                write_u32(code, offset);
            }
            Type::Short => {
                code.push(OP_I32_LOAD16_S);
                write_u32(code, 1);
                write_u32(code, offset);
            }
            Type::UnsignedShort => {
                code.push(OP_I32_LOAD16_U);
                write_u32(code, 1);
                write_u32(code, offset);
            }
            Type::Int
            | Type::UnsignedInt
            | Type::Long
            | Type::UnsignedLong
            | Type::Pointer(_)
            | Type::Reference(_)
            | Type::Enum(_) => {
                code.push(OP_I32_LOAD);
                write_u32(code, 2);
                write_u32(code, offset);
            }
            Type::LongLong | Type::UnsignedLongLong => {
                code.push(OP_I64_LOAD);
                write_u32(code, 3);
                write_u32(code, offset);
            }
            Type::Float => {
                code.push(OP_F32_LOAD);
                write_u32(code, 2);
                write_u32(code, offset);
            }
            Type::Double => {
                code.push(OP_F64_LOAD);
                write_u32(code, 3);
                write_u32(code, offset);
            }
            _ => {
                code.push(OP_I32_LOAD);
                write_u32(code, 2);
                write_u32(code, offset);
            }
        }
    }
}

fn emit_integer_narrowing(ty: &Type, code: &mut Vec<u8>) {
    match ty.strip_const() {
        Type::Char | Type::SignedChar => code.push(OP_I32_EXTEND8_S),
        Type::Short => code.push(OP_I32_EXTEND16_S),
        Type::UnsignedChar => {
            code.push(OP_I32_CONST);
            write_i32(code, 0xff);
            code.push(OP_I32_AND);
        }
        Type::UnsignedShort => {
            code.push(OP_I32_CONST);
            write_i32(code, 0xffff);
            code.push(OP_I32_AND);
        }
        _ => {}
    }
}
