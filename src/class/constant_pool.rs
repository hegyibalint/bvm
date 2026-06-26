use std::fmt;
use std::fmt::{Debug, Display, Formatter};
use std::ops::Index;

use byteorder::{BigEndian, ReadBytesExt};
use clap::builder::Str;

use crate::class::attributes::BootstrapMethodAttribute;
use crate::class::{
    ClassParsingError, ConstantIndexingError, EmptyContext, FieldInfo, MethodInfo, ReadAll, ReadOne,
};

// =============================================================================
// TYPES
// =============================================================================

type ResolvedResult<T> = Result<T, ConstantIndexingError>;

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
// TRAITS
// =============================================================================

pub trait Resolvable<'c, C, R> {
    fn resolve(&self, context: &'c C) -> ResolvedResult<R>;
}

// =============================================================================
// CONSTANT POOL
// =============================================================================

// ConstantClass ---------------------------------------------------------------

#[derive(Debug)]
pub struct ConstClass {
    pub name_index: u16,
}

impl ReadOne for ConstClass {
    fn read_one<R: ReadBytesExt>(
        reader: &mut R,
        _: &EmptyContext,
    ) -> Result<Self, ClassParsingError> {
        let name_index = reader.read_u16::<BigEndian>()?;

        Ok(ConstClass { name_index })
    }
}

#[derive(Debug)]
pub struct ResolvedConstClass<'c> {
    pub name: ResolvedResult<&'c String>,
}

impl<'c> Resolvable<'c, ConstantPoolContext<'c>, ResolvedConstClass<'c>> for ConstClass {
    fn resolve(&self, context: &ConstantPoolContext<'c>) -> ResolvedResult<ResolvedConstClass<'c>> {
        match &context.constant_pool[self.name_index] {
            Constant::Utf8(utf8) => Ok(ResolvedConstClass {
                name: Ok(&utf8.string),
            }),
            ref value => Err(ConstantIndexingError::InvalidReference(
                "Utf8".to_string(),
                value.to_string(),
            )),
        }
    }
}

// ReferenceConstant -----------------------------------------------------------

#[derive(Debug)]
pub struct ConstClassReference {
    pub class_index: u16,
    pub name_and_type_index: u16,
}

impl ReadOne for ConstClassReference {
    fn read_one<R: ReadBytesExt>(
        reader: &mut R,
        _: &EmptyContext,
    ) -> Result<Self, ClassParsingError> {
        let class_index = reader.read_u16::<BigEndian>()?;
        let name_and_type_index = reader.read_u16::<BigEndian>()?;

        Ok(ConstClassReference {
            class_index,
            name_and_type_index,
        })
    }
}

#[derive(Debug)]
pub struct ResolvedConstClassReference<'c> {
    pub class: ResolvedResult<ResolvedConstClass<'c>>,
    pub name_and_type: ResolvedResult<ResolvedConstNameAndType<'c>>,
}

impl<'c> Resolvable<'c, ConstantPoolContext<'c>, ResolvedConstClassReference<'c>>
    for ConstClassReference
{
    fn resolve(
        &self,
        context: &'c ConstantPoolContext,
    ) -> ResolvedResult<ResolvedConstClassReference<'c>> {
        let class = match &context.constant_pool[self.class_index] {
            Constant::Class(class) => class.resolve(context),
            value => Err(ConstantIndexingError::InvalidReference(
                "Class".to_string(),
                value.to_string(),
            )),
        };
        let name_and_type = match &context.constant_pool[self.class_index] {
            Constant::NameAndType(name_and_type) => name_and_type.resolve(context),
            value => Err(ConstantIndexingError::InvalidReference(
                "NameAndType".to_string(),
                value.to_string(),
            )),
        };

        Ok(ResolvedConstClassReference {
            class,
            name_and_type,
        })
    }
}

// ConstantString --------------------------------------------------------------

#[derive(Debug)]
pub struct ConstString {
    pub string_index: u16,
}

impl ReadOne for ConstString {
    fn read_one<R: ReadBytesExt>(
        reader: &mut R,
        _: &EmptyContext,
    ) -> Result<Self, ClassParsingError> {
        let string_index = reader.read_u16::<BigEndian>()?;

        Ok(ConstString { string_index })
    }
}

#[derive(Debug)]
pub struct ResolvedConstString<'c> {
    pub string: ResolvedResult<&'c String>,
}

