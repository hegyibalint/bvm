// =============================================================================
// ATTRIBUTES
// =============================================================================

// ConstantValue Attribute -----------------------------------------------------

use byteorder::{BigEndian, ReadBytesExt};

use crate::class::{ClassLoadError, ReadAll, ReadOne};
use crate::class::constant_pool::{Constant, ConstantPool, ConstantPoolContext};

// =============================================================================
// CONTEXT
// =============================================================================

struct AttributeContext {
    pub constant_pool: &'static ConstantPool,
    pub name_index: usize,
    pub length: usize,
}

// =============================================================================
// ATTRIBUTES
// =============================================================================

// ConstantValue Attribute -----------------------------------------------------

#[derive(Debug)]
pub struct ConstantValueAttribute {
    const_value_index: u16,
}

impl ReadOne<AttributeContext> for ConstantValueAttribute {
    fn read_one<R: ReadBytesExt>(reader: &mut R, context: &AttributeContext) -> Result<Self, ClassLoadError> {
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

impl ReadOne<AttributeContext> for ExceptionTable {
    fn read_one<R: ReadBytesExt>(reader: &mut R, context: &AttributeContext) -> Result<Self, ClassLoadError> {
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

impl ReadAll<AttributeContext> for ExceptionTable {}

#[derive(Debug)]
pub struct CodeAttribute {
    max_stack: u16,
    max_locals: u16,
    code: Vec<u8>,
    exception_tables: Vec<ExceptionTable>,
    attributes: Vec<Attribute>,
}

impl ReadOne<AttributeContext> for CodeAttribute {
    fn read_one<R: ReadBytesExt>(reader: &mut R, context: &AttributeContext) -> Result<Self, ClassLoadError> {
        let max_stack = reader.read_u16::<BigEndian>()?;
        let max_locals = reader.read_u16::<BigEndian>()?;

        let code_length = reader.read_u32::<BigEndian>()? as usize;
        let mut code = vec![0; code_length];
        reader.read_exact(&mut code)?;

        let exception_tables = ExceptionTable::read_all(reader, context)?;

        let const_pool_context = ConstantPoolContext {
            constant_pool: context.constant_pool,
        };
        let attributes = Attribute::read_all(reader, &const_pool_context)?;

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

// pub struct Same {
//     offset: u8
// }
//
// impl ReadOne<AttributeContext> for Same {
//     fn read_one<R: ReadBytesExt>(reader: &mut R, context: &AttributeContext) -> Result<Self, ClassLoadError> {
//         todo!()
//     }
// }
//
// #[derive(Debug)]
// pub enum StackMapFrame {
//     Same,
//     SameLocalsOneStackItem,
//     SameLocalsOneStackItemExtended,
//     Chop,
//     SameFrameExtended,
//     Append,
//     FullFrame,
// }
//
// impl ReadOne<AttributeContext> for StackMapFrame {
//     fn read_one<R: ReadBytesExt>(reader: &mut R, context: &AttributeContext) -> Result<Self, ClassLoadError> {
//         let tag = reader.read_u8()?;
//         match tag {
//             _ if 0 <= tag && tag < 64 => Same()
//         }
//     }
// }
//
// impl ReadAll<AttributeContext> for StackMapFrame {}

// Exceptions Attribute --------------------------------------------------------

#[derive(Debug)]
pub struct ExceptionIndex {
    index: u16,
}

impl ReadOne<AttributeContext> for ExceptionIndex {
    fn read_one<R: ReadBytesExt>(reader: &mut R, context: &AttributeContext) -> Result<Self, ClassLoadError> {
        let index = reader.read_u16::<BigEndian>()?;
        Ok(ExceptionIndex {
            index
        })
    }
}

impl ReadAll<AttributeContext> for ExceptionIndex {}

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

impl ReadOne<AttributeContext> for InnerClass {
    fn read_one<R: ReadBytesExt>(reader: &mut R, context: &AttributeContext) -> Result<Self, ClassLoadError> {
        let inner_class_info_index = reader.read_u16::<BigEndian>()?;
        let outer_class_info_index = reader.read_u16::<BigEndian>()?;
        let inner_name_index = reader.read_u16::<BigEndian>()?;
        let inner_class_access_flags = reader.read_u16::<BigEndian>()?;
        let inner_class_access_flags = InnerClassAccessFlags::from_bits(inner_class_access_flags).ok_or(ClassLoadError::new("Invalid inner class access flags"))?;

        Ok(InnerClass {
            inner_class_info_index,
            outer_class_info_index,
            inner_name_index,
            inner_class_access_flags,
        })
    }
}

impl ReadAll<AttributeContext> for InnerClass {}

// EnclosingMethod Attribute ---------------------------------------------------

#[derive(Debug)]
pub struct EnclosingMethodAttribute {
    class_index: u16,
    method_index: u16,
}

impl ReadOne<AttributeContext> for EnclosingMethodAttribute {
    fn read_one<R: ReadBytesExt>(reader: &mut R, context: &AttributeContext) -> Result<Self, ClassLoadError> {
        let class_index = reader.read_u16::<BigEndian>()?;
        let method_index = reader.read_u16::<BigEndian>()?;

        Ok(EnclosingMethodAttribute {
            class_index,
            method_index,
        })
    }
}

// Signature Attribute ---------------------------------------------------------

#[derive(Debug)]
pub struct SignatureAttribute {
    signature_index: u16,
}

impl ReadOne<AttributeContext> for SignatureAttribute {
    fn read_one<R: ReadBytesExt>(reader: &mut R, context: &AttributeContext) -> Result<Self, ClassLoadError> {
        let signature_index = reader.read_u16::<BigEndian>()?;

        Ok(SignatureAttribute {
            signature_index
        })
    }
}

// SourceFile Attribute --------------------------------------------------------

#[derive(Debug)]
pub struct SourceFileAttribute {
    sourcefile_index: u16,
}

impl ReadOne<AttributeContext> for SourceFileAttribute {
    fn read_one<R: ReadBytesExt>(reader: &mut R, context: &AttributeContext) -> Result<Self, ClassLoadError> {
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

impl ReadOne<AttributeContext> for SourceDebugExtensionAttribute {
    fn read_one<R: ReadBytesExt>(reader: &mut R, context: &AttributeContext) -> Result<Self, ClassLoadError> {
        let mut debug_info = vec![0; context.length];
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
    line_number: u16,
}

impl ReadOne<AttributeContext> for LineNumberTable {
    fn read_one<R: ReadBytesExt>(reader: &mut R, context: &AttributeContext) -> Result<Self, ClassLoadError> {
        let start_pc = reader.read_u16::<BigEndian>()?;
        let line_number = reader.read_u16::<BigEndian>()?;

        Ok(LineNumberTable {
            start_pc,
            line_number,
        })
    }
}

impl ReadAll<AttributeContext> for LineNumberTable {}

// LocalVariableTable Attribute ------------------------------------------------

#[derive(Debug)]
pub struct LocalVariableTable {
    start_pc: u16,
    length: u16,
    name_index: u16,
    descriptor_index: u16,
    index: u16,
}

impl ReadOne<AttributeContext> for LocalVariableTable {
    fn read_one<R: ReadBytesExt>(reader: &mut R, context: &AttributeContext) -> Result<Self, ClassLoadError> {
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
            index,
        })
    }
}

impl ReadAll<AttributeContext> for LocalVariableTable {}

// LocalVariableTypeTable Attribute --------------------------------------------

#[derive(Debug)]
pub struct LocalVariableTypeTable {
    start_pc: u16,
    length: u16,
    name_index: u16,
    signature_index: u16,
    index: u16,
}

impl ReadOne<AttributeContext> for LocalVariableTypeTable {
    fn read_one<R: ReadBytesExt>(reader: &mut R, context: &AttributeContext) -> Result<Self, ClassLoadError> {
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
            index,
        })
    }
}

impl ReadAll<AttributeContext> for LocalVariableTypeTable {}

// Annotations Attribute - Commons ---------------------------------------------

#[derive(Debug)]
pub struct ConstantElementValue {
    const_value_index: u16,
}

impl ReadOne<AttributeContext> for ConstantElementValue {
    fn read_one<R: ReadBytesExt>(reader: &mut R, context: &AttributeContext) -> Result<Self, ClassLoadError> {
        let const_value_index = reader.read_u16::<BigEndian>()?;

        Ok(ConstantElementValue {
            const_value_index
        })
    }
}

#[derive(Debug)]
pub struct EnumElementValue {
    type_name_index: u16,
    const_name_index: u16,
}

impl ReadOne<AttributeContext> for EnumElementValue {
    fn read_one<R: ReadBytesExt>(reader: &mut R, context: &AttributeContext) -> Result<Self, ClassLoadError> {
        let type_name_index = reader.read_u16::<BigEndian>()?;
        let const_name_index = reader.read_u16::<BigEndian>()?;

        Ok(EnumElementValue {
            type_name_index,
            const_name_index,
        })
    }
}

#[derive(Debug)]
pub struct ClassElementValue {
    class_info_index: u16,
}

impl ReadOne<AttributeContext> for ClassElementValue {
    fn read_one<R: ReadBytesExt>(reader: &mut R, context: &AttributeContext) -> Result<Self, ClassLoadError> {
        let class_info_index = reader.read_u16::<BigEndian>()?;

        Ok(ClassElementValue {
            class_info_index
        })
    }
}

#[derive(Debug)]
pub struct AnnotationElementValue {
    annotation: Annotation,
}

impl ReadOne<AttributeContext> for AnnotationElementValue {
    fn read_one<R: ReadBytesExt>(reader: &mut R, context: &AttributeContext) -> Result<Self, ClassLoadError> {
        let annotation = Annotation::read_one(reader, context)?;

        Ok(AnnotationElementValue {
            annotation
        })
    }
}

#[derive(Debug)]
pub struct ArrayElementValue {
    array_values: Vec<ElementValue>,
}

impl ReadOne<AttributeContext> for ArrayElementValue {
    fn read_one<R: ReadBytesExt>(reader: &mut R, context: &AttributeContext) -> Result<Self, ClassLoadError> {
        let array_values = ElementValue::read_all(reader, context)?;

        Ok(ArrayElementValue {
            array_values
        })
    }
}

#[derive(Debug)]
pub enum ElementValue {
    Constant(ConstantElementValue),
    Enum(EnumElementValue),
    Class(ClassElementValue),
    Annotation(AnnotationElementValue),
    Array(ArrayElementValue),
}

impl ReadOne<AttributeContext> for ElementValue {
    fn read_one<R: ReadBytesExt>(reader: &mut R, context: &AttributeContext) -> Result<Self, ClassLoadError> {
        let tag = reader.read_u8()? as char;

        match tag {
            'B' | 'C' | 'D' | 'F' | 'I' | 'J' | 'S' | 'Z' | 's' => Ok(ElementValue::Constant(ConstantElementValue::read_one(reader, context)?)),
            'e' => Ok(ElementValue::Enum(EnumElementValue::read_one(reader, context)?)),
            'c' => Ok(ElementValue::Class(ClassElementValue::read_one(reader, context)?)),
            '@' => Ok(ElementValue::Annotation(AnnotationElementValue::read_one(reader, context)?)),
            '[' => Ok(ElementValue::Array(ArrayElementValue::read_one(reader, context)?)),
            _ => Err(ClassLoadError::new("Unknown tag for annotation element value"))
        }
    }
}

impl ReadAll<AttributeContext> for ElementValue {}

#[derive(Debug)]
pub struct ElementValuePair {
    element_name_index: u16,
    value: ElementValue,
}

impl ReadOne<AttributeContext> for ElementValuePair {
    fn read_one<R: ReadBytesExt>(reader: &mut R, context: &AttributeContext) -> Result<Self, ClassLoadError> {
        let element_name_index = reader.read_u16::<BigEndian>()?;
        let value = ElementValue::read_one(reader, context)?;

        Ok(ElementValuePair {
            element_name_index,
            value,
        })
    }
}

impl ReadAll<AttributeContext> for ElementValuePair {}

// Annotations Attribute - Annotations -----------------------------------------
// Covers:
//  - RuntimeVisibleAnnotations
//  - RuntimeInvisibleAnnotations

#[derive(Debug)]
pub struct Annotation {
    type_index: u16,
    element_value_pairs: Vec<ElementValuePair>,
}

impl ReadOne<AttributeContext> for Annotation {
    fn read_one<R: ReadBytesExt>(reader: &mut R, context: &AttributeContext) -> Result<Self, ClassLoadError> {
        let type_index = reader.read_u16::<BigEndian>()?;
        let element_value_pairs = ElementValuePair::read_all(reader, context)?;

        Ok(Annotation {
            type_index,
            element_value_pairs,
        })
    }
}

impl ReadAll<AttributeContext> for Annotation {}

// Annotations Attribute - Parameter -------------------------------------------
// Covers:
//  - RuntimeVisibleParameterAnnotations
//  - RuntimeInvisibleParameterAnnotations

#[derive(Debug)]
pub struct ParameterAnnotation {
    annotations: Vec<Annotation>,
}

impl ReadOne<AttributeContext> for ParameterAnnotation {
    fn read_one<R: ReadBytesExt>(reader: &mut R, context: &AttributeContext) -> Result<Self, ClassLoadError> {
        let annotations = Annotation::read_all(reader, context)?;

        Ok(ParameterAnnotation {
            annotations
        })
    }
}

impl ReadAll<AttributeContext> for ParameterAnnotation {
    fn read_count<R: ReadBytesExt>(reader: &mut R) -> Result<usize, ClassLoadError> {
        let count = reader.read_u8()? as usize;
        Ok(count)
    }
}

// Annotations Attribute - Default ---------------------------------------------

#[derive(Debug)]
pub struct AnnotationDefault {
    default_value: ElementValue,
}

impl ReadOne<AttributeContext> for AnnotationDefault {
    fn read_one<R: ReadBytesExt>(reader: &mut R, context: &AttributeContext) -> Result<Self, ClassLoadError> {
        let default_value = ElementValue::read_one(reader, context)?;

        Ok(AnnotationDefault {
            default_value
        })
    }
}

// Bootstrap Methods -----------------------------------------------------------

#[derive(Debug)]
pub struct BootstrapMethod {
    bootstrap_method_ref: u16,
    bootstrap_arguments: Vec<u16>,
}

impl ReadOne<AttributeContext> for BootstrapMethod {
    fn read_one<R: ReadBytesExt>(reader: &mut R, context: &AttributeContext) -> Result<Self, ClassLoadError> {
        let bootstrap_method_ref = reader.read_u16::<BigEndian>()?;

        let bootstrap_argument_count = reader.read_u16::<BigEndian>()? as usize;
        let mut bootstrap_arguments = vec![0; bootstrap_argument_count];
        reader.read_u16_into::<BigEndian>(&mut bootstrap_arguments)?;

        Ok(BootstrapMethod {
            bootstrap_method_ref,
            bootstrap_arguments,
        })
    }
}

impl ReadAll<AttributeContext> for BootstrapMethod {}

// Misc Attribute --------------------------------------------------------------

#[derive(Debug)]
pub struct MiscAttribute {
    name_index: usize,
    info: Vec<u8>,
}

impl ReadOne<AttributeContext> for MiscAttribute {
    fn read_one<R: ReadBytesExt>(reader: &mut R, context: &AttributeContext) -> Result<Self, ClassLoadError> {
        let mut info = vec![0; context.length];
        reader.read_exact(&mut info)?;

        Ok(MiscAttribute {
            name_index: context.name_index,
            info,
        })
    }
}

// Attribute -------------------------------------------------------------------

#[derive(Debug)]
pub enum Attribute {
    ConstantValue(ConstantValueAttribute),
    Code(CodeAttribute),
    //StackMapTable(StackMapTableAttribute),
    Exceptions(Vec<ExceptionIndex>),
    InnerClasses(Vec<InnerClass>),
    EnclosingMethod(EnclosingMethodAttribute),
    Synthetic(),
    Signature(SignatureAttribute),
    SourceFile(SourceFileAttribute),
    SourceDebugExtension(SourceDebugExtensionAttribute),
    LineNumberTable(Vec<LineNumberTable>),
    LocalVariableTable(Vec<LocalVariableTable>),
    LocalVariableTypeTable(Vec<LocalVariableTypeTable>),
    Deprecated(),
    RuntimeVisibleAnnotations(Vec<Annotation>),
    RuntimeInvisibleAnnotations(Vec<Annotation>),
    RuntimeVisibleParameterAnnotations(Vec<ParameterAnnotation>),
    RuntimeInvisibleParameterAnnotations(Vec<ParameterAnnotation>),
    AnnotationDefault(AnnotationDefault),
    BootstrapMethods(Vec<BootstrapMethod>),
    Misc(MiscAttribute),
}

impl ReadOne<ConstantPoolContext> for Attribute {
    fn read_one<R: ReadBytesExt>(reader: &mut R, context: &ConstantPoolContext) -> Result<Self, ClassLoadError> {
        let attribute_name_index = reader.read_u16::<BigEndian>()? as usize;
        let attribute_length = reader.read_u32::<BigEndian>()? as usize;

        // Dereference the name from the constant pool
        let attribute_name = match &context.constant_pool[attribute_name_index - 1] {
            // If the referenced constant is an UTF8 reference, we are up to spec
            Constant::Utf8(value) => Ok(String::from_utf8(value.bytes.clone())?),
            // Otherwise, we blow up, as nothing else is acceptable
            _ => Err(ClassLoadError::new("Referenced attribute name should be an UTF-8 constant"))
        }?;

        let attribute_context = AttributeContext {
            constant_pool: context.constant_pool,
            name_index: attribute_name_index,
            length: attribute_length,
        };

        let attribute = match attribute_name.as_str() {
            "ConstantValue" => Attribute::ConstantValue(ConstantValueAttribute::read_one(reader, &attribute_context)?),
            "Code" => Attribute::Code(CodeAttribute::read_one(reader, &attribute_context)?),
            //"StackMapTable" => Attribute::StackMapTable(StackMapTableAttribute::read_one(reader, &attribute_context)?),
            "Exceptions" => Attribute::Exceptions(ExceptionIndex::read_all(reader, &attribute_context)?),
            "InnerClasses" => Attribute::InnerClasses(InnerClass::read_all(reader, &attribute_context)?),
            "EnclosingMethod" => Attribute::EnclosingMethod(EnclosingMethodAttribute::read_one(reader, &attribute_context)?),
            "Synthetic" => Attribute::Synthetic(),
            "Signature" => Attribute::Signature(SignatureAttribute::read_one(reader, &attribute_context)?),
            "SourceFile" => Attribute::SourceFile(SourceFileAttribute::read_one(reader, &attribute_context)?),
            "SourceDebugExtension" => Attribute::SourceDebugExtension(SourceDebugExtensionAttribute::read_one(reader, &attribute_context)?),
            "LineNumberTable" => Attribute::LineNumberTable(LineNumberTable::read_all(reader, &attribute_context)?),
            "LocalVariableTable" => Attribute::LocalVariableTable(LocalVariableTable::read_all(reader, &attribute_context)?),
            "LocalVariableTypeTable" => Attribute::LocalVariableTypeTable(LocalVariableTypeTable::read_all(reader, &attribute_context)?),
            "Deprecated" => Attribute::Deprecated(),
            "RuntimeVisibleAnnotations" => Attribute::RuntimeVisibleAnnotations(Annotation::read_all(reader, &attribute_context)?),
            "RuntimeInvisibleAnnotations" => Attribute::RuntimeInvisibleAnnotations(Annotation::read_all(reader, &attribute_context)?),
            "RuntimeVisibleParameterAnnotations" => Attribute::RuntimeVisibleParameterAnnotations(ParameterAnnotation::read_all(reader, &attribute_context)?),
            "RuntimeInvisibleParameterAnnotations" => Attribute::RuntimeInvisibleParameterAnnotations(ParameterAnnotation::read_all(reader, &attribute_context)?),
            "AnnotationDefault" => Attribute::AnnotationDefault(AnnotationDefault::read_one(reader, &attribute_context)?),
            "BootstrapMethods" => Attribute::BootstrapMethods(BootstrapMethod::read_all(reader, &attribute_context)?),
            _ => Attribute::Misc(MiscAttribute::read_one(reader, &attribute_context)?)
        };
        Ok(attribute)
    }
}

impl ReadAll<ConstantPoolContext> for Attribute {}
