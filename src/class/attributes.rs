// =============================================================================
// ATTRIBUTES
// =============================================================================

// ConstantValue Attribute -----------------------------------------------------

use byteorder::{BigEndian, ReadBytesExt};

use crate::class::attributes::VerificationType::{
    Double, Float, Integer, Long, Null, Object, Top, Uninitialized, UninitializedThis,
};
use crate::class::constant_pool::{Constant, ConstantPool, ConstantPoolContext};
use crate::class::{ClassLoadingError, EmptyContext, ReadAll, ReadOne};

// =============================================================================
// CONTEXT
// =============================================================================

/// Context usable when reading [Attribute] elements.
struct AttributeContext<'a> {
    pub constant_pool: &'a ConstantPool,
    pub name_index: usize,
    pub length: usize,
}

/// Context usable when reading [StackMapTableAttribute] attributes.
#[derive(Debug)]
struct StackFrameContext {
    frame_type: u8,
}

// =============================================================================
// ATTRIBUTES
// =============================================================================

// ConstantValue Attribute -----------------------------------------------------

#[derive(Debug)]
pub struct ConstantValueAttribute {
    const_value_index: u16,
}

impl ReadOne<AttributeContext<'_>> for ConstantValueAttribute {
    fn read_one<R: ReadBytesExt>(
        reader: &mut R,
        _context: &AttributeContext,
    ) -> Result<Self, ClassLoadingError> {
        let const_value_index = reader.read_u16::<BigEndian>()?;
        Ok(ConstantValueAttribute { const_value_index })
    }
}

// Code Attribute --------------------------------------------------------------

#[derive(Debug)]
pub struct ExceptionTableAttribute {
    start_pc: u16,
    end_pc: u16,
    handler_pc: u16,
    catch_type: u16,
}

impl ReadOne<AttributeContext<'_>> for ExceptionTableAttribute {
    fn read_one<R: ReadBytesExt>(
        reader: &mut R,
        _context: &AttributeContext,
    ) -> Result<Self, ClassLoadingError> {
        let start_pc = reader.read_u16::<BigEndian>()?;
        let end_pc = reader.read_u16::<BigEndian>()?;
        let handler_pc = reader.read_u16::<BigEndian>()?;
        let catch_type = reader.read_u16::<BigEndian>()?;

        Ok(ExceptionTableAttribute {
            start_pc,
            end_pc,
            handler_pc,
            catch_type,
        })
    }
}

impl ReadAll<AttributeContext<'_>> for ExceptionTableAttribute {}

#[derive(Debug)]
pub struct CodeAttribute {
    max_stack: u16,
    max_locals: u16,
    code: Vec<u8>,
    exception_tables: Vec<ExceptionTableAttribute>,
    attributes: Vec<Attribute>,
}

