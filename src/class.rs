use std::{fmt, io, string};
use std::error::Error;
use std::fs::read;
use std::io::Read;
use std::ops::Index;
use std::string::ParseError;

use byteorder::{BigEndian, ReadBytesExt};

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
// Common traits
// =============================================================================

trait Counted {
    fn read_count<R: ReadBytesExt>(reader: &mut R) -> Result<usize, ClassLoadError> {
        let count = reader.read_u16::<BigEndian>()? as usize;
        Ok(count)
    }
}

trait ReadOne<T> {
    fn read<R: ReadBytesExt>(reader: &mut R) -> Result<T, ClassLoadError>;
}

trait ReadAll<T>: Counted where T: ReadOne<T> {
    fn read_all<R: ReadBytesExt>(reader: &mut R) -> Result<Vec<T>, ClassLoadError> {
        let count = Self::read_count(reader)?;
        let mut elements = Vec::with_capacity(count);
        for _i in 0..count {
            let element = T::read(reader)?;
            elements.push(element);
        }
        Ok(elements)
    }
}

trait ReadOneWithConstPool<T> {
    fn read<R: ReadBytesExt>(reader: &mut R, const_pool: &ConstantPool) -> Result<T, ClassLoadError>;
}

trait ReadAllWithConstPool<T>: Counted where T: ReadOneWithConstPool<T>
{
    fn read_all<R: ReadBytesExt>(reader: &mut R, const_pool: &ConstantPool) -> Result<Vec<T>, ClassLoadError> {
        let count = Self::read_count(reader)?;
        let mut elements = Vec::with_capacity(count);
        for _i in 0..count {
            let element = T::read(reader, const_pool)?;
            elements.push(element);
        }

        Ok(elements)
    }
}

// ============================================================================
// CONSTANT POOL
// ============================================================================

// ConstantClass ---------------------------------------------------------------

#[derive(Debug)]
pub struct ClassConstant {
    name_index: u16,
}

impl ReadOne<ClassConstant> for ClassConstant {
    fn read<R: ReadBytesExt>(reader: &mut R) -> Result<Self, ClassLoadError> {
        Ok(ClassConstant {
            name_index: reader.read_u16::<BigEndian>()?
        })
    }
}

// ConstantFieldRef ------------------------------------------------------------

#[derive(Debug)]
pub struct FieldRefConstant {
    class_index: u16,
    name_and_type_index: u16,
}

impl ReadOne<FieldRefConstant> for FieldRefConstant {
    fn read<R: ReadBytesExt>(reader: &mut R) -> Result<FieldRefConstant, ClassLoadError> {
        Ok(FieldRefConstant {
            class_index: reader.read_u16::<BigEndian>()?,
            name_and_type_index: reader.read_u16::<BigEndian>()?,
        })
    }
}

// ConstantMethodRef -----------------------------------------------------------

#[derive(Debug)]
pub struct MethodRefConstant {
    class_index: u16,
    name_and_type_index: u16,
}

impl ReadOne<MethodRefConstant> for MethodRefConstant {
    fn read<R: ReadBytesExt>(reader: &mut R) -> Result<MethodRefConstant, ClassLoadError> {
        Ok(MethodRefConstant {
            class_index: reader.read_u16::<BigEndian>()?,
            name_and_type_index: reader.read_u16::<BigEndian>()?,
        })
    }
}

// ConstantInterfaceMethodRef --------------------------------------------------

#[derive(Debug)]
pub struct InterfaceMethodRefConstant {
    class_index: u16,
    name_and_type_index: u16,
}

impl ReadOne<InterfaceMethodRefConstant> for InterfaceMethodRefConstant {
    fn read<R: ReadBytesExt>(reader: &mut R) -> Result<InterfaceMethodRefConstant, ClassLoadError> {
        Ok(InterfaceMethodRefConstant {
            class_index: reader.read_u16::<BigEndian>()?,
            name_and_type_index: reader.read_u16::<BigEndian>()?,
        })
    }
}

// ConstantString --------------------------------------------------------------

#[derive(Debug)]
pub struct StringConstant {
    string_index: u16,
}

impl ReadOne<StringConstant> for StringConstant {
    fn read<R: ReadBytesExt>(reader: &mut R) -> Result<StringConstant, ClassLoadError> {
        Ok(StringConstant {
            string_index: reader.read_u16::<BigEndian>()?
        })
    }
}

// ConstantInteger -------------------------------------------------------------

#[derive(Debug)]
pub struct IntegerConstant {
    value: i32,
}

