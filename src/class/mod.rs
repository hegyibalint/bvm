use std::fmt::{Debug, Display, Formatter};
use std::{fmt, io, string};

use byteorder::{BigEndian, ReadBytesExt};

use crate::class::attributes::Attribute;
use crate::class::constant_pool::{
    Constant, ConstantPool, ConstantPoolContext, Resolvable, ResolvedConstNameAndType,
};

pub mod attributes;
pub mod constant_pool;

// =============================================================================
// MACROS
// =============================================================================

/// Macro to print either an Ok value as is, or an Err written in parentheses.
macro_rules! write_result {
    ($f:expr, $result:expr) => {
        match $result {
            Ok(value) => write!($f, "{:#?}", value),
            Err(error) => write!($f, "({})", error),
        }
    };
}

// =============================================================================
// STATIC VALUES
// =============================================================================

/// This is the magic value used to start every class file.
static CLASS_MAGIC: u32 = 0xCAFEBABE;

// =============================================================================
// ERRORS
// =============================================================================

// ClassParsingError -----------------------------------------------------------

#[derive(Debug)]
pub enum ClassParsingError {
    ParsingError(&'static str),
    IoError(io::Error),
    Utf8Error(string::FromUtf8Error),
}

impl ClassParsingError {
    pub fn new(msg: &'static str) -> ClassParsingError {
        ClassParsingError::ParsingError(msg)
    }
}

impl Display for ClassParsingError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            ClassParsingError::ParsingError(details) => {
                write!(f, "{}", details)
            }
            ClassParsingError::IoError(err) => {
                write!(f, "{}", err)
            }
            ClassParsingError::Utf8Error(err) => {
                write!(f, "{}", err)
            }
        }
    }
}

impl From<io::Error> for ClassParsingError {
    fn from(err: io::Error) -> Self {
        ClassParsingError::IoError(err)
    }
}

impl From<string::FromUtf8Error> for ClassParsingError {
    fn from(err: string::FromUtf8Error) -> Self {
        ClassParsingError::Utf8Error(err)
    }
}

// ClassIndexingError ---------------------------------------------------------

#[derive(Debug)]
pub enum ConstantIndexingError {
    InvalidIndex(String),
    InvalidReference(String, String),
}

impl Display for ConstantIndexingError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ConstantIndexingError::InvalidIndex(msg) => {
                write!(f, "invalid index {}", msg)
            }
            ConstantIndexingError::InvalidReference(msg, ref_name) => {
                write!(f, "invalid reference, expected {} got {}", msg, ref_name)
            }
        }
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

trait ReadOne<C = EmptyContext>
where
    Self: Sized,
{
    fn read_one<R: ReadBytesExt>(reader: &mut R, context: &C) -> Result<Self, ClassParsingError>;
}