impl ReadOne<AttributeContext<'_>> for CodeAttribute {
    fn read_one<R: ReadBytesExt>(
        reader: &mut R,
        context: &AttributeContext,
    ) -> Result<Self, ClassLoadingError> {
        let max_stack = reader.read_u16::<BigEndian>()?;
        let max_locals = reader.read_u16::<BigEndian>()?;

        let code_length = reader.read_u32::<BigEndian>()? as usize;
        let mut code = vec![0; code_length];
        reader.read_exact(&mut code)?;

        let exception_tables = ExceptionTableAttribute::read_all(reader, context)?;

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

#[derive(Debug)]
pub struct ObjectVariableInfo {
    pub constant_index: u16,
}

impl ReadOne<EmptyContext> for ObjectVariableInfo {
    fn read_one<R: ReadBytesExt>(
        reader: &mut R,
        _context: &EmptyContext,
    ) -> Result<Self, ClassLoadingError> {
        let cpool_index = reader.read_u16::<BigEndian>()?;
        Ok(ObjectVariableInfo {
            constant_index: cpool_index,
        })
    }
}

#[derive(Debug)]
pub struct UninitializedVariableInfo {
    pub offset: u16,
}

impl ReadOne<EmptyContext> for UninitializedVariableInfo {
    fn read_one<R: ReadBytesExt>(
        reader: &mut R,
        _context: &EmptyContext,
    ) -> Result<Self, ClassLoadingError> {
        let offset = reader.read_u16::<BigEndian>()?;
        Ok(UninitializedVariableInfo { offset })
    }
}

#[derive(Debug)]
pub enum VerificationType {
    Top,
    Integer,
    Float,
    Long,
    Double,
    Null,
    UninitializedThis,
    Object(ObjectVariableInfo),
    Uninitialized(UninitializedVariableInfo),
}

impl ReadOne<EmptyContext> for VerificationType {
    fn read_one<R: ReadBytesExt>(
        reader: &mut R,
        _context: &EmptyContext,
    ) -> Result<Self, ClassLoadingError> {
        let tag = reader.read_u8()?;
        match tag {
            0 => Ok(Top),
            1 => Ok(Integer),
            2 => Ok(Float),
            5 => Ok(Null),
            6 => Ok(UninitializedThis),
            7 => Ok(Object(ObjectVariableInfo::read_one(
                reader,
                &EmptyContext::default(),
            )?)),
            8 => Ok(Uninitialized(UninitializedVariableInfo::read_one(
                reader,
                &EmptyContext::default(),
            )?)),
            4 => Ok(Long),
            3 => Ok(Double),
            _ => Err(ClassLoadingError::new("Cannot determine verification type")),
        }
    }
}

impl ReadAll<EmptyContext> for VerificationType {}

#[derive(Debug)]
pub struct SameFrame {
    offset_delta: u8,
}

impl ReadOne<StackFrameContext> for SameFrame {
    fn read_one<R: ReadBytesExt>(
        reader: &mut R,
        context: &StackFrameContext,
    ) -> Result<Self, ClassLoadingError> {
        let offset_delta = context.frame_type;
        Ok(SameFrame { offset_delta })
    }
}

#[derive(Debug)]
pub struct SameLocalsOneStackItemFrame {
    offset_delta: u8,
    stack: VerificationType,
}

impl ReadOne<StackFrameContext> for SameLocalsOneStackItemFrame {
    fn read_one<R: ReadBytesExt>(
        reader: &mut R,
        context: &StackFrameContext,
    ) -> Result<Self, ClassLoadingError> {
        let offset_delta = context.frame_type - 64;
        let stack = VerificationType::read_one(reader, &EmptyContext::default())?;
        Ok(SameLocalsOneStackItemFrame {
            offset_delta,
            stack,
        })
    }
}

#[derive(Debug)]
pub struct SameLocalsOneStackItemExtendedFrame {
    offset_delta: u16,
    stack: VerificationType,
}

impl ReadOne<EmptyContext> for SameLocalsOneStackItemExtendedFrame {
    fn read_one<R: ReadBytesExt>(
        reader: &mut R,
        _context: &EmptyContext,
    ) -> Result<Self, ClassLoadingError> {
        let offset_delta = reader.read_u16::<BigEndian>()?;
        let stack = VerificationType::read_one(reader, &EmptyContext::default())?;
        Ok(SameLocalsOneStackItemExtendedFrame {
            offset_delta,
            stack,
        })
    }
}

#[derive(Debug)]
pub struct ChopFrame {
    offset_delta: u16,
}

impl ReadOne<EmptyContext> for ChopFrame {
    fn read_one<R: ReadBytesExt>(
        reader: &mut R,
        _context: &EmptyContext,
    ) -> Result<Self, ClassLoadingError> {
        let offset_delta = reader.read_u16::<BigEndian>()?;
        Ok(ChopFrame { offset_delta })
    }
}

#[derive(Debug)]
pub struct SameExtendedFrame {
    offset_delta: u16,
}

impl ReadOne<EmptyContext> for SameExtendedFrame {
    fn read_one<R: ReadBytesExt>(
        reader: &mut R,
        _context: &EmptyContext,
    ) -> Result<Self, ClassLoadingError> {
        let offset_delta = reader.read_u16::<BigEndian>()?;
        Ok(SameExtendedFrame { offset_delta })
    }
}

#[derive(Debug)]
pub struct AppendFrame {
    offset_delta: u16,
    locals: Vec<VerificationType>,
}

impl ReadOne<StackFrameContext> for AppendFrame {
    fn read_one<R: ReadBytesExt>(
        reader: &mut R,
        context: &StackFrameContext,
    ) -> Result<Self, ClassLoadingError> {
        let offset_delta = reader.read_u16::<BigEndian>()?;
        let mut locals = Vec::new();
        for _ in 0..(context.frame_type - 251) {
            locals.push(VerificationType::read_one(
                reader,
                &EmptyContext::default(),
            )?);
        }
        Ok(AppendFrame {
            offset_delta,
            locals,
        })
    }
}

#[derive(Debug)]
pub struct FullFrame {
    offset_delta: u16,
    locals: Vec<VerificationType>,
    stack: Vec<VerificationType>,
}

impl ReadOne<EmptyContext> for FullFrame {
    fn read_one<R: ReadBytesExt>(
        reader: &mut R,
        _context: &EmptyContext,
    ) -> Result<Self, ClassLoadingError> {
        let offset_delta = reader.read_u16::<BigEndian>()?;
        let locals = VerificationType::read_all(reader, &EmptyContext::default())?;
        let stack = VerificationType::read_all(reader, &EmptyContext::default())?;

        Ok(FullFrame {
            offset_delta,
            locals,
            stack,
        })
    }
}

#[derive(Debug)]
pub enum StackMapTableAttribute {
    Same(SameFrame),
    SameLocalsOneStackItem(SameLocalsOneStackItemFrame),
    SameLocalsOneStackItemExtended(SameLocalsOneStackItemExtendedFrame),
    Chop(ChopFrame),
    SameExtended(SameExtendedFrame),
    Append(AppendFrame),
    Full(FullFrame),
}

impl ReadOne<AttributeContext<'_>> for StackMapTableAttribute {
    fn read_one<R: ReadBytesExt>(
        reader: &mut R,
        _context: &AttributeContext,
    ) -> Result<Self, ClassLoadingError> {
        let frame_type = reader.read_u8()?;
        let frame_context = StackFrameContext { frame_type };

        let frame = match frame_type {
            0..=63 => Ok(StackMapTableAttribute::Same(SameFrame::read_one(
                reader,
                &frame_context,
            )?)),
            64..=127 => Ok(StackMapTableAttribute::SameLocalsOneStackItem(
                SameLocalsOneStackItemFrame::read_one(reader, &frame_context)?,
            )),
            128..=246 => Err(ClassLoadingError::new(
                format!("Reserved frame type {}", frame_type).as_str(),
            )),
            247 => Ok(StackMapTableAttribute::SameLocalsOneStackItemExtended(
                SameLocalsOneStackItemExtendedFrame::read_one(reader, &EmptyContext::default())?,
            )),
            248..=250 => Ok(StackMapTableAttribute::Chop(ChopFrame::read_one(
                reader,
                &EmptyContext::default(),
            )?)),
            251 => Ok(StackMapTableAttribute::SameExtended(
                SameExtendedFrame::read_one(reader, &EmptyContext::default())?,
            )),
            252..=254 => Ok(StackMapTableAttribute::Append(AppendFrame::read_one(
                reader,
                &frame_context,
            )?)),
            255 => Ok(StackMapTableAttribute::Full(FullFrame::read_one(
                reader,
                &EmptyContext::default(),
            )?)),
            value => Err(ClassLoadingError::new(
                format!("Unknown frame type {}", value).as_str(),
            )),
        };

        return frame;
    }
}