impl ReadOne<IntegerConstant> for IntegerConstant {
    fn read<R: ReadBytesExt>(reader: &mut R) -> Result<IntegerConstant, ClassLoadError> {
        Ok(IntegerConstant {
            value: reader.read_i32::<BigEndian>()?
        })
    }
}

// ConstantFloat ---------------------------------------------------------------

#[derive(Debug)]
pub struct ConstantFloat {
    value: f32,
}

impl ReadOne<ConstantFloat> for ConstantFloat {
    fn read<R: ReadBytesExt>(reader: &mut R) -> Result<ConstantFloat, ClassLoadError> {
        Ok(ConstantFloat {
            value: reader.read_f32::<BigEndian>()?
        })
    }
}

// ConstantLong ----------------------------------------------------------------

#[derive(Debug)]
pub struct ConstantLong {
    value: i64,
}

impl ReadOne<ConstantLong> for ConstantLong {
    fn read<R: ReadBytesExt>(reader: &mut R) -> Result<ConstantLong, ClassLoadError> {
        Ok(ConstantLong {
            value: reader.read_i64::<BigEndian>()?
        })
    }
}

// ConstantDouble --------------------------------------------------------------

#[derive(Debug)]
pub struct ConstantDouble {
    value: f64,
}

impl ReadOne<ConstantDouble> for ConstantDouble {
    fn read<R: ReadBytesExt>(reader: &mut R) -> Result<ConstantDouble, ClassLoadError> {
        Ok(ConstantDouble {
            value: reader.read_f64::<BigEndian>()?
        })
    }
}

// ConstantNameAndType ---------------------------------------------------------

#[derive(Debug)]
pub struct ConstantNameAndType {
    name_index: u16,
    descriptor_index: u16,
}

impl ReadOne<ConstantNameAndType> for ConstantNameAndType {
    fn read<R: ReadBytesExt>(reader: &mut R) -> Result<ConstantNameAndType, ClassLoadError> {
        Ok(ConstantNameAndType {
            name_index: reader.read_u16::<BigEndian>()?,
            descriptor_index: reader.read_u16::<BigEndian>()?,
        })
    }
}

// ConstantUtf8 ----------------------------------------------------------------

#[derive(Debug)]
pub struct ConstantUtf8 {
    string: String,
}

impl ReadOne<ConstantUtf8> for ConstantUtf8 {
    fn read<R: ReadBytesExt>(reader: &mut R) -> Result<ConstantUtf8, ClassLoadError> {
        let length = reader.read_u16::<BigEndian>()?;
        let mut bytes: Vec<u8> = vec![0; length as usize];
        reader.read_exact(&mut bytes)?;
        let string = String::from_utf8(bytes)?;

        Ok(ConstantUtf8 {
            string
        })
    }
}

// ConstantMethodHandle --------------------------------------------------------

#[derive(Debug)]
pub struct ConstantMethodHandle {
    reference_kind: u8,
    reference_index: u16,
}

impl ReadOne<ConstantMethodHandle> for ConstantMethodHandle {
    fn read<R: ReadBytesExt>(reader: &mut R) -> Result<ConstantMethodHandle, ClassLoadError> {
        Ok(ConstantMethodHandle {
            reference_kind: reader.read_u8()?,
            reference_index: reader.read_u16::<BigEndian>()?,
        })
    }
}

// ConstantMethodType ----------------------------------------------------------

#[derive(Debug)]
pub struct ConstantMethodType {
    descriptor_index: u16,
}

impl ReadOne<ConstantMethodType> for ConstantMethodType {
    fn read<R: ReadBytesExt>(reader: &mut R) -> Result<ConstantMethodType, ClassLoadError> {
        Ok(ConstantMethodType {
            descriptor_index: reader.read_u16::<BigEndian>()?
        })
    }
}

// ConstantInvokeDynamic -------------------------------------------------------

#[derive(Debug)]
pub struct ConstantInvokeDynamic {
    bootstrap_method_attr_index: u16,
    name_and_type_index: u16,
}

impl ReadOne<ConstantInvokeDynamic> for ConstantInvokeDynamic {
    fn read<R: ReadBytesExt>(reader: &mut R) -> Result<ConstantInvokeDynamic, ClassLoadError> {
        Ok(ConstantInvokeDynamic {
            bootstrap_method_attr_index: reader.read_u16::<BigEndian>()?,
            name_and_type_index: reader.read_u16::<BigEndian>()?,
        })
    }
}