impl<'c> Resolvable<'c, ConstantPoolContext<'c>, ResolvedConstString<'c>> for ConstString {
    fn resolve(
        &self,
        context: &ConstantPoolContext<'c>,
    ) -> ResolvedResult<ResolvedConstString<'c>> {
        match &context.constant_pool[self.string_index] {
            Constant::Utf8(utf8) => Ok(ResolvedConstString {
                string: Ok(&utf8.string),
            }),
            value => Err(ConstantIndexingError::InvalidReference(
                "Utf8".to_string(),
                value.to_string(),
            )),
        }
    }
}

// ConstantInteger -------------------------------------------------------------

#[derive(Debug)]
pub struct ConstInteger {
    pub value: i32,
}

impl ReadOne for ConstInteger {
    fn read_one<R: ReadBytesExt>(
        reader: &mut R,
        _: &EmptyContext,
    ) -> Result<Self, ClassParsingError> {
        let value = reader.read_i32::<BigEndian>()?;

        Ok(ConstInteger { value })
    }
}

// ConstantFloat ---------------------------------------------------------------

#[derive(Debug)]
pub struct ConstFloat {
    pub value: f32,
}

impl ReadOne for ConstFloat {
    fn read_one<R: ReadBytesExt>(
        reader: &mut R,
        _: &EmptyContext,
    ) -> Result<Self, ClassParsingError> {
        let value = reader.read_f32::<BigEndian>()?;

        Ok(ConstFloat { value })
    }
}

// ConstantLong ----------------------------------------------------------------

#[derive(Debug)]
pub struct ConstLong {
    pub value: i64,
}

impl ReadOne for ConstLong {
    fn read_one<R: ReadBytesExt>(
        reader: &mut R,
        _: &EmptyContext,
    ) -> Result<Self, ClassParsingError> {
        let value = reader.read_i64::<BigEndian>()?;

        Ok(ConstLong { value })
    }
}

// ConstantDouble --------------------------------------------------------------

#[derive(Debug)]
pub struct ConstDouble {
    pub value: f64,
}

impl ReadOne for ConstDouble {
    fn read_one<R: ReadBytesExt>(
        reader: &mut R,
        _: &EmptyContext,
    ) -> Result<Self, ClassParsingError> {
        let value = reader.read_f64::<BigEndian>()?;

        Ok(ConstDouble { value })
    }
}

// ConstantNameAndType ---------------------------------------------------------

#[derive(Debug)]
pub struct ConstNameAndType {
    pub name_index: u16,
    pub descriptor_index: u16,
}

impl ReadOne for ConstNameAndType {
    fn read_one<R: ReadBytesExt>(
        reader: &mut R,
        _: &EmptyContext,
    ) -> Result<Self, ClassParsingError> {
        let name_index = reader.read_u16::<BigEndian>()?;
        let descriptor_index = reader.read_u16::<BigEndian>()?;

        Ok(ConstNameAndType {
            name_index,
            descriptor_index,
        })
    }
}

#[derive(Debug)]
pub struct ResolvedConstNameAndType<'c> {
    pub name: ResolvedResult<&'c String>,
    pub type_name: ResolvedResult<&'c String>,
}

impl<'c> Resolvable<'c, ConstantPoolContext<'c>, ResolvedConstNameAndType<'c>>
    for ConstNameAndType
{
    fn resolve(
        &self,
        context: &'c ConstantPoolContext,
    ) -> Result<ResolvedConstNameAndType<'c>, ConstantIndexingError> {
        let name = context.constant_pool.get_utf8(self.name_index);
        let type_name = context.constant_pool.get_utf8(self.descriptor_index);

        Ok(ResolvedConstNameAndType { name, type_name })
    }
}

// ConstantUtf8 ----------------------------------------------------------------

#[derive(Debug)]
pub struct ConstUtf8 {
    pub string: String,
}