impl ReadAll<AttributeContext<'_>> for StackMapTableAttribute {}

// Exceptions Attribute --------------------------------------------------------

#[derive(Debug)]
pub struct ExceptionIndexAttribute {
    index: u16,
}

impl ReadOne<AttributeContext<'_>> for ExceptionIndexAttribute {
    fn read_one<R: ReadBytesExt>(
        reader: &mut R,
        _context: &AttributeContext,
    ) -> Result<Self, ClassLoadingError> {
        let index = reader.read_u16::<BigEndian>()?;
        Ok(ExceptionIndexAttribute { index })
    }
}

impl ReadAll<AttributeContext<'_>> for ExceptionIndexAttribute {}

// InnerClasses Attribute ------------------------------------------------------

bitflags::bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
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
pub struct InnerClassAttribute {
    inner_class_info_index: u16,
    outer_class_info_index: u16,
    inner_name_index: u16,
    inner_class_access_flags: InnerClassAccessFlags,
}

impl ReadOne<AttributeContext<'_>> for InnerClassAttribute {
    fn read_one<R: ReadBytesExt>(
        reader: &mut R,
        _context: &AttributeContext,
    ) -> Result<Self, ClassLoadingError> {
        let inner_class_info_index = reader.read_u16::<BigEndian>()?;
        let outer_class_info_index = reader.read_u16::<BigEndian>()?;
        let inner_name_index = reader.read_u16::<BigEndian>()?;
        let inner_class_access_flags = reader.read_u16::<BigEndian>()?;
        let inner_class_access_flags =
            InnerClassAccessFlags::from_bits(inner_class_access_flags)
                .ok_or(ClassLoadingError::new("Invalid inner class access flags"))?;

        Ok(InnerClassAttribute {
            inner_class_info_index,
            outer_class_info_index,
            inner_name_index,
            inner_class_access_flags,
        })
    }
}