// Constant ----------------------------------------------------------------

#[derive(Debug)]
pub enum ConstantInfo {
    Class(ClassConstant),
    Field(FieldRefConstant),
    Method(MethodRefConstant),
    InterfaceMethod(InterfaceMethodRefConstant),
    String(StringConstant),
    Integer(IntegerConstant),
    Float(ConstantFloat),
    Long(ConstantLong),
    Double(ConstantDouble),
    NameAndType(ConstantNameAndType),
    Utf8(ConstantUtf8),
    MethodHandle(ConstantMethodHandle),
    MethodType(ConstantMethodType),
    InvokeDynamic(ConstantInvokeDynamic),
}

type ConstantPool = Vec<ConstantInfo>;

// =============================================================================
// ATTRIBUTES
// =============================================================================

// ConstantValue Attribute -----------------------------------------------------

#[derive(Debug)]
pub struct ConstantValueAttribute {
    const_value_index: u16,
}

impl ReadOne<ConstantValueAttribute> for ConstantValueAttribute {
    fn read<R: ReadBytesExt>(reader: &mut R) -> Result<ConstantValueAttribute, ClassLoadError> {
        let const_value_index = reader.read_u16::<BigEndian>()?;
        Ok(ConstantValueAttribute {
            const_value_index
        })
    }
}

// Code Attribute --------------------------------------------------------------

#[derive(Debug)]
pub struct ExceptionTable {
    start_pc: u16,
    end_pc: u16,
    handler_pc: u16,
    catch_type: u16,
}

impl ReadOne<ExceptionTable> for ExceptionTable {
    fn read<R: ReadBytesExt>(reader: &mut R) -> Result<ExceptionTable, ClassLoadError> {
        let start_pc = reader.read_u16::<BigEndian>()?;
        let end_pc = reader.read_u16::<BigEndian>()?;
        let handler_pc = reader.read_u16::<BigEndian>()?;
        let catch_type = reader.read_u16::<BigEndian>()?;

        Ok(ExceptionTable {
            start_pc,
            end_pc,
            handler_pc,
            catch_type,
        })
    }
}

impl Counted for ExceptionTable {}
impl ReadAll<ExceptionTable> for ExceptionTable {}

#[derive(Debug)]
pub struct CodeAttribute {
    max_stack: u16,
    max_locals: u16,
    code: Vec<u8>,
    exception_tables: Vec<ExceptionTable>,
    attributes: Vec<Attribute>,
}

impl ReadOneWithConstPool<CodeAttribute> for CodeAttribute {
    fn read<R: ReadBytesExt>(reader: &mut R, const_pool: &ConstantPool) -> Result<CodeAttribute, ClassLoadError> {
        let max_stack = reader.read_u16::<BigEndian>()?;
        let max_locals = reader.read_u16::<BigEndian>()?;

        let code_length = reader.read_u32::<BigEndian>()? as usize;
        let mut code = vec![0; code_length];
        reader.read_exact(&mut code)?;

        let exception_tables = ExceptionTable::read_all(reader)?;

        let attributes_count = reader.read_u16::<BigEndian>()? as usize;
        let mut attributes = Vec::new();
        for _i in 0..attributes_count {
            let attribute = Attribute::read(reader, const_pool)?;
            attributes.push(attribute);
        }

        Ok(CodeAttribute {
            max_stack,
            max_locals,
            code,
            exception_tables,
            attributes,
        })
    }
}

// StackMapFrame Attribute -----------------------------------------------------

#[derive(Debug)]
pub struct StackMapFrame {

}

#[derive(Debug)]
pub struct StackMapTableAttribute {}

impl ReadOneWithConstPool<StackMapTableAttribute> for StackMapTableAttribute {
    fn read<R: ReadBytesExt>(reader: &mut R, const_pool: &ConstantPool) -> Result<StackMapTableAttribute, ClassLoadError> {
        // TODO: Figure this crap out
        todo!()
    }
}

// Exceptions Attribute --------------------------------------------------------

#[derive(Debug)]
struct ExceptionIndex {
    index: u16
}

impl ReadOneWithConstPool<ExceptionIndex> for ExceptionIndex {
    fn read<R: ReadBytesExt>(reader: &mut R, const_pool: &ConstantPool) -> Result<ExceptionIndex, ClassLoadError> {
        // TODO: Check references
        let index = reader.read_u16::<BigEndian>()?;
        Ok(ExceptionIndex {
            index
        })
    }
}

