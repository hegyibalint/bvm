use std::{fmt, io, string};
use std::error::Error;
use std::fmt::Debug;

use byteorder::{BigEndian, ReadBytesExt};

use crate::class::attributes::Attribute;
use crate::class::constant_pool::{ConstantPool, ConstantPoolContext};

pub mod attributes;
pub mod constant_pool;

// =============================================================================
// STATIC VALUES
// =============================================================================

static CLASS_MAGIC: u32 = 0xCAFEBABE;

// =============================================================================
// ERRORS
// =============================================================================

#[derive(Debug)]
pub struct ClassLoadError {
    details: String,
}

impl ClassLoadError {
    fn new(msg: &str) -> ClassLoadError {
        ClassLoadError { details: msg.to_string() }
    }
}

impl fmt::Display for ClassLoadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl Error for ClassLoadError {
    fn description(&self) -> &str {
        &self.details
    }
}

impl From<io::Error> for ClassLoadError {
    fn from(err: io::Error) -> Self {
        ClassLoadError::new(err.description())
    }
}

impl From<string::FromUtf8Error> for ClassLoadError {
    fn from(err: string::FromUtf8Error) -> Self {
        ClassLoadError::new(err.description())
    }
}

// =============================================================================
// CONTEXT
// =============================================================================

#[derive(Default)]
struct EmptyContext {}

// =============================================================================
// COMMON TRAITS
// =============================================================================

// TODO: Think about a struct/trait sharing the context with all modules

trait ReadOne<C = EmptyContext> where
    Self: Sized
{
    fn read_one<R: ReadBytesExt>(reader: &mut R, context: &C) -> Result<Self, ClassLoadError>;
}

trait ReadAll<C = EmptyContext> where
    Self: ReadOne<C>
{
    fn read_count<R: ReadBytesExt>(reader: &mut R) -> Result<usize, ClassLoadError> {
        let count = reader.read_u16::<BigEndian>()? as usize;
        Ok(count)
    }

    fn skip_amount(element: &Self) -> usize {
        return 0
    }

    fn read_all_from<R: ReadBytesExt>(reader: &mut R, context: &C, from: usize) -> Result<Vec<Self>, ClassLoadError> {
        let count = Self::read_count(reader)?;
        let mut elements = Vec::with_capacity(count);

        let mut index: usize = from;
        while index < count {
            let element = Self::read_one(reader, context)?;
            let skip = Self::skip_amount(&element);
            index += 1 + skip;
            elements.push(element);
        }

        Ok(elements)
    }

    fn read_all<R: ReadBytesExt>(reader: &mut R, context: &C) -> Result<Vec<Self>, ClassLoadError> {
        Self::read_all_from(reader, context, 0)
    }
}

// =============================================================================
// FIELDS
// =============================================================================

// Field Info ------------------------------------------------------------------

bitflags::bitflags! {
    struct FieldAccessFlags: u16 {
        const PUBLIC = 0x0001;
        const PRIVATE = 0x0002;
        const PROTECTED = 0x0004;
        const STATIC = 0x0008;
        const FINAL = 0x0010;
        const VOLATILE = 0x0040;
        const TRANSIENT = 0x0080;
        const SYNTHETIC = 0x1000;
        const ENUM = 0x4000;
    }
}

#[derive(Debug)]
pub struct FieldInfo {
    access_flags: FieldAccessFlags,
    name_index: u16,
    descriptor_index: u16,
    attributes: Vec<Attribute>,
}

impl ReadOne<ConstantPoolContext> for FieldInfo {
    fn read_one<R: ReadBytesExt>(reader: &mut R, context: &ConstantPoolContext) -> Result<Self, ClassLoadError> {
        let access_flags = reader.read_u16::<BigEndian>()?;
        let access_flags = FieldAccessFlags::from_bits(access_flags).ok_or(ClassLoadError::new("Invalid field access flags"))?;
        let name_index = reader.read_u16::<BigEndian>()?;
        let descriptor_index = reader.read_u16::<BigEndian>()?;
        let attributes = Attribute::read_all(reader, context)?;

        Ok(FieldInfo {
            access_flags,
            name_index,
            descriptor_index,
            attributes,
        })
    }
}