impl ReadAll<AttributeContext<'_>> for InnerClassAttribute {}

// EnclosingMethod Attribute ---------------------------------------------------

#[derive(Debug)]
pub struct EnclosingMethodAttribute {
    class_index: u16,
    method_index: u16,
}

impl ReadOne<AttributeContext<'_>> for EnclosingMethodAttribute {
    fn read_one<R: ReadBytesExt>(
        reader: &mut R,
        _context: &AttributeContext,
    ) -> Result<Self, ClassLoadingError> {
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

impl ReadOne<AttributeContext<'_>> for SignatureAttribute {
    fn read_one<R: ReadBytesExt>(
        reader: &mut R,
        _context: &AttributeContext,
    ) -> Result<Self, ClassLoadingError> {
        let signature_index = reader.read_u16::<BigEndian>()?;

        Ok(SignatureAttribute { signature_index })
    }
}

// SourceFile Attribute --------------------------------------------------------

#[derive(Debug)]
pub struct SourceFileAttribute {
    sourcefile_index: u16,
}

impl ReadOne<AttributeContext<'_>> for SourceFileAttribute {
    fn read_one<R: ReadBytesExt>(
        reader: &mut R,
        _context: &AttributeContext,
    ) -> Result<Self, ClassLoadingError> {
        let sourcefile_index = reader.read_u16::<BigEndian>()?;

        Ok(SourceFileAttribute { sourcefile_index })
    }
}

// SourceDebugExtension Attribute ----------------------------------------------

#[derive(Debug)]
pub struct SourceDebugExtensionAttribute {
    debug_info: Vec<u8>,
}

impl ReadOne<AttributeContext<'_>> for SourceDebugExtensionAttribute {
    fn read_one<R: ReadBytesExt>(
        reader: &mut R,
        context: &AttributeContext,
    ) -> Result<Self, ClassLoadingError> {
        let mut debug_info = vec![0; context.length];
        reader.read_exact(&mut debug_info)?;

        Ok(SourceDebugExtensionAttribute { debug_info })
    }
}

// LineNumberTable Attribute ---------------------------------------------------

#[derive(Debug)]
pub struct LineNumberTableAttribute {
    start_pc: u16,
    line_number: u16,
}

impl ReadOne<AttributeContext<'_>> for LineNumberTableAttribute {
    fn read_one<R: ReadBytesExt>(
        reader: &mut R,
        _context: &AttributeContext,
    ) -> Result<Self, ClassLoadingError> {
        let start_pc = reader.read_u16::<BigEndian>()?;
        let line_number = reader.read_u16::<BigEndian>()?;

        Ok(LineNumberTableAttribute {
            start_pc,
            line_number,
        })
    }
}