impl ConstUtf8 {
    fn str_length(bytes: &Vec<u8>) -> Result<usize, ClassParsingError> {
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
            Err(ClassParsingError::new(
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
    ) -> Result<Self, ClassParsingError> {
        let length = reader.read_u16::<BigEndian>()?;

        let mut bytes: Vec<u8> = vec![0; length as usize];
        reader.read_exact(&mut bytes)?;
        // let string = Self::convert_bytes(&bytes)?;
        let string = String::from_utf8(bytes)?;

        Ok(ConstUtf8 { string })
    }
}

// ConstantMethodHandle --------------------------------------------------------

#[derive(Debug, Clone)]
pub enum ReferenceKind {
    GetField,
    GetStatic,
    PutField,
    PutStatic,
    InvokeVirtual,
    InvokeStatic,
    InvokeSpecial,
    NewInvokeSpecial,
    InvokeInterface,
}

#[derive(Debug)]
pub struct ConstMethodHandle {
    pub reference_kind: ReferenceKind,
    pub reference_index: u16,
}

impl ReadOne for ConstMethodHandle {
    fn read_one<R: ReadBytesExt>(
        reader: &mut R,
        _: &EmptyContext,
    ) -> Result<Self, ClassParsingError> {
        let reference_kind = match reader.read_u8()? {
            1 => ReferenceKind::GetField,
            2 => ReferenceKind::GetStatic,
            3 => ReferenceKind::PutField,
            4 => ReferenceKind::PutStatic,
            5 => ReferenceKind::InvokeVirtual,
            6 => ReferenceKind::InvokeStatic,
            7 => ReferenceKind::InvokeSpecial,
            8 => ReferenceKind::NewInvokeSpecial,
            9 => ReferenceKind::InvokeInterface,
            _ => {
                return Err(ClassParsingError::new(
                    "Invalid reference kind in ConstantMethodHandle",
                ))
            }
        };
        let reference_index = reader.read_u16::<BigEndian>()?;

        Ok(ConstMethodHandle {
            reference_kind,
            reference_index,
        })
    }
}

#[derive(Debug)]
pub enum ResolvedMethodReference<'c> {
    Field(ResolvedResult<&'c ConstClassReference>),
    Method(ResolvedResult<&'c ConstClassReference>),
    InterfaceMethod(ResolvedResult<&'c ConstClassReference>),
}

#[derive(Debug)]
pub struct ResolvedConstMethodHandle<'c> {
    pub reference_kind: ReferenceKind,
    pub reference: ResolvedResult<ResolvedMethodReference<'c>>,
}

impl<'c> Resolvable<'c, ConstantPoolContext<'c>, ResolvedConstMethodHandle<'c>>
    for ConstMethodHandle
{
    fn resolve(
        &self,
        context: &'c ConstantPoolContext,
    ) -> ResolvedResult<ResolvedConstMethodHandle<'c>> {
        let reference = match self.reference_kind {
            ReferenceKind::GetField
            | ReferenceKind::GetStatic
            | ReferenceKind::PutField
            | ReferenceKind::PutStatic => {
                let field = context
                    .constant_pool
                    .get_class_reference(self.reference_index);
                ResolvedMethodReference::Field(field)
            }
            ReferenceKind::InvokeVirtual | ReferenceKind::NewInvokeSpecial => {
                let method = context
                    .constant_pool
                    .get_class_reference(self.reference_index);
                ResolvedMethodReference::Method(method)
            }
            // TODO: Handle bytecode version 52.0 difference described in 4.4.8.
            ReferenceKind::InvokeStatic | ReferenceKind::InvokeSpecial => {
                let method = context
                    .constant_pool
                    .get_class_reference(self.reference_index);
                ResolvedMethodReference::Method(method)
            }
            ReferenceKind::InvokeInterface => {
                let method = context
                    .constant_pool
                    .get_class_reference(self.reference_index);
                ResolvedMethodReference::InterfaceMethod(method)
            }
        };

        Ok(ResolvedConstMethodHandle {
            reference_kind: self.reference_kind.clone(),
            reference: Ok(reference),
        })
    }
}

// ConstantMethodType ----------------------------------------------------------

#[derive(Debug)]
pub struct ConstMethodType {
    pub descriptor_index: u16,
}

impl ReadOne for ConstMethodType {
    fn read_one<R: ReadBytesExt>(
        reader: &mut R,
        _: &EmptyContext,
    ) -> Result<Self, ClassParsingError> {
        let descriptor_index = reader.read_u16::<BigEndian>()?;
        Ok(ConstMethodType { descriptor_index })
    }
}

#[derive(Debug)]
pub struct ResolvedConstMethodType<'c> {
    pub descriptor: ResolvedResult<&'c String>,
}

impl<'c> Resolvable<'c, ConstantPoolContext<'c>, ResolvedConstMethodType<'c>> for ConstMethodType {
    fn resolve(
        &self,
        context: &'c ConstantPoolContext,
    ) -> ResolvedResult<ResolvedConstMethodType<'c>> {
        let descriptor = context.constant_pool.get_utf8(self.descriptor_index);
        Ok(ResolvedConstMethodType { descriptor })
    }
}