impl Counted for ExceptionIndex {}
impl ReadAllWithConstPool<ExceptionIndex> for ExceptionIndex {}

#[derive(Debug)]
struct ExceptionsAttribute {
    exception_indices: Vec<ExceptionIndex>
}

impl ReadOneWithConstPool<ExceptionsAttribute> for ExceptionsAttribute {
    fn read<R: ReadBytesExt>(reader: &mut R, const_pool: &ConstantPool) -> Result<ExceptionsAttribute, ClassLoadError> {
        let exception_indices = ExceptionIndex::read_all(reader, const_pool)?;
        Ok(ExceptionsAttribute {
            exception_indices
        })
    }
}

// InnerClasses Attribute ------------------------------------------------------

bitflags::bitflags! {
    struct InnerClassAccessFlags: u16 {
        const PUBLIC = 0x0001;
        const PRIVATE = 0x0002;
        const PROTECTED = 0x0004;
        const STATIC = 0x0008;
        const FINAL = 0x0010;
        const INTERFACE = 0x0200;
        const ABSTRACT = 0x0400;
        const SYNTHETIC = 0x1000;
        const ANNOTATION = 0x2000;
        const ENUM = 0x4000;
    }
}

#[derive(Debug)]
pub struct InnerClass {
    inner_class_info_index: u16,
    outer_class_info_index: u16,
    inner_name_index: u16,
    inner_class_access_flags: InnerClassAccessFlags,
}

impl ReadOneWithConstPool<InnerClass> for InnerClass {
    fn read<R: ReadBytesExt>(reader: &mut R, const_pool: &ConstantPool) -> Result<InnerClass, ClassLoadError> {
        // TODO: Check references
        let inner_class_info_index = reader.read_u16::<BigEndian>()?;
        let outer_class_info_index = reader.read_u16::<BigEndian>()?;
        let inner_name_index = reader.read_u16::<BigEndian>()?;
        let inner_class_access_flags = reader.read_u16::<BigEndian>()?;
        let inner_class_access_flags = InnerClassAccessFlags::from_bits(inner_class_access_flags).ok_or(ClassLoadError::new("Invalid inner class access flags"))?;

        Ok(InnerClass {
            inner_class_info_index,
            outer_class_info_index,
            inner_name_index,
            inner_class_access_flags
        })
    }
}

impl Counted for InnerClass {}
impl ReadAllWithConstPool<InnerClass> for InnerClass { }

#[derive(Debug)]
pub struct InnerClassAttribute {
    inner_classes: Vec<InnerClass>
}

impl ReadOneWithConstPool<InnerClassAttribute> for InnerClassAttribute {
    fn read<R: ReadBytesExt>(reader: &mut R, const_pool: &ConstantPool) -> Result<InnerClassAttribute, ClassLoadError> {
        let inner_classes = InnerClass::read_all(reader, const_pool)?;
        Ok(InnerClassAttribute {
            inner_classes
        })
    }
}

// EnclosingMethod Attribute ---------------------------------------------------

#[derive(Debug)]
pub struct EnclosingMethodAttribute {
    class_index: u16,
    method_index: u16
}

impl ReadOneWithConstPool<EnclosingMethodAttribute> for EnclosingMethodAttribute {
    fn read<R: ReadBytesExt>(reader: &mut R, const_pool: &ConstantPool) -> Result<EnclosingMethodAttribute, ClassLoadError> {
        // TODO: Check references
        let class_index = reader.read_u16::<BigEndian>()?;
        let method_index = reader.read_u16::<BigEndian>()?;

        Ok(EnclosingMethodAttribute {
            class_index,
            method_index
        })
    }
}

// Signature Attribute ---------------------------------------------------------

#[derive(Debug)]
pub struct SignatureAttribute {
    signature_index: u16,
}

impl ReadOneWithConstPool<SignatureAttribute> for SignatureAttribute {
    fn read<R: ReadBytesExt>(reader: &mut R, const_pool: &ConstantPool) -> Result<SignatureAttribute, ClassLoadError> {
        // TODO: Check references
        let signature_index = reader.read_u16::<BigEndian>()?;

        Ok(SignatureAttribute {
            signature_index
        })
    }
}

// SourceFile Attribute --------------------------------------------------------

#[derive(Debug)]
pub struct SourceFileAttribute {
    sourcefile_index: u16
}

