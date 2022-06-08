use std::fmt::Debug;
use std::io::Read;
use std::ops::Index;

use byteorder::{BigEndian, ReadBytesExt};

use crate::class::{ClassLoadError, EmptyContext, ReadAll, ReadOne};

// =============================================================================
// CONTEXT
// =============================================================================

pub struct ConstantPoolContext {
    pub constant_pool: &'static ConstantPool
}

// ============================================================================
// CONSTANT POOL
// ============================================================================

// ConstantClass ---------------------------------------------------------------

#[derive(Debug)]
pub struct Class{
    name_index: u16,
}

impl ReadOne for Class{
    fn read_one<R: ReadBytesExt>(reader: &mut R, _: &EmptyContext) -> Result<Self, ClassLoadError> {
        let name_index = reader.read_u16::<BigEndian>()?;

        Ok(Class{
            name_index
        })
    }
}

// ReferenceConstant -----------------------------------------------------------
// Covers:
//  - Field
//  - Method
//  - InterfaceMethod

#[derive(Debug)]
pub struct ClassReference{
    class_index: u16,
    name_and_type_index: u16,
}

impl ReadOne for ClassReference{
    fn read_one<R: ReadBytesExt>(reader: &mut R, _: &EmptyContext) -> Result<Self, ClassLoadError> {
        let class_index = reader.read_u16::<BigEndian>()?;
        let name_and_type_index = reader.read_u16::<BigEndian>()?;

        Ok(ClassReference{
            class_index,
            name_and_type_index,
        })
    }
}

// ConstantString --------------------------------------------------------------

#[derive(Debug)]
pub struct String{
    string_index: u16,
}

impl ReadOne for String{
    fn read_one<R: ReadBytesExt>(reader: &mut R, _: &EmptyContext) -> Result<Self, ClassLoadError> {
        let string_index = reader.read_u16::<BigEndian>()?;

        Ok(String{
            string_index
        })
    }
}

// ConstantInteger -------------------------------------------------------------

#[derive(Debug)]
pub struct Integer{
    value: i32,
}

impl ReadOne for Integer{
    fn read_one<R: ReadBytesExt>(reader: &mut R, _: &EmptyContext) -> Result<Self, ClassLoadError> {
        let value = reader.read_i32::<BigEndian>()?;

        Ok(Integer{
            value
        })
    }
}

// ConstantFloat ---------------------------------------------------------------

#[derive(Debug)]
pub struct Float{
    value: f32,
}

impl ReadOne for Float{
    fn read_one<R: ReadBytesExt>(reader: &mut R, _: &EmptyContext) -> Result<Self, ClassLoadError> {
        let value = reader.read_f32::<BigEndian>()?;

        Ok(Float{
            value
        })
    }
}

// ConstantLong ----------------------------------------------------------------

#[derive(Debug)]
pub struct Long{
    value: i64,
}

impl ReadOne for Long{
    fn read_one<R: ReadBytesExt>(reader: &mut R, _: &EmptyContext) -> Result<Self, ClassLoadError> {
        let value = reader.read_i64::<BigEndian>()?;

        Ok(Long{
            value
        })
    }
}

// ConstantDouble --------------------------------------------------------------

#[derive(Debug)]
pub struct Double{
    value: f64,
}

impl ReadOne for Double{
    fn read_one<R: ReadBytesExt>(reader: &mut R, _: &EmptyContext) -> Result<Self, ClassLoadError> {
        let value = reader.read_f64::<BigEndian>()?;

        Ok(Double{
            value
        })
    }
}

// ConstantNameAndType ---------------------------------------------------------

#[derive(Debug)]
pub struct NameAndType{
    name_index: u16,
    descriptor_index: u16,
}

impl ReadOne for NameAndType{
    fn read_one<R: ReadBytesExt>(reader: &mut R, _: &EmptyContext) -> Result<Self, ClassLoadError> {
        let name_index = reader.read_u16::<BigEndian>()?;
        let descriptor_index = reader.read_u16::<BigEndian>()?;

        Ok(NameAndType{
            name_index,
            descriptor_index,
        })
    }
}

// ConstantUtf8 ----------------------------------------------------------------

#[derive(Debug)]
pub struct Utf8{
    pub bytes: Vec<u8>,
}

