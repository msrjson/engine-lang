// Continuum Abstract Syntax Tree (AST) Definition
// Architecture: CTM-SPEC-02.1 Standard AST

use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct SourceLocation {
    pub line: usize,
    pub column: usize,
}

impl SourceLocation {
    pub fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }
}

impl fmt::Display for SourceLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Module {
    pub name: String,
    pub imports: Vec<Import>,
    pub declarations: Vec<TopLevelDecl>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Import {
    pub path: Vec<String>,
    pub items: ImportItems,
    pub loc: SourceLocation,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ImportItems {
    All,
    Specific(Vec<String>),
    Single(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Directive {
    pub name: String,
    pub args: Vec<(Option<String>, Expr)>,
    pub loc: SourceLocation,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TopLevelDecl {
    TypeAlias(TypeAliasDecl),
    Entity(EntityDecl),
    Struct(StructDecl),
    Enum(EnumDecl),
    Trait(TraitDecl),
    Impl(ImplDecl),
    Action(ActionDecl),
    Boundary(BoundaryDecl),
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypeAliasDecl {
    pub name: String,
    pub type_params: Vec<String>,
    pub target_type: TypeRef,
    pub refinement: Option<Expr>,
    pub loc: SourceLocation,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EntityDecl {
    pub directives: Vec<Directive>,
    pub name: String,
    pub type_params: Vec<String>,
    pub fields: Vec<EntityField>,
    pub invariant: Option<Expr>,
    pub loc: SourceLocation,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EntityField {
    pub directives: Vec<Directive>,
    pub name: String,
    pub field_type: TypeRef,
    pub default_value: Option<Expr>,
    pub refinement: Option<Expr>,
    pub loc: SourceLocation,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StructDecl {
    pub directives: Vec<Directive>,
    pub name: String,
    pub type_params: Vec<String>,
    pub fields: Vec<StructField>,
    pub loc: SourceLocation,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StructField {
    pub name: String,
    pub field_type: TypeRef,
    pub loc: SourceLocation,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumDecl {
    pub directives: Vec<Directive>,
    pub name: String,
    pub type_params: Vec<String>,
    pub variants: Vec<EnumVariant>,
    pub loc: SourceLocation,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumVariant {
    pub name: String,
    pub payload: EnumPayload,
    pub loc: SourceLocation,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EnumPayload {
    Unit,
    Tuple(Vec<TypeRef>),
    Struct(Vec<StructField>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct TraitDecl {
    pub is_pub: bool,
    pub name: String,
    pub type_params: Vec<String>,
    pub items: Vec<TraitItem>,
    pub contract: Option<ContractBlock>,
    pub loc: SourceLocation,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TraitItem {
    Type(String),
    ActionSig(ActionSignature),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionSignature {
    pub name: String,
    pub type_params: Vec<String>,
    pub params: Vec<Param>,
    pub return_type: Option<TypeRef>,
    pub loc: SourceLocation,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ContractBlock {
    pub formulas: Vec<Expr>,
    pub loc: SourceLocation,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ImplDecl {
    pub type_params: Vec<String>,
    pub trait_name: Option<String>,
    pub target_type: TypeRef,
    pub items: Vec<ImplItem>,
    pub loc: SourceLocation,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ImplItem {
    Type(String, TypeRef),
    Action(ActionDecl),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionDecl {
    pub is_pub: bool,
    pub name: String,
    pub type_params: Vec<String>,
    pub params: Vec<Param>,
    pub return_type: Option<TypeRef>,
    pub body: Block,
    pub loc: SourceLocation,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub name: String,
    pub param_type: TypeRef,
    pub loc: SourceLocation,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BoundaryDecl {
    pub name: String,
    pub routes: Vec<BoundaryRoute>,
    pub loc: SourceLocation,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BoundaryRoute {
    pub directives: Vec<Directive>,
    pub method: String,
    pub path: String,
    pub target_action: String,
    pub loc: SourceLocation,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeRef {
    Named(String, Vec<TypeRef>),
    Vector(usize),
    Decimal(usize, usize),
    Reference(bool, Box<TypeRef>), // (is_mut, inner)
    Unit,
}

impl fmt::Display for TypeRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TypeRef::Named(name, args) => {
                if args.is_empty() {
                    write!(f, "{}", name)
                } else {
                    let formatted_args: Vec<String> = args.iter().map(|a| a.to_string()).collect();
                    write!(f, "{}<{}>", name, formatted_args.join(", "))
                }
            }
            TypeRef::Vector(dim) => write!(f, "Vector({})", dim),
            TypeRef::Decimal(p, s) => write!(f, "Decimal({}, {})", p, s),
            TypeRef::Reference(is_mut, inner) => {
                if *is_mut {
                    write!(f, "&mut {}", inner)
                } else {
                    write!(f, "&{}", inner)
                }
            }
            TypeRef::Unit => write!(f, "()"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub statements: Vec<Statement>,
    pub loc: SourceLocation,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Let {
        is_mut: bool,
        name: String,
        type_ref: Option<TypeRef>,
        init: Expr,
        loc: SourceLocation,
    },
    Atomic {
        arg: Option<Expr>,
        body: Block,
        loc: SourceLocation,
    },
    Assign {
        target: Expr,
        op: AssignOperator,
        value: Expr,
        loc: SourceLocation,
    },
    Return {
        value: Option<Expr>,
        loc: SourceLocation,
    },
    If {
        condition: Expr,
        then_branch: Block,
        else_branch: Option<Block>,
        loc: SourceLocation,
    },
    Match {
        target: Expr,
        arms: Vec<MatchArm>,
        loc: SourceLocation,
    },
    For {
        var_name: String,
        iterable: Expr,
        body: Block,
        loc: SourceLocation,
    },
    Expr(Expr),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssignOperator {
    Assign,
    AddAssign,
    SubAssign,
    MulAssign,
    DivAssign,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub body: Expr,
    pub loc: SourceLocation,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    Wildcard,
    Identifier(String),
    Variant(String, Vec<Pattern>),
    Struct(String, Vec<(String, Pattern)>),
    Literal(Literal),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Expr {
    pub kind: ExprKind,
    pub loc: SourceLocation,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExprKind {
    Literal(Literal),
    Identifier(String),
    Qualified(Vec<String>),
    Unary {
        op: UnaryOperator,
        expr: Box<Expr>,
    },
    Binary {
        left: Box<Expr>,
        op: BinaryOperator,
        right: Box<Expr>,
    },
    Call {
        callee: Box<Expr>,
        args: Vec<Expr>,
    },
    MethodCall {
        receiver: Box<Expr>,
        method: String,
        args: Vec<Expr>,
    },
    FieldAccess {
        expr: Box<Expr>,
        field: String,
    },
    SafeFieldAccess {
        expr: Box<Expr>,
        field: String,
    },
    Index {
        target: Box<Expr>,
        index: Box<Expr>,
    },
    StructInit {
        name: String,
        fields: Vec<(String, Expr)>,
    },
    VectorLiteral(Vec<Expr>),
    ListLiteral(Vec<Expr>),
    Try(Box<Expr>),
    AtomicTry(Box<Expr>),
    Closure {
        params: Vec<String>,
        body: Box<Expr>,
    },
    Block(Block),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Integer(i64),
    Float(f64),
    Decimal(String),
    String(String),
    Boolean(bool),
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOperator {
    Neg,
    Not,
    BitNot,
    Ref,
    RefMut,
    Deref,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOperator {
    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Exp,

    // Vector SIMD
    VecAdd,
    VecSub,
    VecMul,
    CosineMetric,

    // Shift
    Shl,
    Shr,

    // Relational
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,

    // Bitwise
    BitAnd,
    BitOr,
    BitXor,

    // Logical
    LogAnd,
    LogOr,

    // Causal & Functional
    Pipeline,
    Backpipe,
    AsyncPipe,
    CausalEdge,
}