impl ReadOneWithConstPool<SourceFileAttribute> for SourceFileAttribute {
    fn read<R: ReadBytesExt>(reader: &mut R, const_pool: &ConstantPool) -> Result<SourceFileAttribute, ClassLoadError> {
        // TODO: Check references
        let sourcefile_index = reader.read_u16::<BigEndian>()?;

        Ok(SourceFileAttribute {
            sourcefile_index
        })
    }
}

// SourceDebugExtension Attribute ----------------------------------------------

#[derive(Debug)]
pub struct SourceDebugExtensionAttribute {
    debug_info: Vec<u8>,
}

impl SourceDebugExtensionAttribute {
    fn read<R: ReadBytesExt>(reader: &mut R, attribute_length: usize, const_pool: &ConstantPool) -> Result<SourceDebugExtensionAttribute, ClassLoadError> {
        // TODO: Check references
        let mut debug_info = vec![0; attribute_length];
        reader.read_exact(&mut debug_info)?;

        Ok(SourceDebugExtensionAttribute {
            debug_info
        })
    }
}

// LineNumberTable Attribute ---------------------------------------------------

#[derive(Debug)]
pub struct LineNumberTable {
    start_pc: u16,
    line_number: u16
}

impl ReadOne<LineNumberTable> for LineNumberTable {
    fn read<R: ReadBytesExt>(reader: &mut R) -> Result<LineNumberTable, ClassLoadError> {
        let start_pc = reader.read_u16::<BigEndian>()?;
        let line_number = reader.read_u16::<BigEndian>()?;

        Ok(LineNumberTable {
            start_pc,
            line_number
        })
    }
}

impl Counted for LineNumberTable {}
impl ReadAll<LineNumberTable> for LineNumberTable {}

#[derive(Debug)]
pub struct LineNumberTableAttribute {
    line_number_tables: Vec<LineNumberTable>
}

impl ReadOne<LineNumberTableAttribute> for LineNumberTableAttribute {
    fn read<R: ReadBytesExt>(reader: &mut R) -> Result<LineNumberTableAttribute, ClassLoadError> {
        let line_number_tables = LineNumberTable::read_all(reader)?;
        Ok(LineNumberTableAttribute {
            line_number_tables
        })
    }
}

// LocalVariableTable Attribute ------------------------------------------------

#[derive(Debug)]
pub struct LocalVariableTable {
    start_pc: u16,
    length: u16,
    name_index: u16,
    descriptor_index: u16,
    index: u16
}

impl ReadOneWithConstPool<LocalVariableTable> for LocalVariableTable {
    fn read<R: ReadBytesExt>(reader: &mut R, const_pool: &ConstantPool) -> Result<LocalVariableTable, ClassLoadError> {
        // TODO: Check references
        let start_pc = reader.read_u16::<BigEndian>()?;
        let length = reader.read_u16::<BigEndian>()?;
        let name_index = reader.read_u16::<BigEndian>()?;
        let descriptor_index = reader.read_u16::<BigEndian>()?;
        let index = reader.read_u16::<BigEndian>()?;

        Ok(LocalVariableTable {
            start_pc,
            length,
            name_index,
            descriptor_index,
            index
        })
    }
}

impl Counted for LocalVariableTable {}
impl ReadAllWithConstPool<LocalVariableTable> for LocalVariableTable {}

#[derive(Debug)]
pub struct LocalVariableTableAttribute {
    local_variable_tables: Vec<LocalVariableTable>
}

impl ReadOneWithConstPool<LocalVariableTableAttribute> for LocalVariableTableAttribute {
    fn read<R: ReadBytesExt>(reader: &mut R, const_pool: &ConstantPool) -> Result<LocalVariableTableAttribute, ClassLoadError> {
        let local_variable_tables = LocalVariableTable::read_all(reader, const_pool)?;

        Ok(LocalVariableTableAttribute {
            local_variable_tables
        })
    }
}

// LocalVariableTypeTable Attribute --------------------------------------------

#[derive(Debug)]
pub struct LocalVariableTypeTable {
    start_pc: u16,
    length: u16,
    name_index: u16,
    signature_index: u16,
    index: u16
}

impl ReadOneWithConstPool<LocalVariableTypeTable> for LocalVariableTypeTable {
    fn read<R: ReadBytesExt>(reader: &mut R, const_pool: &ConstantPool) -> Result<LocalVariableTypeTable, ClassLoadError> {
        let start_pc = reader.read_u16::<BigEndian>()?;
        let length = reader.read_u16::<BigEndian>()?;
        let name_index = reader.read_u16::<BigEndian>()?;
        let signature_index = reader.read_u16::<BigEndian>()?;
        let index = reader.read_u16::<BigEndian>()?;

        Ok(LocalVariableTypeTable {
            start_pc,
            length,
            name_index,
            signature_index,
            index
        })
    }
}