impl ReadAll<AttributeContext<'_>> for LineNumberTableAttribute {}

// LocalVariableTable Attribute ------------------------------------------------

#[derive(Debug)]
pub struct LocalVariableTableAttribute {
    start_pc: u16,
    length: u16,
    name_index: u16,
    descriptor_index: u16,
    index: u16,
}

impl ReadOne<AttributeContext<'_>> for LocalVariableTableAttribute {
    fn read_one<R: ReadBytesExt>(
        reader: &mut R,
        _context: &AttributeContext,
    ) -> Result<Self, ClassLoadingError> {
        let start_pc = reader.read_u16::<BigEndian>()?;
        let length = reader.read_u16::<BigEndian>()?;
        let name_index = reader.read_u16::<BigEndian>()?;
        let descriptor_index = reader.read_u16::<BigEndian>()?;
        let index = reader.read_u16::<BigEndian>()?;

        Ok(LocalVariableTableAttribute {
            start_pc,
            length,
            name_index,
            descriptor_index,
            index,
        })
    }
}

impl ReadAll<AttributeContext<'_>> for LocalVariableTableAttribute {}

// LocalVariableTypeTable Attribute --------------------------------------------

#[derive(Debug)]
pub struct LocalVariableTypeTableAttribute {
    start_pc: u16,
    length: u16,
    name_index: u16,
    signature_index: u16,
    index: u16,
}

impl ReadOne<AttributeContext<'_>> for LocalVariableTypeTableAttribute {
    fn read_one<R: ReadBytesExt>(
        reader: &mut R,
        _context: &AttributeContext,
    ) -> Result<Self, ClassLoadingError> {
        let start_pc = reader.read_u16::<BigEndian>()?;
        let length = reader.read_u16::<BigEndian>()?;
        let name_index = reader.read_u16::<BigEndian>()?;
        let signature_index = reader.read_u16::<BigEndian>()?;
        let index = reader.read_u16::<BigEndian>()?;

        Ok(LocalVariableTypeTableAttribute {
            start_pc,
            length,
            name_index,
            signature_index,
            index,
        })
    }
}

impl ReadAll<AttributeContext<'_>> for LocalVariableTypeTableAttribute {}

// Annotations Attribute - Commons ---------------------------------------------

#[derive(Debug)]
pub struct ConstantElementValueAttribute {
    const_value_index: u16,
}

impl ReadOne<AttributeContext<'_>> for ConstantElementValueAttribute {
    fn read_one<R: ReadBytesExt>(
        reader: &mut R,
        _context: &AttributeContext,
    ) -> Result<Self, ClassLoadingError> {
        let const_value_index = reader.read_u16::<BigEndian>()?;

        Ok(ConstantElementValueAttribute { const_value_index })
    }
}

#[derive(Debug)]
pub struct EnumElementValue {
    type_name_index: u16,
    const_name_index: u16,
}

impl ReadOne<AttributeContext<'_>> for EnumElementValue {
    fn read_one<R: ReadBytesExt>(
        reader: &mut R,
        _context: &AttributeContext,
    ) -> Result<Self, ClassLoadingError> {
        let type_name_index = reader.read_u16::<BigEndian>()?;
        let const_name_index = reader.read_u16::<BigEndian>()?;

        Ok(EnumElementValue {
            type_name_index,
            const_name_index,
        })
    }
}

#[derive(Debug)]
pub struct ClassElementValueAttribute {
    class_info_index: u16,
}

impl ReadOne<AttributeContext<'_>> for ClassElementValueAttribute {
    fn read_one<R: ReadBytesExt>(
        reader: &mut R,
        _context: &AttributeContext,
    ) -> Result<Self, ClassLoadingError> {
        let class_info_index = reader.read_u16::<BigEndian>()?;

        Ok(ClassElementValueAttribute { class_info_index })
    }
}

#[derive(Debug)]
pub struct AnnotationElementValue {
    annotation: AnnotationAttribute,
}

