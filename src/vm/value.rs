//! JVM value model: `JValue` plus type descriptor helpers.

/// A runtime value in a register, field, or array slot.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum JValue {
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    /// Reference to an arena object (see `object.rs`). Never null.
    Obj(u32),
    Null,
}

impl JValue {
    pub fn is_null(&self) -> bool {
        matches!(self, JValue::Null)
    }

    /// True for a reference that is null in DEX semantics: either `Null` or an
    /// `Int(0)` produced by a `const 0` in an object-typed register.
    pub fn is_null_ref(&self) -> bool {
        matches!(self, JValue::Null | JValue::Int(0))
    }

    /// Wide values (long/double) occupy a register pair in DEX; the VM stores
    /// them in a single slot with the second slot unused.
    pub fn is_wide(&self) -> bool {
        matches!(self, JValue::Long(_) | JValue::Double(_))
    }

    pub fn truthy(&self) -> bool {
        matches!(self, JValue::Int(v) if *v != 0)
    }

    /// Zero test used by if-eqz/if-nez on int/long/float/double regs.
    pub fn is_zero(&self) -> bool {
        match self {
            JValue::Int(v) => *v == 0,
            JValue::Long(v) => *v == 0,
            JValue::Float(v) => *v == 0.0,
            JValue::Double(v) => *v == 0.0,
            JValue::Null => true,
            JValue::Obj(_) => false,
        }
    }

    pub fn as_int(&self) -> i32 {
        match self {
            JValue::Int(v) => *v,
            JValue::Null => 0,
            _ => panic!("expected int, got {self:?}"),
        }
    }

    pub fn as_long(&self) -> i64 {
        match self {
            JValue::Long(v) => *v,
            _ => panic!("expected long, got {self:?}"),
        }
    }

    pub fn as_obj(&self) -> u32 {
        match self {
            JValue::Obj(o) => *o,
            JValue::Null => panic!("null object"),
            _ => panic!("expected object, got {self:?}"),
        }
    }

    /// Reference slot as an arena id, with every non-reference reported as
    /// absent. Use this where the caller already handles a missing object
    /// (host lookups, payload reads) instead of demanding a reference.
    pub fn as_obj_opt(&self) -> Option<u32> {
        match self {
            JValue::Obj(o) => Some(*o),
            _ => None,
        }
    }

    /// Reference slot as an arena id, reporting *why* it is not one so the
    /// caller can raise the JVM-correct throwable instead of panicking.
    pub fn as_obj_checked(&self) -> Result<u32, ValueTypeMismatch> {
        match self {
            JValue::Obj(o) => Ok(*o),
            _ if self.is_null_ref() => Err(ValueTypeMismatch::NullRef),
            other => Err(ValueTypeMismatch::WrongType(other.ty_tag())),
        }
    }

    /// `as_int` without the panic: `Null` keeps the existing zero convention.
    pub fn as_int_checked(&self) -> Result<i32, ValueTypeMismatch> {
        match self {
            JValue::Int(v) => Ok(*v),
            JValue::Null => Ok(0),
            other => Err(ValueTypeMismatch::WrongType(other.ty_tag())),
        }
    }

    /// `as_long` without the panic. An `Int` is sign-extended: a caller that
    /// reached a `J` parameter through a widened register is a calling
    /// convention mistake, not a type error worth aborting a parse for.
    pub fn as_long_checked(&self) -> Result<i64, ValueTypeMismatch> {
        match self {
            JValue::Long(v) => Ok(*v),
            JValue::Int(v) => Ok(i64::from(*v)),
            JValue::Null => Ok(0),
            other => Err(ValueTypeMismatch::WrongType(other.ty_tag())),
        }
    }

    pub fn ty_tag(&self) -> &'static str {
        match self {
            JValue::Int(_) => "int",
            JValue::Long(_) => "long",
            JValue::Float(_) => "float",
            JValue::Double(_) => "double",
            JValue::Obj(_) => "object",
            JValue::Null => "null",
        }
    }
}

/// Why a value could not be read as the kind a native expected. The distinction
/// matters because the JVM reaction differs: a DEX null reference is a
/// `NullPointerException`, a genuinely wrong kind is a `ClassCastException`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueTypeMismatch {
    /// A null reference in DEX semantics: `Null`, or the `Int(0)` that a
    /// `const 0` leaves in an object-typed register.
    NullRef,
    /// A value of another kind (`int`, `long`, `float`, `double`) where a
    /// reference was required, carrying the kind actually seen.
    WrongType(&'static str),
}

/// Parse a type descriptor into its kind.
#[derive(Debug, Clone, PartialEq)]
pub enum TypeKind {
    Prim(char),
    Class(String), // without surrounding L...;
    Array(String), // element descriptor
}

/// Parse a descriptor like `Lcom/foo/Bar;`, `[I`, `[[Ljava/lang/String;`, `I`.
pub fn parse_desc(desc: &str) -> TypeKind {
    if let Some(rest) = desc.strip_prefix('L') {
        TypeKind::Class(rest.trim_end_matches(';').to_string())
    } else if let Some(rest) = desc.strip_prefix('[') {
        TypeKind::Array(rest.to_string())
    } else {
        TypeKind::Prim(desc.chars().next().unwrap_or('V'))
    }
}

pub fn is_primitive_desc(desc: &str) -> bool {
    matches!(desc, "B" | "C" | "D" | "F" | "I" | "J" | "S" | "Z" | "V")
}

pub fn is_wide_desc(desc: &str) -> bool {
    matches!(desc, "J" | "D")
}

/// The default (zero/null) value for a descriptor.
pub fn default_of(desc: &str) -> JValue {
    match desc {
        "J" => JValue::Long(0),
        "D" => JValue::Double(0.0),
        "F" => JValue::Float(0.0),
        "Z" | "B" | "C" | "S" | "I" | "V" => JValue::Int(0),
        _ => JValue::Null,
    }
}

/// Dotted class name from a descriptor: `Lcom/foo/Bar;` -> `com.foo.Bar`.
pub fn dotted_name(desc: &str) -> String {
    match parse_desc(desc) {
        TypeKind::Class(c) => c.replace('/', "."),
        TypeKind::Array(e) => format!("{}[]", dotted_name(&e)),
        TypeKind::Prim(p) => match p {
            'B' => "byte".into(),
            'C' => "char".into(),
            'D' => "double".into(),
            'F' => "float".into(),
            'I' => "int".into(),
            'J' => "long".into(),
            'S' => "short".into(),
            'Z' => "boolean".into(),
            'V' => "void".into(),
            _ => p.to_string(),
        },
    }
}
