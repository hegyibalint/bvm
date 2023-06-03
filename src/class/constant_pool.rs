use std::fmt::Debug;
use std::ops::Index;

use byteorder::{BigEndian, ReadBytesExt};

use crate::class::{ClassLoadingError, EmptyContext, ReadAll, ReadOne};

// =============================================================================
// CONTEXT
// =============================================================================

pub struct ConstantPoolContext<'a> {
    pub constant_pool: &'a ConstantPool,
}

impl<'a> ConstantPoolContext<'a> {
    pub fn new(constant_pool: &'a ConstantPool) -> ConstantPoolContext {
        ConstantPoolContext { constant_pool }
    }
}

// =============================================================================
// CONSTANT POOL
// =============================================================================

// ConstantClass ---------------------------------------------------------------

#[derive(Debug)]
pub struct ConstClass {
    name_index: u16,
}

impl ReadOne for ConstClass {
    fn read_one<R: ReadBytesExt>(
        reader: &mut R,
        _: &EmptyContext,
    ) -> Result<Self, ClassLoadingError> {
        let name_index = reader.read_u16::<BigEndian>()?;

        Ok(ConstClass { name_index })
    }
}

// ReferenceConstant -----------------------------------------------------------
// Covers:
//  - Field
//  - Method
//  - InterfaceMethod

#[derive(Debug)]
pub struct ConstClassReference {
    class_index: u16,
    name_and_type_index: u16,
}

impl ReadOne for ConstClassReference {
    fn read_one<R: ReadBytesExt>(
        reader: &mut R,
        _: &EmptyContext,
    ) -> Result<Self, ClassLoadingError> {
        let class_index = reader.read_u16::<BigEndian>()?;
        let name_and_type_index = reader.read_u16::<BigEndian>()?;

        Ok(ConstClassReference {
            class_index,
            name_and_type_index,
        })
    }
}

// ConstantString --------------------------------------------------------------

#[derive(Debug)]
pub struct ConstString {
    string_index: u16,
}

impl ReadOne for ConstString {
    fn read_one<R: ReadBytesExt>(
        reader: &mut R,
        _: &EmptyContext,
    ) -> Result<Self, ClassLoadingError> {
        let string_index = reader.read_u16::<BigEndian>()?;

        Ok(ConstString { string_index })
    }
}

// ConstantInteger -------------------------------------------------------------

#[derive(Debug)]
pub struct ConstInteger {
    value: i32,
}

impl ReadOne for ConstInteger {
    fn read_one<R: ReadBytesExt>(
        reader: &mut R,
        _: &EmptyContext,
    ) -> Result<Self, ClassLoadingError> {
        let value = reader.read_i32::<BigEndian>()?;

        Ok(ConstInteger { value })
    }
}

// ConstantFloat ---------------------------------------------------------------

#[derive(Debug)]
pub struct ConstFloat {
    value: f32,
}

impl ReadOne for ConstFloat {
    fn read_one<R: ReadBytesExt>(
        reader: &mut R,
        _: &EmptyContext,
    ) -> Result<Self, ClassLoadingError> {
        let value = reader.read_f32::<BigEndian>()?;

        Ok(ConstFloat { value })
    }
}

// ConstantLong ----------------------------------------------------------------

#[derive(Debug)]
pub struct ConstLong {
    value: i64,
}

impl ReadOne for ConstLong {
    fn read_one<R: ReadBytesExt>(
        reader: &mut R,
        _: &EmptyContext,
    ) -> Result<Self, ClassLoadingError> {
        let value = reader.read_i64::<BigEndian>()?;

        Ok(ConstLong { value })
    }
}

// ConstantDouble --------------------------------------------------------------

#[derive(Debug)]
pub struct ConstDouble {
    value: f64,
}

impl ReadOne for ConstDouble {
    fn read_one<R: ReadBytesExt>(
        reader: &mut R,
        _: &EmptyContext,
    ) -> Result<Self, ClassLoadingError> {
        let value = reader.read_f64::<BigEndian>()?;

        Ok(ConstDouble { value })
    }
}

// ConstantNameAndType ---------------------------------------------------------