// impl Utf8{
//     fn str_length(bytes: &Vec<u8>) -> Result<usize, ClassLoadError> {
//         let mut size = 0;
//
//         let mut index = 0;
//         while index < bytes.len() {
//             let byte = bytes[index];
//
//             match byte {
//                 _ if byte >= 0xED => {
//                     size += 1;
//                     index += 6;
//                 },
//                 _ if byte >= 0xE0 => {
//                     size += 1;
//                     index += 3;
//                 }
//                 _ if byte >= 0x80 => {
//                     size += 1;
//                     index += 2;
//                 }
//                 _ => {
//                     size += 1;
//                     index += 1;
//                 }
//             }
//         }
//
//         if index > bytes.len() {
//             Err(ClassLoadError::new("String length computation error"))
//         } else {
//             Ok(size)
//         }
//     }
//
//     fn convert_bytes(bytes: &Vec<u8>) -> Result<String, ClassLoadError> {
//         let mut index = 0;
//         let string = String::from_utf8(bytes.to_vec())?;
//
//         let size = Self::str_length(&bytes);
//         if string.len() != bytes.len() {
//             println!("String disrepancy");
//             // if string.len() != size {
//             //     println!("Wrong estimation: {} != {}", string.len(), size);
//             // }
//         }
//
//         return Ok(string);
//     }
// }

impl ReadOne for Utf8{
    fn read_one<R: ReadBytesExt>(reader: &mut R, _: &EmptyContext) -> Result<Self, ClassLoadError> {
        let length = reader.read_u16::<BigEndian>()?;

        let mut bytes: Vec<u8> = vec![0; length as usize];
        reader.read_exact(&mut bytes)?;

        Ok(Utf8{
            bytes
        })
    }
}

// ConstantMethodHandle --------------------------------------------------------

#[derive(Debug)]
pub struct MethodHandle{
    reference_kind: u8,
    reference_index: u16,
}

impl ReadOne for MethodHandle{
    fn read_one<R: ReadBytesExt>(reader: &mut R, _: &EmptyContext) -> Result<Self, ClassLoadError> {
        let reference_kind = reader.read_u8()?;
        let reference_index = reader.read_u16::<BigEndian>()?;

        Ok(MethodHandle{
            reference_kind,
            reference_index,
        })
    }
}

// ConstantMethodType ----------------------------------------------------------

#[derive(Debug)]
pub struct MethodType{
    descriptor_index: u16,
}

impl ReadOne for MethodType{
    fn read_one<R: ReadBytesExt>(reader: &mut R, _: &EmptyContext) -> Result<Self, ClassLoadError> {
        let descriptor_index = reader.read_u16::<BigEndian>()?;
        Ok(MethodType{
            descriptor_index
        })
    }
}

// ConstantInvokeDynamic -------------------------------------------------------

#[derive(Debug)]
pub struct InvokeDynamic{
    bootstrap_method_attr_index: u16,
    name_and_type_index: u16,
}

impl ReadOne for InvokeDynamic{
    fn read_one<R: ReadBytesExt>(reader: &mut R, _: &EmptyContext) -> Result<Self, ClassLoadError> {
        let bootstrap_method_attr_index = reader.read_u16::<BigEndian>()?;
        let name_and_type_index = reader.read_u16::<BigEndian>()?;

        Ok(InvokeDynamic{
            bootstrap_method_attr_index,
            name_and_type_index,
        })
    }
}

// Module Constant -------------------------------------------------------------

#[derive(Debug)]
pub struct Module{
    name_index: u16,
}

impl ReadOne for Module{
    fn read_one<R: ReadBytesExt>(reader: &mut R, _: &EmptyContext) -> Result<Self, ClassLoadError> {
        let name_index = reader.read_u16::<BigEndian>()?;

        Ok(Module{
            name_index
        })
    }
}

// Package Constant ------------------------------------------------------------

#[derive(Debug)]
pub struct Package{
    name_index: u16,
}

impl ReadOne for Package{
    fn read_one<R: ReadBytesExt>(reader: &mut R, _: &EmptyContext) -> Result<Self, ClassLoadError> {
        let name_index = reader.read_u16::<BigEndian>()?;

        Ok(Package{
            name_index
        })
    }
}

// Constant --------------------------------------------------------------------

pub struct Skip<T> {
    value: T,
    skip: usize
}