impl ReadOne<AttributeContext<'_>> for AnnotationElementValue {
    fn read_one<R: ReadBytesExt>(
        reader: &mut R,
        context: &AttributeContext,
    ) -> Result<Self, ClassLoadingError> {
        let annotation = AnnotationAttribute::read_one(reader, context)?;

        Ok(AnnotationElementValue { annotation })
    }
}

#[derive(Debug)]
pub struct ArrayElementValue {
    array_values: Vec<ElementValue>,
}

impl ReadOne<AttributeContext<'_>> for ArrayElementValue {
    fn read_one<R: ReadBytesExt>(
        reader: &mut R,
        context: &AttributeContext,
    ) -> Result<Self, ClassLoadingError> {
        let array_values = ElementValue::read_all(reader, context)?;

        Ok(ArrayElementValue { array_values })
    }
}

#[derive(Debug)]
pub enum ElementValue {
    Constant(ConstantElementValueAttribute),
    Enum(EnumElementValue),
    Class(ClassElementValueAttribute),
    Annotation(AnnotationElementValue),
    Array(ArrayElementValue),
}

impl ReadOne<AttributeContext<'_>> for ElementValue {
    fn read_one<R: ReadBytesExt>(
        reader: &mut R,
        context: &AttributeContext,
    ) -> Result<Self, ClassLoadingError> {
        let tag = reader.read_u8()? as char;

        match tag {
            'B' | 'C' | 'D' | 'F' | 'I' | 'J' | 'S' | 'Z' | 's' => Ok(ElementValue::Constant(
                ConstantElementValueAttribute::read_one(reader, context)?,
            )),
            'e' => Ok(ElementValue::Enum(EnumElementValue::read_one(
                reader, context,
            )?)),
            'c' => Ok(ElementValue::Class(ClassElementValueAttribute::read_one(
                reader, context,
            )?)),
            '@' => Ok(ElementValue::Annotation(AnnotationElementValue::read_one(
                reader, context,
            )?)),
            '[' => Ok(ElementValue::Array(ArrayElementValue::read_one(
                reader, context,
            )?)),
            _ => Err(ClassLoadingError::new(
                "Unknown tag for annotation element value",
            )),
        }
    }
}

impl ReadAll<AttributeContext<'_>> for ElementValue {}

#[derive(Debug)]
pub struct ElementValuePair {
    element_name_index: u16,
    value: ElementValue,
}

impl ReadOne<AttributeContext<'_>> for ElementValuePair {
    fn read_one<R: ReadBytesExt>(
        reader: &mut R,
        context: &AttributeContext,
    ) -> Result<Self, ClassLoadingError> {
        let element_name_index = reader.read_u16::<BigEndian>()?;
        let value = ElementValue::read_one(reader, context)?;

        Ok(ElementValuePair {
            element_name_index,
            value,
        })
    }
}

impl ReadAll<AttributeContext<'_>> for ElementValuePair {}

// Annotations Attribute - Annotations -----------------------------------------
// Covers:
//  - RuntimeVisibleAnnotations
//  - RuntimeInvisibleAnnotations

#[derive(Debug)]
pub struct AnnotationAttribute {
    type_index: u16,
    element_value_pairs: Vec<ElementValuePair>,
}

impl ReadOne<AttributeContext<'_>> for AnnotationAttribute {
    fn read_one<R: ReadBytesExt>(
        reader: &mut R,
        context: &AttributeContext,
    ) -> Result<Self, ClassLoadingError> {
        let type_index = reader.read_u16::<BigEndian>()?;
        let element_value_pairs = ElementValuePair::read_all(reader, context)?;

        Ok(AnnotationAttribute {
            type_index,
            element_value_pairs,
        })
    }
}

impl ReadAll<AttributeContext<'_>> for AnnotationAttribute {}

// Annotations Attribute - Parameter -------------------------------------------
// Covers:
//  - RuntimeVisibleParameterAnnotations
//  - RuntimeInvisibleParameterAnnotations

#[derive(Debug)]
pub struct ParameterAnnotationAttribute {
    annotations: Vec<AnnotationAttribute>,
}