impl Counted for LocalVariableTypeTable {}
impl ReadAllWithConstPool<LocalVariableTypeTable> for LocalVariableTypeTable {}

#[derive(Debug)]
pub struct LocalVariableTypeTableAttribute {
    local_variable_type_tables: Vec<LocalVariableTypeTable>
}

impl ReadOneWithConstPool<LocalVariableTypeTableAttribute> for LocalVariableTypeTableAttribute {
    fn read<R: ReadBytesExt>(reader: &mut R, const_pool: &ConstantPool) -> Result<LocalVariableTypeTableAttribute, ClassLoadError> {
        let local_variable_type_tables = LocalVariableTypeTable::read_all(reader, const_pool)?;

        Ok(LocalVariableTypeTableAttribute {
            local_variable_type_tables
        })
    }
}

// RuntimeVisibleAnnotations Attribute -----------------------------------------

pub struct ElementValue {

}

pub struct Annotation {
    type_index: u16,

}

pub struct RuntimeVisibleAnnotations {
    annotations: Vec<Annotation>
}

// Misc Attribute --------------------------------------------------------------

#[derive(Debug)]
struct MiscAttribute {
    name_index: u16,
    info: Vec<u8>
}

impl MiscAttribute {
    fn read<R: ReadBytesExt>(reader: &mut R, name_index: u16, attribute_length: usize, const_pool: &ConstantPool) -> Result<MiscAttribute, ClassLoadError> {
        // TODO: Check references
        let mut info = vec![0; attribute_length];
        reader.read_exact(&mut info)?;

        Ok(MiscAttribute {
            name_index,
            info
        })
    }
}

// Attribute -------------------------------------------------------------------

#[derive(Debug)]
pub enum Attribute {
    ConstantValue(ConstantValueAttribute),
    Code(CodeAttribute),
    StackMapTable(StackMapTableAttribute),
    Exceptions(ExceptionsAttribute),
    InnerClasses(InnerClassAttribute),
    EnclosingMethod(EnclosingMethodAttribute),
    Synthetic(),
    Signature(SignatureAttribute),
    SourceFile(SourceFileAttribute),
    SourceDebugExtension(SourceDebugExtensionAttribute),
    LineNumberTable(LineNumberTableAttribute),
    LocalVariableTable(LocalVariableTableAttribute),
    LocalVariableTypeTable(LocalVariableTypeTableAttribute),
    Deprecated(),
    RuntimeVisibleAnnotations(),
    RuntimeInvisibleAnnotations(),
    RuntimeVisibleParameterAnnotations(),
    RuntimeInvisibleParameterAnnotations(),
    AnnotationDefault(),
    BootstrapMethods(),
    Misc(MiscAttribute),
}

impl Attribute {
    pub fn read<R: ReadBytesExt>(reader: &mut R, const_pool: &ConstantPool) -> Result<Attribute, ClassLoadError> {
        let attribute_name_index = reader.read_u16::<BigEndian>()?;
        let attribute_length = reader.read_u32::<BigEndian>()? as usize;

        // Dereference the name from the constant pool
        let attribute_name = match &const_pool[attribute_name_index as usize - 1] {
            // If the referenced constant is an UTF8 reference, we are up to spec
            ConstantInfo::Utf8(value) => Ok(&value.string),
            // Otherwise, we blow up, as nothing else is acceptable
            _ => Err(ClassLoadError::new("Referenced attribute name should be an UTF-8 constant"))
        }?.as_str();

        let attribute = match attribute_name {
            "ConstantValue" => Attribute::ConstantValue(ConstantValueAttribute::read(reader)?),
            "Code" => Attribute::Code(CodeAttribute::read(reader, const_pool)?),
            "StackMapTable" => Attribute::StackMapTable(StackMapTableAttribute::read(reader, const_pool)?),
            "Exceptions" => Attribute::Exceptions(ExceptionsAttribute::read(reader, const_pool)?),
            "InnerClasses" => Attribute::InnerClasses(InnerClassAttribute::read(reader, const_pool)?),
            "EnclosingMethod" => Attribute::EnclosingMethod(EnclosingMethodAttribute::read(reader, const_pool)?),
            "Synthetic" => Attribute::Synthetic(),
            "Signature" => Attribute::Signature(SignatureAttribute::read(reader, const_pool)?),
            "SourceFile" => Attribute::SourceFile(SourceFileAttribute::read(reader, const_pool)?),
            "SourceDebugExtension" => Attribute::SourceDebugExtension(SourceDebugExtensionAttribute::read(reader, attribute_length, const_pool)?),
            "LineNumberTable" => Attribute::LineNumberTable(LineNumberTableAttribute::read(reader)?),
            "LocalVariableTable" => Attribute::LocalVariableTable(LocalVariableTableAttribute::read(reader, const_pool)?),
            "LocalVariableTypeTable" => Attribute::LocalVariableTypeTable(LocalVariableTypeTableAttribute::read(reader, const_pool)?),
            "Deprecated" => Attribute::Deprecated(),
            // "RuntimeVisibleAnnotations" => Attribute::read_runtime_visible_annotations(reader, const_pool),
            // "RuntimeInvisibleAnnotations" => Attribute::read_runtime_invisible_annotations(reader, const_pool),
            // "RuntimeVisibleParameterAnnotations" => Attribute::read_runtime_visible_parameter_annotations(reader, const_pool),
            // "RuntimeInvisibleParameterAnnotations" => Attribute::read_runtime_invisible_parameter_annotations(reader, const_pool),
            // "AnnotationDefault" => Attribute::read_annotation_default(reader, const_pool),
            // "BootstrapMethods" => Attribute::read_bootstrap_methods(reader, const_pool),
            _ => Attribute::Misc(MiscAttribute::read(reader, attribute_name_index, attribute_length, const_pool)?)
        };
        Ok(attribute)
    }
}