#[derive(Debug)]
pub struct ConstNameAndType {
    name_index: u16,
    descriptor_index: u16,
}

impl ReadOne for ConstNameAndType {
    fn read_one<R: ReadBytesExt>(
        reader: &mut R,
        _: &EmptyContext,
    ) -> Result<Self, ClassLoadingError> {
        let name_index = reader.read_u16::<BigEndian>()?;
        let descriptor_index = reader.read_u16::<BigEndian>()?;

        Ok(ConstNameAndType {
            name_index,
            descriptor_index,
        })
    }
}

// ConstantUtf8 ----------------------------------------------------------------

#[derive(Debug)]
pub struct ConstUtf8 {
    pub string: String,
}

impl ConstUtf8 {
    fn str_length(bytes: &Vec<u8>) -> Result<usize, ClassLoadingError> {
        let mut size = 0;
        let mut index = 0;
        while index < bytes.len() {
            let byte = bytes[index];

            match byte {
                _ if byte >= 0xED => {
                    size += 1;
                    index += 6;
                }
                _ if byte >= 0xE0 => {
                    size += 1;
                    index += 3;
                }
                _ if byte >= 0x80 => {
                    size += 1;
                    index += 2;
                }
                _ => {
                    size += 1;
                    index += 1;
                }
            }
        }

        if index > bytes.len() {
            Err(ClassLoadingError::new(
                "String length computation error, index overran the length of the string",
            ))
        } else {
            Ok(size)
        }
    }

    // fn convert_bytes(bytes: &Vec<u8>) -> Result<String, ClassLoadError> {
    //     let length = Self::str_length(bytes)?;
    //     let mut string = String::with_capacity(length);
    //
    //     return Ok(string);
    // }
}

impl ReadOne for ConstUtf8 {
    fn read_one<R: ReadBytesExt>(
        reader: &mut R,
        _: &EmptyContext,
    ) -> Result<Self, ClassLoadingError> {
        let length = reader.read_u16::<BigEndian>()?;

        let mut bytes: Vec<u8> = vec![0; length as usize];
        reader.read_exact(&mut bytes)?;
        // let string = Self::convert_bytes(&bytes)?;
        let string = String::from_utf8(bytes)?;

        Ok(ConstUtf8 { string })
    }
}

// ConstantMethodHandle --------------------------------------------------------

#[derive(Debug)]
pub struct ConstMethodHandle {
    reference_kind: u8,
    reference_index: u16,
}

impl ReadOne for ConstMethodHandle {
    fn read_one<R: ReadBytesExt>(
        reader: &mut R,
        _: &EmptyContext,
    ) -> Result<Self, ClassLoadingError> {
        let reference_kind = reader.read_u8()?;
        let reference_index = reader.read_u16::<BigEndian>()?;

        Ok(ConstMethodHandle {
            reference_kind,
            reference_index,
        })
    }
}

// ConstantMethodType ----------------------------------------------------------

#[derive(Debug)]
pub struct ConstMethodType {
    descriptor_index: u16,
}

impl ReadOne for ConstMethodType {
    fn read_one<R: ReadBytesExt>(
        reader: &mut R,
        _: &EmptyContext,
    ) -> Result<Self, ClassLoadingError> {
        let descriptor_index = reader.read_u16::<BigEndian>()?;
        Ok(ConstMethodType { descriptor_index })
    }
}

// ConstantInvokeDynamic -------------------------------------------------------

#[derive(Debug)]
pub struct ConstInvokeDynamic {
    bootstrap_method_attr_index: u16,
    name_and_type_index: u16,
}

impl ReadOne for ConstInvokeDynamic {
    fn read_one<R: ReadBytesExt>(
        reader: &mut R,
        _: &EmptyContext,
    ) -> Result<Self, ClassLoadingError> {
        let bootstrap_method_attr_index = reader.read_u16::<BigEndian>()?;
        let name_and_type_index = reader.read_u16::<BigEndian>()?;

        Ok(ConstInvokeDynamic {
            bootstrap_method_attr_index,
            name_and_type_index,
        })
    }
}

// Constant --------------------------------------------------------------------

pub struct Skip<T> {
    value: T,
    skip: usize,
}