// =============================================================================
// METHODS
// =============================================================================

bitflags::bitflags! {
    struct MethodAccessFlags: u16 {
        const PUBLIC = 0x0001;
        const PRIVATE = 0x0002;
        const PROTECTED = 0x0004;
        const STATIC = 0x0008;
        const FINAL = 0x0010;
        const SYNCHRONIZED = 0x0020;
        const BRIDGE = 0x0040;
        const VARARGS = 0x0080;
        const NATIVE = 0x0100;
        const ABSTRACT = 0x0400;
        const STRICT = 0x0800;
        const SYNTHETIC = 0x1000;
    }
}

#[derive(Debug)]
pub struct MethodInfo {
    access_flags: MethodAccessFlags,
    name_index: u16,
    descriptor_index: u16,
    attributes: Vec<Attribute>,
}

impl ReadOne<ConstantPoolContext> for MethodInfo {
    fn read_one<R: ReadBytesExt>(reader: &mut R, context: &ConstantPoolContext) -> Result<Self, ClassLoadError> {
        let access_flags = reader.read_u16::<BigEndian>()?;
        let access_flags = MethodAccessFlags::from_bits(access_flags).ok_or(ClassLoadError::new("Invalid method access flags"))?;
        let name_index = reader.read_u16::<BigEndian>()?;
        let descriptor_index = reader.read_u16::<BigEndian>()?;
        let attributes = Attribute::read_all(reader, context)?;

        Ok(MethodInfo {
            access_flags,
            name_index,
            descriptor_index,
            attributes,
        })
    }
}

// =============================================================================
// CLASS
// =============================================================================

bitflags::bitflags! {
    struct ClassAccessFlags: u16 {
        const PUBLIC = 0x0001;
        const FINAL = 0x0010;
        const SUPER = 0x0020;
        const INTERFACE = 0x0200;
        const ABSTRACT = 0x0400;
        const SYNTHETIC = 0x1000;
        const ANNOTATION = 0x2000;
        const ENUM = 0x4000;
    }
}

#[derive(Debug)]
pub struct Class {
    minor_version: u16,
    major_version: u16,
    constant_pool: ConstantPool,
    access_flags: ClassAccessFlags,
    this_class: u16,
    super_class: u16,
    interface_indices: Vec<u16>,
    fields: Vec<FieldInfo>,
    methods: Vec<MethodInfo>,
    attributes: Vec<Attribute>,
}

impl Class {}

impl Class {
    pub fn read<R: ReadBytesExt>(reader: &mut R) -> Result<(), ClassLoadError> {
        let magic = reader.read_u32::<BigEndian>()?;
        if magic != CLASS_MAGIC {
            return Err(ClassLoadError::new("Magic header is not matching"));
        }

        let minor_version = reader.read_u16::<BigEndian>()?;
        let major_version = reader.read_u16::<BigEndian>()?;
        let constant_pool = ConstantPool::read_one(reader, &EmptyContext::default())?;
        // let access_flags = reader.read_u16::<BigEndian>()?;
        // let access_flags = ClassAccessFlags::from_bits(access_flags).ok_or(ClassLoadError::new("Invalid class access flags"))?;
        // let this_class = reader.read_u16::<BigEndian>()?;
        // let super_class = reader.read_u16::<BigEndian>()?;
        // let interface_indices = Class::read_interface_indices(reader)?;
        // let fields = Class::read_fields(reader, &constant_pool)?;
        // let methods = Class::read_methods(reader, &constant_pool)?;
        // let attributes = Class::read_attributes(reader, &constant_pool)?;

        // let mut rest = Vec::new();
        // reader.read_to_end(&mut rest);
        // if !rest.is_empty() {
        //     return Err(ClassLoadError::new("Data is still present after reading class file"));
        // }

        Ok(())

        // return Ok(Class {
        //     minor_version,
        //     major_version,
        //     constant_pool,
        //     access_flags,
        //     this_class,
        //     super_class,
        //     interface_indices,
        //     fields,
        //     methods,
        //     attributes,
        // });
    }
}