#[cfg(test)]
mod tests;

use std::{error::Error, fmt::Display};

pub type Register = u32;
pub type Address = u32;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Locator {
    Address(Address),
    FromRegister(Register),
}
pub type Bytes = (u8, u32, u32, u32);
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum ByteCode {
    None,
    Halt,
    Jump {
        addr: Locator,
    },
    JumpIf {
        cond: Register,
        addr: Locator,
    },

    String {
        dst: Register,
        addr: u32,
    },
    Int {
        dst: Register,
        value: u64,
    },
    Float {
        dst: Register,
        value: f64,
    },
    Bool {
        dst: Register,
        value: bool,
    },

    Move {
        dst: Register,
        src: Register,
    },
    Field {
        dst: Register,
        src: Register,
        field: u32,
    },
    Call {
        addr: Locator,
        args: u32,
        dst: Register
    },

    Binary {
        op: BinaryOperation,
        dst: Register,
        left: Register,
        right: Register,
    },
    Unary {
        op: UnaryOperation,
        dst: Register,
        right: Register,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum BinaryOperation {
    Add,
    Sub,
    Div,
    Mul,
}
#[derive(Debug, Clone, PartialEq)]
pub struct BinaryOperationError;
impl TryFrom<u8> for BinaryOperation {
    type Error = BinaryOperationError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Add),
            1 => Ok(Self::Sub),
            2 => Ok(Self::Div),
            3 => Ok(Self::Mul),
            _ => Err(BinaryOperationError),
        }
    }
}
impl From<BinaryOperation> for u8 {
    fn from(val: BinaryOperation) -> Self {
        match val {
            BinaryOperation::Add => 0,
            BinaryOperation::Sub => 1,
            BinaryOperation::Div => 2,
            BinaryOperation::Mul => 3,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum UnaryOperation {
    Neg,
}
#[derive(Debug, Clone, PartialEq)]
pub struct UnaryOperationError;
impl TryFrom<u8> for UnaryOperation {
    type Error = UnaryOperationError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Neg),
            _ => Err(UnaryOperationError),
        }
    }
}
impl From<UnaryOperation> for u8 {
    fn from(val: UnaryOperation) -> Self {
        match val {
            UnaryOperation::Neg => 0,
        }
    }
}

impl From<Locator> for u32 {
    fn from(value: Locator) -> Self {
        match value {
            Locator::FromRegister(addr) | Locator::Address(addr) => addr,
        }
    }
}
impl From<ByteCode> for Bytes {
    fn from(value: ByteCode) -> Self {
        match value {
            ByteCode::None => (0x00, 0, 0, 0),
            ByteCode::Halt => (0x01, 0, 0, 0),
            ByteCode::Jump { addr } => match addr {
                Locator::Address(addr) => (0x02, 0, addr, 0),
                Locator::FromRegister(addr) => (0x03, 0, addr, 0),
            }
            ByteCode::JumpIf { cond, addr } => match addr {
                Locator::Address(addr) => (0x04, cond, addr, 0),
                Locator::FromRegister(addr) => (0x05, cond, addr, 0),
            }
            ByteCode::String { dst, addr } => (0x10, dst, addr, 0),
            ByteCode::Int { dst, value } => {
                (0x11, dst, value as u32, (value >> 32) as u32)
            }
            ByteCode::Float { dst, value } => {
                let bits = value.to_bits();
                (0x12, dst, bits as u32, (bits >> 32) as u32)
            }
            ByteCode::Bool { dst, value } => (0x13, dst, value.into(), 0),
            ByteCode::Move { dst, src } => (0x20, dst, src, 0),
            ByteCode::Field { dst, src, field } => (0x21, dst, src, field),
            ByteCode::Call { addr, args, dst } => match addr {
                Locator::Address(addr) => (0x22, addr, args, dst),
                Locator::FromRegister(addr) => (0x23, addr, args, dst),
            }
            ByteCode::Binary { op, dst, left, right } => (0x30 + op as u8, dst, left, right),
            ByteCode::Unary { op, dst, right } => (0x40 + op as u8, dst, right, 0),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ByteCodeError {
    InvalidOperation,
    InvalidBinaryOperation(u8),
    InvalidUnaryOperation(u8),
}
impl Display for ByteCodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ByteCodeError::InvalidOperation => write!(f, "invalid operation"),
            ByteCodeError::InvalidBinaryOperation(op) => {
                write!(f, "invalid binary operation 0x20 + 0x{op:2x?}")
            }
            ByteCodeError::InvalidUnaryOperation(op) => {
                write!(f, "invalid unary operation 0x30 + 0x{op:2x?}")
            }
        }
    }
}
impl Error for ByteCodeError {}
impl TryFrom<Bytes> for ByteCode {
    type Error = ByteCodeError;
    fn try_from(value: Bytes) -> Result<Self, Self::Error> {
        match value.0 {
            0x00 => Ok(Self::None),
            0x01 => Ok(Self::Halt),
            0x02 => Ok(Self::Jump {
                addr: Locator::Address(value.2),
            }),
            0x03 => Ok(Self::Jump {
                addr: Locator::FromRegister(value.2),
            }),
            0x04 => Ok(Self::JumpIf {
                cond: value.1,
                addr: Locator::Address(value.2),
            }),
            0x05 => Ok(Self::JumpIf {
                cond: value.1,
                addr: Locator::FromRegister(value.2),
            }),

            0x10 => Ok(Self::String {
                dst: value.1,
                addr: value.2,
            }),
            0x11 => Ok(Self::Int {
                dst: value.1,
                value: (value.2 as u64) | ((value.3 as u64) << 32),
            }),
            0x12 => Ok(Self::Float {
                dst: value.1,
                value: f64::from_bits((value.2 as u64) | ((value.3 as u64) << 32)),
            }),
            0x13 => Ok(Self::Bool {
                dst: value.1,
                value: value.2 != 0,
            }),

            0x20 => Ok(Self::Move {
                dst: value.1,
                src: value.2,
            }),
            0x21 => Ok(Self::Field {
                dst: value.1,
                src: value.2,
                field: value.3,
            }),
            0x22 => Ok(Self::Call {
                addr: Locator::Address(value.1),
                args: value.2,
                dst: value.3
            }),
            0x23 => Ok(Self::Call {
                addr: Locator::FromRegister(value.1),
                args: value.2,
                dst: value.3
            }),

            0x30..=0x3f => Ok(Self::Binary {
                op: BinaryOperation::try_from(value.0 - 0x20)
                    .map_err(|_| ByteCodeError::InvalidBinaryOperation(value.0 - 0x20))?,
                dst: value.1,
                left: value.2,
                right: value.3,
            }),
            0x40..=0x4f => Ok(Self::Unary {
                op: UnaryOperation::try_from(value.0 - 0x20)
                    .map_err(|_| ByteCodeError::InvalidUnaryOperation(value.0 - 0x30))?,
                dst: value.1,
                right: value.2,
            }),
            _ => Err(ByteCodeError::InvalidOperation),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    strings: Vec<String>,
    code: Vec<ByteCode>,
}
#[derive(Debug, Clone, PartialEq)]
pub enum ProgramParseError {
    InsufficiantBytes,
    ByteCodeError(ByteCodeError)
}
impl Display for ProgramParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProgramParseError::InsufficiantBytes => write!(f, "insufficiant bytes"),
            ProgramParseError::ByteCodeError(err) => err.fmt(f),
        }
    }
}
impl Error for ProgramParseError {}
impl TryFrom<&[u8]> for Program {
    type Error = ProgramParseError;
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let mut bytes = value.iter();