#[derive(Debug)]
pub enum Constant {
    Utf8(ConstUtf8),
    Integer(ConstInteger),
    Float(ConstFloat),
    Long(ConstLong),
    Double(ConstDouble),
    Class(ConstClass),
    String(ConstString),
    Field(ConstClassReference),
    Method(ConstClassReference),
    InterfaceMethod(ConstClassReference),
    NameAndType(ConstNameAndType),
    MethodHandle(ConstMethodHandle),
    MethodType(ConstMethodType),
    InvokeDynamic(ConstInvokeDynamic),
}

impl ReadOne for Constant {
    fn read_one<R: ReadBytesExt>(
        reader: &mut R,
        _: &EmptyContext,
    ) -> Result<Self, ClassLoadingError> {
        let tag = reader.read_u8()?;

        let context = EmptyContext::default();
        let constant = match tag {
            1 => Ok(Constant::Utf8(ConstUtf8::read_one(reader, &context)?)),
            3 => Ok(Constant::Integer(ConstInteger::read_one(reader, &context)?)),
            4 => Ok(Constant::Float(ConstFloat::read_one(reader, &context)?)),
            5 => Ok(Constant::Long(ConstLong::read_one(reader, &context)?)),
            6 => Ok(Constant::Double(ConstDouble::read_one(reader, &context)?)),
            7 => Ok(Constant::Class(ConstClass::read_one(reader, &context)?)),
            8 => Ok(Constant::String(ConstString::read_one(reader, &context)?)),
            9 => Ok(Constant::Field(ConstClassReference::read_one(
                reader, &context,
            )?)),
            10 => Ok(Constant::Method(ConstClassReference::read_one(
                reader, &context,
            )?)),
            11 => Ok(Constant::InterfaceMethod(ConstClassReference::read_one(
                reader, &context,
            )?)),
            12 => Ok(Constant::NameAndType(ConstNameAndType::read_one(
                reader, &context,
            )?)),
            15 => Ok(Constant::MethodHandle(ConstMethodHandle::read_one(
                reader, &context,
            )?)),
            16 => Ok(Constant::MethodType(ConstMethodType::read_one(
                reader, &context,
            )?)),
            18 => Ok(Constant::InvokeDynamic(ConstInvokeDynamic::read_one(
                reader, &context,
            )?)),
            _ => Err(ClassLoadingError::new("Cannot match constant tag")),
        }?;
        Ok(constant)
    }
}

impl ReadAll for Constant {
    fn skip_amount(element: &Constant) -> usize {
        return match *element {
            Constant::Long(_) | Constant::Double(_) => 1,
            _ => 0,
        };
    }
}

// Constant Pool ---------------------------------------------------------------

#[derive(Debug)]
pub struct ConstantPool {
    constants: Vec<Constant>,
    skip_table: Vec<usize>,
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
    fn read_one<R: ReadBytesExt>(
        reader: &mut R,
        context: &EmptyContext,
    ) -> Result<Self, ClassLoadingError> {
        let constants = Constant::read_all_from(reader, context, 1)?;
        let mut skip_table = ConstantPool::assemble_skip_table(&constants);

        Ok(ConstantPool {
            constants,
            skip_table,
        })
    }
}

impl Index<usize> for ConstantPool {
    type Output = Constant;

    fn index(&self, index: usize) -> &Self::Output {
        let vec_index = (index - 1) as usize;

        let skips: usize = self.skip_table.iter().filter(|x| x < &&vec_index).count();
        let skipped_index = vec_index - skips;

        return &self.constants[skipped_index];
    }
}

impl Index<u16> for ConstantPool {
    type Output = Constant;

    fn index(&self, index: u16) -> &Self::Output {
        let index = index as usize;
        return ConstantPool::index(self, index);
    }
}

// ============================================================================
// CONSTANT POOL TESTS
// ============================================================================

#[cfg(test)]
mod const_utf8_tests {
    use super::ConstUtf8;

    #[test]
    fn test_conversion() {
        let bytes = vec![0x0f, 0x0f];
        let len = ConstUtf8::str_length(&bytes);

        assert_eq!(len,)
    }
}