trait ReadAll<C = EmptyContext>
where
    Self: ReadOne<C>,
{
    fn read_count<R: ReadBytesExt>(reader: &mut R) -> Result<usize, ClassParsingError> {
        let count = reader.read_u16::<BigEndian>()? as usize;
        Ok(count)
    }

    fn skip_amount(_element: &Self) -> usize {
        return 0;
    }

    fn read_all_from<R: ReadBytesExt>(
        reader: &mut R,
        context: &C,
        from: usize,
    ) -> Result<Vec<Self>, ClassParsingError> {
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

    fn read_all<R: ReadBytesExt>(
        reader: &mut R,
        context: &C,
    ) -> Result<Vec<Self>, ClassParsingError> {
        Self::read_all_from(reader, context, 0)
    }
}

// =============================================================================
// CLASS FIELDS
// =============================================================================

// Field Info ------------------------------------------------------------------

bitflags::bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct FieldAccessFlags: u16 {
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
    pub access_flags: FieldAccessFlags,
    pub name_index: u16,
    pub descriptor_index: u16,
    pub attributes: Vec<Attribute>,
}

impl ReadOne<ConstantPoolContext<'_>> for FieldInfo {
    fn read_one<R: ReadBytesExt>(
        reader: &mut R,
        context: &ConstantPoolContext,
    ) -> Result<Self, ClassParsingError> {
        let access_flags = reader.read_u16::<BigEndian>()?;
        let access_flags = FieldAccessFlags::from_bits(access_flags)
            .ok_or(ClassParsingError::new("Invalid field access flags"))?;
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

impl ReadAll<ConstantPoolContext<'_>> for FieldInfo {}

// Interface -------------------------------------------------------------------

#[derive(Debug)]
pub struct Interface {
    pub interface_index: u16,
}

impl ReadOne<EmptyContext> for Interface {
    fn read_one<R: ReadBytesExt>(
        reader: &mut R,
        _: &EmptyContext,
    ) -> Result<Self, ClassParsingError> {
        let interface_index = reader.read_u16::<BigEndian>()?;
        Ok(Interface { interface_index })
    }
}

impl ReadAll for Interface {}

// Method Info -----------------------------------------------------------------

bitflags::bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct MethodAccessFlags: u16 {
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
    pub access_flags: MethodAccessFlags,
    pub name_index: u16,
    pub descriptor_index: u16,
    pub attributes: Vec<Attribute>,
}

impl ReadOne<ConstantPoolContext<'_>> for MethodInfo {
    fn read_one<R: ReadBytesExt>(
        reader: &mut R,
        context: &ConstantPoolContext,
    ) -> Result<Self, ClassParsingError> {
        let access_flags = reader.read_u16::<BigEndian>()?;
        let access_flags = MethodAccessFlags::from_bits(access_flags)
            .ok_or(ClassParsingError::new("Invalid method access flags"))?;
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

impl ReadAll<ConstantPoolContext<'_>> for MethodInfo {}

// =============================================================================
// CLASS
// =============================================================================

bitflags::bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct ClassAccessFlags: u16 {
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

impl Display for ClassAccessFlags {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        Display::fmt(&self.0, f)
    }
}

#[derive(Debug)]
pub struct Class {
    pub minor_version: u16,
    pub major_version: u16,
    pub constant_pool: ConstantPool,
    pub access_flags: ClassAccessFlags,
    pub this_class: u16,
    pub super_class: u16,
    pub interfaces: Vec<Interface>,
    pub fields: Vec<FieldInfo>,
    pub methods: Vec<MethodInfo>,
    pub attributes: Vec<Attribute>,
}

impl Class {
    pub fn read<R: ReadBytesExt>(reader: &mut R) -> Result<Class, ClassParsingError> {
        let magic = reader.read_u32::<BigEndian>()?;
        if magic != CLASS_MAGIC {
            return Err(ClassParsingError::new("Magic header is not matching"));
        }

        let empty_context = EmptyContext::default();

        let minor_version = reader.read_u16::<BigEndian>()?;
        let major_version = reader.read_u16::<BigEndian>()?;
        let constant_pool = ConstantPool::read_one(reader, &empty_context)?;
        let access_flags = reader.read_u16::<BigEndian>()?;
        let access_flags = ClassAccessFlags::from_bits(access_flags)
            .ok_or(ClassParsingError::new("Invalid class access flags"))?;
        let this_class = reader.read_u16::<BigEndian>()?;
        let super_class = reader.read_u16::<BigEndian>()?;
        let interfaces = Interface::read_all(reader, &empty_context)?;
        let fields = FieldInfo::read_all(reader, &ConstantPoolContext::new(&constant_pool))?;
        let methods = MethodInfo::read_all(reader, &ConstantPoolContext::new(&constant_pool))?;
        let attributes = Attribute::read_all(reader, &ConstantPoolContext::new(&constant_pool))?;

        let mut rest = Vec::new();
        reader.read(&mut rest)?;
        if !rest.is_empty() {
            return Err(ClassParsingError::new(
                "Data is still present after reading class file",
            ));
        }

        return Ok(Class {
            minor_version,
            major_version,
            constant_pool,
            access_flags,
            this_class,
            super_class,
            interfaces,
            fields,
            methods,
            attributes,
        });
    }

    fn java_release_version(major_version: u16, minor_version: u16) -> &'static str {
        match major_version {
            45 => match minor_version {
                0 => "1.0",
                3 => "1.1",
                _ => "Unknown",
            },
            46 => match minor_version {
                0 => "1.2",
                _ => "Unknown",
            },
            47 => match minor_version {
                0 => "1.3",
                _ => "Unknown",
            },
            48 => match minor_version {
                0 => "1.4",
                _ => "Unknown",
            },
            49 => match minor_version {
                0 => "1.5",
                _ => "Unknown",
            },
            50 => match minor_version {
                0 => "1.6",
                _ => "Unknown",
            },
            51 => match minor_version {
                0 => "1.7",
                _ => "Unknown",
            },
            52 => match minor_version {
                0 => "1.8",
                _ => "Unknown",
            },
            53 => match minor_version {
                0 => "1.9",
                _ => "Unknown",
            },
            54 => match minor_version {
                0 => "1.10",
                _ => "Unknown",
            },
            55 => match minor_version {
                0 => "1.11",
                _ => "Unknown",
            },
            56 => match minor_version {
                0 => "1.12",
                _ => "Unknown",
            },
            57 => match minor_version {
                0 => "1.13",
                _ => "Unknown",
            },
            58 => match minor_version {
                0 => "1.14",
                _ => "Unknown",
            },
            59 => match minor_version {
                0 => "1.15",
                _ => "Unknown",
            },
            60 => match minor_version {
                0 => "1.16",
                _ => "Unknown",
            },
            _ => "Unknown",
        }
    }
}

impl Display for Class {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let constant_pool_context = ConstantPoolContext::new(&self.constant_pool);

        write!(f, "Class")?;
        match self.constant_pool.get_class_name(self.this_class) {
            Ok(class_name) => write!(f, " '{}'", class_name),
            Err(err) => write!(f, "({})", err),
        }?;

        write!(f, " extends")?;
        match self.super_class {
            0 => write!(f, " java.lang.Object"),
            _ => match self.constant_pool.get_class_name(self.super_class) {
                Ok(class_name) => write!(f, " '{}'", class_name),
                Err(err) => write!(f, "({})", err),
            },
        }?;
        writeln!(f)?;

        writeln!(
            f,
            "  Major version: {}, minor version: {} (Java {})",
            self.major_version,
            self.minor_version,
            Class::java_release_version(self.major_version, self.minor_version)
        )?;
        writeln!(f, "  Access flags: {}", self.access_flags)?;

        writeln!(f, "  Constant pool:")?;
        for (index, constant_enum) in self.constant_pool.iter().enumerate() {
            write!(f, "    #{} => ", index + 1)?;
            match constant_enum {
                Constant::Utf8(constant) => {
                    write!(f, "UTF-8: {}", constant.string)?;
                }
                Constant::Integer(constant) => {
                    write!(f, "Integer: {}", constant.value)?;
                }
                Constant::Float(constant) => {
                    write!(f, "Float: {}", constant.value)?;
                }
                Constant::Long(constant) => {
                    write!(f, "Long: {}", constant.value)?;
                }
                Constant::Double(constant) => {
                    write!(f, "Double: {}", constant.value)?;
                }
                Constant::Class(constant) => {
                    write!(f, "Class: ")?;
                    write_result!(f, self.constant_pool.get_utf8(constant.name_index))?;
                }
                Constant::String(constant) => {
                    write!(f, "String @ {}: ", constant.string_index)?;
                    write_result!(f, self.constant_pool.get_utf8(constant.string_index))?;
                }
                Constant::Field(constant)
                | Constant::Method(constant)
                | Constant::InterfaceMethod(constant) => {
                    match constant_enum {
                        Constant::Field(_) => write!(f, "Field: ")?,
                        Constant::Method(_) => write!(f, "Method: ")?,
                        Constant::InterfaceMethod(_) => write!(f, "InterfaceMethod: ")?,
                        _ => unreachable!(),
                    }
                    match &self.constant_pool[constant.name_and_type_index] {
                        Constant::NameAndType(constant) => {
                            match constant.resolve(&constant_pool_context) {
                                Ok(resolved) => {
                                    write_result!(f, resolved.name)?;
                                    write!(f, " ")?;
                                    write_result!(f, resolved.type_name)
                                }
                                Err(err) => write!(f, "({})", err),
                            }
                        }
                        constant => write!(f, "(invalid reference: {})", constant),
                    }?;
                }
                Constant::NameAndType(constant) => {
                    write!(f, "NameAndType: ")?;
                    match self.constant_pool.get_utf8(constant.descriptor_index) {
                        Ok(name) => write!(f, "{}", name),
                        Err(err) => match err {
                            ConstantIndexingError::InvalidIndex(err) => {
                                write!(f, "(invalid index {})", err)
                            }

                            ConstantIndexingError::InvalidReference(assumed, actual) => {
                                write!(f, "(invalid type {})", actual)
                            }
                        },
                    }?;
                    match self.constant_pool.get_utf8(constant.name_index) {
                        Ok(name) => write!(f, "{}", name),
                        Err(err) => match err {
                            ConstantIndexingError::InvalidIndex(err) => {
                                write!(f, "(invalid index {})", err)
                            }
                            ConstantIndexingError::InvalidReference(assumed, actual) => {
                                write!(f, "(invalid type {})", actual)
                            }
                        },
                    }?;
                }
                Constant::MethodHandle(constant) => {}
                Constant::MethodType(constant) => {}
                Constant::InvokeDynamic(constant) => {}
            }
            writeln!(f)?;
        }

        return Ok(());
    }
}
