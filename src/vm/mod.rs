use crate::class::Class;

// ============================================================================
// ERRORS
// ============================================================================

enum VirtualMachineError {}

// ============================================================================
// OBJECTS
// ============================================================================

// Object references ----------------------------------------------------------

enum ReferenceValue {
    Null,
    Object(Class),
    Array(Box<ReferenceValue>),
    Interface(Class),
}

enum PrimitiveValue {
    Boolean(bool),
    Byte(i8),
    Short(i16),
    Char(u16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    ReturnAddress(usize),
}

enum Value {
    Reference(ReferenceValue),
    Primitive(PrimitiveValue),
}

// ============================================================================
// THREADS
// ============================================================================

// Stack frame ----------------------------------------------------------------

struct StackFrame<'c> {
    class: &'c Class,
    local_variables: Vec<Value>,
    operand_stack: Vec<Value>,
}

impl<'c> StackFrame<'c> {
    fn new(class: &'c Class) -> StackFrame<'c> {
        StackFrame {
            class,
            local_variables: Vec::new(),
            operand_stack: Vec::new(),
        }
    }
}

struct Thread<'c> {
    pc: usize,
    stack: Vec<StackFrame<'c>>,
}

impl<'c> Thread<'c> {
    fn new(class: &'c Class) -> Thread<'c> {
        let stack_frame = StackFrame::new(&class);

        Thread {
            pc: 0,
            stack: vec![stack_frame],
        }
    }
}

// ============================================================================
// VIRTUAL MACHINE
// ============================================================================

struct VM<'c> {
    threads: Vec<Thread<'c>>,
    heap: Vec<Value>,
    method_area: Vec<Class>,
    main_class: &'c Class,
}

impl<'c> VM<'c> {
    pub fn new(main_class: &'c Class) -> VM<'c> {
        let main_thread = Thread::new(main_class);

        VM {
            threads: vec![main_thread],
            heap: Vec::new(),
            method_area: Vec::new(),
            main_class,
        }
    }
}