// ConstantInvokeDynamic -------------------------------------------------------

#[derive(Debug)]
pub struct ConstInvokeDynamic {
    pub bootstrap_method_attr_index: u16,
    pub name_and_type_index: u16,
}

pub struct ResolvedInvokeDynamic<'a> {
    pub bootstrap_method: &'a BootstrapMethodAttribute,
    pub name_and_type: &'a ConstNameAndType,
}

impl ReadOne for ConstInvokeDynamic {
    fn read_one<R: ReadBytesExt>(
        reader: &mut R,
        _: &EmptyContext,
    ) -> Result<Self, ClassParsingError> {
        let bootstrap_method_attr_index = reader.read_u16::<BigEndian>()?;
        let name_and_type_index = reader.read_u16::<BigEndian>()?;

        Ok(ConstInvokeDynamic {
            bootstrap_method_attr_index,
            name_and_type_index,
        })
    }
}

// Constant --------------------------------------------------------------------

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
    ) -> Result<Self, ClassParsingError> {
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
            _ => Err(ClassParsingError::new("Cannot match constant tag")),
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

impl Display for Constant {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Constant::Utf8(_) => write!(f, "Utf8"),
            Constant::Integer(_) => write!(f, "Integer"),
            Constant::Float(_) => write!(f, "Float"),
            Constant::Long(_) => write!(f, "Long"),
            Constant::Double(_) => write!(f, "Double"),
            Constant::Class(_) => write!(f, "Class"),
            Constant::String(_) => write!(f, "String"),
            Constant::Field(_) => write!(f, "Field"),
            Constant::Method(_) => write!(f, "Method"),
            Constant::InterfaceMethod(_) => write!(f, "InterfaceMethod"),
            Constant::NameAndType(_) => write!(f, "NameAndType"),
            Constant::MethodHandle(_) => write!(f, "MethodHandle"),
            Constant::MethodType(_) => write!(f, "MethodType"),
            Constant::InvokeDynamic(_) => write!(f, "InvokeDynamic"),
        }
    }
}

// Constant Pool ---------------------------------------------------------------

#[derive(Debug)]
pub struct ConstantPool {
    constants: Vec<Constant>,
    skip_table: Vec<usize>,
}

impl ConstantPool {
    pub fn iter(&self) -> impl Iterator<Item = &Constant> {
        self.constants.iter()
    }

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

    pub fn get_utf8(&self, index: u16) -> ResolvedResult<&String> {
        let utf8 = match &self[index] {
            Constant::Utf8(ref utf8) => Ok(utf8),
            constant => Err(ConstantIndexingError::InvalidReference(
                "Constant".to_string(),
                constant.to_string(),
            )),
        }?;

        return Ok(&utf8.string);
    }

    pub fn get_class_name(&self, index: u16) -> ResolvedResult<&String> {
        match &self[index] {
            Constant::Class(ref class) => self.get_utf8(class.name_index),
            constant => Err(ConstantIndexingError::InvalidReference(
                "Class".to_string(),
                constant.to_string(),
            )),
        }
    }

    pub fn get_string(&self, index: u16) -> ResolvedResult<&String> {
        match &self[index] {
            Constant::String(ref string) => self.get_utf8(string.string_index),
            constant => Err(ConstantIndexingError::InvalidReference(
                "String".to_string(),
                constant.to_string(),
            )),
        }
    }

    pub fn get_class_reference(&self, index: u16) -> ResolvedResult<&ConstClassReference> {
        match &self[index] {
            Constant::Field(field) => Ok(&field),
            constant => Err(ConstantIndexingError::InvalidReference(
                "Field".to_string(),
                constant.to_string(),
            )),
        }
    }

    fn get_method_reference(&self, index: u16) -> ResolvedResult<&ConstMethodHandle> {
        match &self[index] {
            Constant::MethodHandle(method_handle) => Ok(&method_handle),
            constant => Err(ConstantIndexingError::InvalidReference(
                "MethodHandle".to_string(),
                constant.to_string(),
            )),
        }
    }
}

impl ReadOne for ConstantPool {
    fn read_one<R: ReadBytesExt>(
        reader: &mut R,
        context: &EmptyContext,
    ) -> Result<Self, ClassParsingError> {
        let constants = Constant::read_all_from(reader, context, 1)?;
        let skip_table = ConstantPool::assemble_skip_table(&constants);

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