#[derive(Debug)]
pub enum Constant {
    Utf8(Utf8),
    Integer(Integer),
    Float(Float),
    Long(Long),
    Double(Double),
    Class(Class),
    String(String),
    Field(ClassReference),
    Method(ClassReference),
    InterfaceMethod(ClassReference),
    NameAndType(NameAndType),
    MethodHandle(MethodHandle),
    MethodType(MethodType),
    InvokeDynamic(InvokeDynamic),
    Module(Module),
    Package(Package),
}

impl ReadOne for Constant {
    fn read_one<R: ReadBytesExt>(reader: &mut R, _: &EmptyContext) -> Result<Self, ClassLoadError> {
        let tag = reader.read_u8()?;

        let context = EmptyContext::default();
        let constant = match tag {
            1 => Ok(Constant::Utf8(Utf8::read_one(reader, &context)?)),
            3 => Ok(Constant::Integer(Integer::read_one(reader, &context)?)),
            4 => Ok(Constant::Float(Float::read_one(reader, &context)?)),
            5 => Ok(Constant::Long(Long::read_one(reader, &context)?)),
            6 => Ok(Constant::Double(Double::read_one(reader, &context)?)),
            7 => Ok(Constant::Class(Class::read_one(reader, &context)?)),
            8 => Ok(Constant::String(String::read_one(reader, &context)?)),
            9 => Ok(Constant::Field(ClassReference::read_one(reader, &context)?)),
            10 => Ok(Constant::Method(ClassReference::read_one(reader, &context)?)),
            11 => Ok(Constant::InterfaceMethod(ClassReference::read_one(reader, &context)?)),
            12 => Ok(Constant::NameAndType(NameAndType::read_one(reader, &context)?)),
            15 => Ok(Constant::MethodHandle(MethodHandle::read_one(reader, &context)?)),
            16 => Ok(Constant::MethodType(MethodType::read_one(reader, &context)?)),
            18 => Ok(Constant::InvokeDynamic(InvokeDynamic::read_one(reader, &context)?)),
            19 => Ok(Constant::Module(Module::read_one(reader, &context)?)),
            20 => Ok(Constant::Package(Package::read_one(reader, &context)?)),
            _ => Err(ClassLoadError::new("Cannot match constant tag"))
        }?;
        Ok(constant)
    }
}

impl ReadAll for Constant {
    fn skip_amount(element: &Constant) -> usize {
        return match *element {
            Constant::Long(_) | Constant::Double(_) => 1,
            _ => 0,
        }
    }
}

// Constant Pool ---------------------------------------------------------------

#[derive(Debug)]
pub struct ConstantPool {
    constants: Vec<Constant>,
    skip_table: Vec<usize>
}

impl ConstantPool {
    fn assemble_skip_table(constants: &Vec<Constant>) -> Vec<usize> {
        let mut skip_table = Vec::new();
        for (i, value) in constants.iter().enumerate() {
            match *value {
                Constant::Long(_) | Constant::Double(_) => skip_table.push(i),
                _ => {}
            }
        }

        return skip_table;
    }
}

impl ReadOne for ConstantPool {
    fn read_one<R: ReadBytesExt>(reader: &mut R, context: &EmptyContext) -> Result<Self, ClassLoadError> {
        let constants = Constant::read_all_from(reader, context, 1)?;
        let mut skip_table = ConstantPool::assemble_skip_table(&constants);

        Ok(ConstantPool {
            constants,
            skip_table
        })
    }
}

impl Index<usize> for ConstantPool {
    type Output = Constant;

    fn index(&self, index: usize) -> &Self::Output {
        let vec_index = (index - 1) as usize;

        let skips: usize = self
            .skip_table
            .iter()
            .filter(|x| x < &&vec_index)
            .sum();
        let skipped_index = vec_index - skips;

        return &self.constants[skipped_index];
    }
}

impl Index<u16> for ConstantPool {
    type Output = Constant;

    fn index(&self, index: u16) -> &Self::Output {
        let index = index as usize;
        return ConstantPool::index(self, index)
    }
}

// ============================================================================
// CONSTANT POOL TESTS
// ============================================================================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conversion() {
        let bytes = vec!(
            0x0f,
            0x0f
        );

        let len = Utf8Constant::str_length(&bytes);

    }
}