        let size = {
            let (Some(n1), Some(n2), Some(n3), Some(n4)) = (bytes.next().copied(), bytes.next().copied(), bytes.next().copied(), bytes.next().copied()) else {
                return Err(ProgramParseError::InsufficiantBytes);
            };
            u32::from_be_bytes([n1, n2, n3, n4])
        };
        let mut strings = Vec::with_capacity(size as usize);
        for _ in 0..size {
            let string_size = {
                let (Some(n1), Some(n2), Some(n3), Some(n4)) = (bytes.next().copied(), bytes.next().copied(), bytes.next().copied(), bytes.next().copied()) else {
                    return Err(ProgramParseError::InsufficiantBytes);
                };
                u32::from_be_bytes([n1, n2, n3, n4])
            };
            let mut string = String::new();
            for _ in 0..string_size {
                let Some(c) = bytes.next().copied() else {
                    return Err(ProgramParseError::InsufficiantBytes);
                };
                string.push(c as char);
            }
            strings.push(string);
        }

        let mut code = vec![];
        while let Some(instr) = bytes.next().copied() {
            let arg1 = {
                let (Some(n1), Some(n2), Some(n3), Some(n4)) = (bytes.next().copied(), bytes.next().copied(), bytes.next().copied(), bytes.next().copied()) else {
                    return Err(ProgramParseError::InsufficiantBytes);
                };
                u32::from_be_bytes([n1, n2, n3, n4])
            };
            let arg2 = {
                let (Some(n1), Some(n2), Some(n3), Some(n4)) = (bytes.next().copied(), bytes.next().copied(), bytes.next().copied(), bytes.next().copied()) else {
                    return Err(ProgramParseError::InsufficiantBytes);
                };
                u32::from_be_bytes([n1, n2, n3, n4])
            };
            let arg3 = {
                let (Some(n1), Some(n2), Some(n3), Some(n4)) = (bytes.next().copied(), bytes.next().copied(), bytes.next().copied(), bytes.next().copied()) else {
                    return Err(ProgramParseError::InsufficiantBytes);
                };
                u32::from_be_bytes([n1, n2, n3, n4])
            };
            code.push(ByteCode::try_from((instr, arg1, arg2, arg3)).map_err(ProgramParseError::ByteCodeError)?);
        }

        Ok(Self { strings, code })
    }
}
impl From<Program> for Vec<u8> {
    fn from(program: Program) -> Self {
        let mut bytes = vec![];

        bytes.extend((program.strings.len() as u32).to_be_bytes());
        for string in program.strings {
            bytes.extend((string.len() as u32).to_be_bytes());
            bytes.extend(string.chars().map(|c| c as u8));
        }

        for bytecode in program.code {
            let (instr, arg1, arg2, arg3): Bytes = bytecode.into();
            bytes.push(instr);
            bytes.extend(arg1.to_be_bytes());
            bytes.extend(arg2.to_be_bytes());
            bytes.extend(arg3.to_be_bytes());
        }

        bytes
    }
}