impl ReadOne<AttributeContext<'_>> for ParameterAnnotationAttribute {
    fn read_one<R: ReadBytesExt>(
        reader: &mut R,
        context: &AttributeContext,
    ) -> Result<Self, ClassLoadingError> {
        let annotations = AnnotationAttribute::read_all(reader, context)?;

        Ok(ParameterAnnotationAttribute { annotations })
    }
}

impl ReadAll<AttributeContext<'_>> for ParameterAnnotationAttribute {
    fn read_count<R: ReadBytesExt>(reader: &mut R) -> Result<usize, ClassLoadingError> {
        let count = reader.read_u8()? as usize;
        Ok(count)
    }
}

// Annotations Attribute - Default ---------------------------------------------

#[derive(Debug)]
pub struct AnnotationDefaultAttribute {
    default_value: ElementValue,
}

impl ReadOne<AttributeContext<'_>> for AnnotationDefaultAttribute {
    fn read_one<R: ReadBytesExt>(
        reader: &mut R,
        context: &AttributeContext,
    ) -> Result<Self, ClassLoadingError> {
        let default_value = ElementValue::read_one(reader, context)?;

        Ok(AnnotationDefaultAttribute { default_value })
    }
}

// Bootstrap Methods -----------------------------------------------------------

#[derive(Debug)]
pub struct BootstrapMethodAttribute {
    bootstrap_method_ref: u16,
    bootstrap_arguments: Vec<u16>,
}

impl ReadOne<AttributeContext<'_>> for BootstrapMethodAttribute {
    fn read_one<R: ReadBytesExt>(
        reader: &mut R,
        _context: &AttributeContext,
    ) -> Result<Self, ClassLoadingError> {
        let bootstrap_method_ref = reader.read_u16::<BigEndian>()?;

        let bootstrap_argument_count = reader.read_u16::<BigEndian>()? as usize;
        let mut bootstrap_arguments = vec![0; bootstrap_argument_count];
        reader.read_u16_into::<BigEndian>(&mut bootstrap_arguments)?;

        Ok(BootstrapMethodAttribute {
            bootstrap_method_ref,
            bootstrap_arguments,
        })
    }
}

impl ReadAll<AttributeContext<'_>> for BootstrapMethodAttribute {}

// Misc Attribute --------------------------------------------------------------

#[derive(Debug)]
pub struct MiscAttribute {
    name_index: usize,
    info: Vec<u8>,
}

impl ReadOne<AttributeContext<'_>> for MiscAttribute {
    fn read_one<R: ReadBytesExt>(
        reader: &mut R,
        context: &AttributeContext,
    ) -> Result<Self, ClassLoadingError> {
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
    StackMapTable(Vec<StackMapTableAttribute>),
    Exceptions(Vec<ExceptionIndexAttribute>),
    InnerClasses(Vec<InnerClassAttribute>),
    EnclosingMethod(EnclosingMethodAttribute),
    Synthetic(),
    Signature(SignatureAttribute),
    SourceFile(SourceFileAttribute),
    SourceDebugExtension(SourceDebugExtensionAttribute),
    LineNumberTable(Vec<LineNumberTableAttribute>),
    LocalVariableTable(Vec<LocalVariableTableAttribute>),
    LocalVariableTypeTable(Vec<LocalVariableTypeTableAttribute>),
    Deprecated(),
    RuntimeVisibleAnnotations(Vec<AnnotationAttribute>),
    RuntimeInvisibleAnnotations(Vec<AnnotationAttribute>),
    RuntimeVisibleParameterAnnotations(Vec<ParameterAnnotationAttribute>),
    RuntimeInvisibleParameterAnnotations(Vec<ParameterAnnotationAttribute>),
    AnnotationDefault(AnnotationDefaultAttribute),
    BootstrapMethods(Vec<BootstrapMethodAttribute>),
    Misc(MiscAttribute),
}