// =============================================================================
// FIELDS
// =============================================================================

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

impl FieldInfo {
    fn read_attributes<R: ReadBytesExt>(reader: &mut R, constant_pool: &ConstantPool) -> Result<Vec<Attribute>, ClassLoadError> {
        let attributes_count = reader.read_u16::<BigEndian>()? as usize;
        let mut attributes = Vec::with_capacity(attributes_count);
        for _i in 0..attributes_count {
            let attribute = Attribute::read(reader, constant_pool)?;
            attributes.push(attribute);
        }

        return Ok(attributes);
    }

    pub fn read<R: ReadBytesExt>(reader: &mut R, constant_pool: &ConstantPool) -> Result<FieldInfo, ClassLoadError> {
        let access_flags = reader.read_u16::<BigEndian>()?;
        let access_flags = FieldAccessFlags::from_bits(access_flags).ok_or(ClassLoadError::new("Invalid field access flags"))?;
        let name_index = reader.read_u16::<BigEndian>()?;
        let descriptor_index = reader.read_u16::<BigEndian>()?;
        let attributes = FieldInfo::read_attributes(reader, constant_pool)?;

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

impl MethodInfo {
    fn read_attributes<R: ReadBytesExt>(reader: &mut R, constant_pool: &ConstantPool) -> Result<Vec<Attribute>, ClassLoadError> {
        let attributes_count = reader.read_u16::<BigEndian>()? as usize;
        let mut attributes = Vec::with_capacity(attributes_count);
        for _i in 0..attributes_count {
            let attribute = Attribute::read(reader, constant_pool)?;
            attributes.push(attribute);
        }

        return Ok(attributes);
    }

    pub fn read<R: ReadBytesExt>(reader: &mut R, constant_pool: &ConstantPool) -> Result<MethodInfo, ClassLoadError> {
        let access_flags = reader.read_u16::<BigEndian>()?;
        let access_flags = MethodAccessFlags::from_bits(access_flags).ok_or(ClassLoadError::new("Invalid method access flags"))?;
        let name_index = reader.read_u16::<BigEndian>()?;
        let descriptor_index = reader.read_u16::<BigEndian>()?;
        let attributes = MethodInfo::read_attributes(reader, constant_pool)?;

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
    fn read_constant_pool<R: ReadBytesExt>(reader: &mut R) -> Result<Vec<ConstantInfo>, ClassLoadError> {
        let constant_pool_count = reader.read_u16::<BigEndian>()?;
        let mut constants = Vec::new();

        for _ in 1..constant_pool_count {
            let tag = reader.read_u8()?;
            let constant = match tag {
                1 => Ok(ConstantInfo::Utf8(ConstantUtf8::read(reader)?)),
                3 => Ok(ConstantInfo::Integer(IntegerConstant::read(reader)?)),
                4 => Ok(ConstantInfo::Float(ConstantFloat::read(reader)?)),
                5 => Ok(ConstantInfo::Long(ConstantLong::read(reader)?)),
                6 => Ok(ConstantInfo::Double(ConstantDouble::read(reader)?)),
                7 => Ok(ConstantInfo::Class(ClassConstant::read(reader)?)),
                8 => Ok(ConstantInfo::String(StringConstant::read(reader)?)),
                9 => Ok(ConstantInfo::Field(FieldRefConstant::read(reader)?)),
                10 => Ok(ConstantInfo::Method(MethodRefConstant::read(reader)?)),
                11 => Ok(ConstantInfo::InterfaceMethod(InterfaceMethodRefConstant::read(reader)?)),
                12 => Ok(ConstantInfo::NameAndType(ConstantNameAndType::read(reader)?)),
                15 => Ok(ConstantInfo::MethodHandle(ConstantMethodHandle::read(reader)?)),
                16 => Ok(ConstantInfo::MethodType(ConstantMethodType::read(reader)?)),
                18 => Ok(ConstantInfo::InvokeDynamic(ConstantInvokeDynamic::read(reader)?)),
                _ => Err(ClassLoadError::new("Cannot match constant tag"))
            }?;
            constants.push(constant);
        }

        Ok(constants)
    }

    pub fn read_interface_indices<R: ReadBytesExt>(reader: &mut R) -> Result<Vec<u16>, ClassLoadError> {
        let interfaces_count = reader.read_u16::<BigEndian>()? as usize;
        let mut interface_indices = vec![0; interfaces_count];
        reader.read_u16_into::<BigEndian>(&mut interface_indices)?;

        return Ok(interface_indices);
    }

    pub fn read_fields<R: ReadBytesExt>(reader: &mut R, constant_pool: &ConstantPool) -> Result<Vec<FieldInfo>, ClassLoadError> {
        let field_count = reader.read_u16::<BigEndian>()? as usize;

        let mut fields = Vec::with_capacity(field_count);
        for _i in 0..field_count {
            let field = FieldInfo::read(reader, constant_pool)?;
            fields.push(field);
        }

        return Ok(fields);
    }

    fn read_methods<R: ReadBytesExt>(reader: &mut R, constant_pool: &ConstantPool) -> Result<Vec<MethodInfo>, ClassLoadError> {
        let method_count = reader.read_u16::<BigEndian>()? as usize;

        let mut methods = Vec::with_capacity(method_count);
        for _i in 0..method_count {
            let method = MethodInfo::read(reader, constant_pool)?;
            methods.push(method);
        }

        return Ok(methods);
    }

    fn read_attributes<R: ReadBytesExt>(reader: &mut R, constant_pool: &ConstantPool) -> Result<Vec<Attribute>, ClassLoadError> {
        let attributes_count = reader.read_u16::<BigEndian>()? as usize;
        let mut attributes = Vec::with_capacity(attributes_count);
        for _i in 0..attributes_count {
            let attribute = Attribute::read(reader, constant_pool)?;
            attributes.push(attribute);
        }

        return Ok(attributes);
    }

    pub fn read<R: ReadBytesExt>(reader: &mut R) -> Result<Class, ClassLoadError> {
        let magic = reader.read_u32::<BigEndian>()?;
        if magic == CLASS_MAGIC {
            println!("Magic header is matching")
        } else {
            return Err(ClassLoadError::new("Magic header is not matching"));
        }

        let minor_version = reader.read_u16::<BigEndian>()?;
        let major_version = reader.read_u16::<BigEndian>()?;
        let constant_pool = Class::read_constant_pool(reader)?;
        let access_flags = reader.read_u16::<BigEndian>()?;
        let access_flags = ClassAccessFlags::from_bits(access_flags).ok_or(ClassLoadError::new("Invalid class access flags"))?;
        let this_class = reader.read_u16::<BigEndian>()?;
        let super_class = reader.read_u16::<BigEndian>()?;
        let interface_indices = Class::read_interface_indices(reader)?;
        let fields = Class::read_fields(reader, &constant_pool)?;
        let methods = Class::read_methods(reader, &constant_pool)?;
        let attributes = Class::read_attributes(reader, &constant_pool)?;

        let mut rest = Vec::new();
        reader.read_to_end(&mut rest);
        if !rest.is_empty() {
            return Err(ClassLoadError::new("Data is still present after reading class file"));
        }

        return Ok(Class {
            minor_version,
            major_version,
            constant_pool,
            access_flags,
            this_class,
            super_class,
            interface_indices,
            fields,
            methods,
            attributes,
        });
    }
}