impl<'a> ReadOne<ConstantPoolContext<'a>> for Attribute {
    fn read_one<R: ReadBytesExt>(
        reader: &mut R,
        context: &ConstantPoolContext<'a>,
    ) -> Result<Self, ClassLoadingError> {
        let attribute_name_index = reader.read_u16::<BigEndian>()? as usize;
        let attribute_length = reader.read_u32::<BigEndian>()? as usize;

        // Dereference the name from the constant pool
        let attribute_name = match &context.constant_pool[attribute_name_index] {
            // If the referenced constant is an UTF-8 reference, we are up to spec
            Constant::Utf8(value) => Ok(&value.string),
            // Otherwise, we blow up, as nothing else is acceptable
            _ => Err(ClassLoadingError::new(
                "Referenced attribute name should be an UTF-8 constant",
            )),
        }?;

        let attribute_context = AttributeContext {
            constant_pool: context.constant_pool,
            name_index: attribute_name_index,
            length: attribute_length,
        };

        let attribute = match attribute_name.as_str() {
            "ConstantValue" => Attribute::ConstantValue(ConstantValueAttribute::read_one(
                reader,
                &attribute_context,
            )?),
            "Code" => Attribute::Code(CodeAttribute::read_one(reader, &attribute_context)?),
            "StackMapTable" => Attribute::StackMapTable(StackMapTableAttribute::read_all(
                reader,
                &attribute_context,
            )?),
            "Exceptions" => Attribute::Exceptions(ExceptionIndexAttribute::read_all(
                reader,
                &attribute_context,
            )?),
            "InnerClasses" => {
                Attribute::InnerClasses(InnerClassAttribute::read_all(reader, &attribute_context)?)
            }
            "EnclosingMethod" => Attribute::EnclosingMethod(EnclosingMethodAttribute::read_one(
                reader,
                &attribute_context,
            )?),
            "Synthetic" => Attribute::Synthetic(),
            "Signature" => {
                Attribute::Signature(SignatureAttribute::read_one(reader, &attribute_context)?)
            }
            "SourceFile" => {
                Attribute::SourceFile(SourceFileAttribute::read_one(reader, &attribute_context)?)
            }
            "SourceDebugExtension" => Attribute::SourceDebugExtension(
                SourceDebugExtensionAttribute::read_one(reader, &attribute_context)?,
            ),
            "LineNumberTable" => Attribute::LineNumberTable(LineNumberTableAttribute::read_all(
                reader,
                &attribute_context,
            )?),
            "LocalVariableTable" => Attribute::LocalVariableTable(
                LocalVariableTableAttribute::read_all(reader, &attribute_context)?,
            ),
            "LocalVariableTypeTable" => Attribute::LocalVariableTypeTable(
                LocalVariableTypeTableAttribute::read_all(reader, &attribute_context)?,
            ),
            "Deprecated" => Attribute::Deprecated(),
            "RuntimeVisibleAnnotations" => Attribute::RuntimeVisibleAnnotations(
                AnnotationAttribute::read_all(reader, &attribute_context)?,
            ),
            "RuntimeInvisibleAnnotations" => Attribute::RuntimeInvisibleAnnotations(
                AnnotationAttribute::read_all(reader, &attribute_context)?,
            ),
            "RuntimeVisibleParameterAnnotations" => Attribute::RuntimeVisibleParameterAnnotations(
                ParameterAnnotationAttribute::read_all(reader, &attribute_context)?,
            ),
            "RuntimeInvisibleParameterAnnotations" => {
                Attribute::RuntimeInvisibleParameterAnnotations(
                    ParameterAnnotationAttribute::read_all(reader, &attribute_context)?,
                )
            }
            "AnnotationDefault" => Attribute::AnnotationDefault(
                AnnotationDefaultAttribute::read_one(reader, &attribute_context)?,
            ),
            "BootstrapMethods" => Attribute::BootstrapMethods(BootstrapMethodAttribute::read_all(
                reader,
                &attribute_context,
            )?),
            _ => Attribute::Misc(MiscAttribute::read_one(reader, &attribute_context)?),
        };
        Ok(attribute)
    }
}

impl ReadAll<ConstantPoolContext<'_>> for Attribute {}
