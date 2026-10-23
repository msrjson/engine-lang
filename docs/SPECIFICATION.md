# Engine Lang Specification
**Document ID**: `ENG-SPEC-03.1`
**Status**: Official Normative Specification
**Target Architectures**: x86-64 (Intel Optane, CXL 2.0/3.0), ARM64 (FEAT_DPB / FEAT_DPB2)
**Encoding**: UTF-8

---

## 1. Overview and Core Semantics

Engine Lang is an infrastructure-agnostic, statically typed systems programming language featuring a unified Single-Level Store (SLS) on non-volatile physical memory (NVRAM/CXL). Engine Lang replaces external database layers, caches, message brokers, and ORM serializations by treating persistent memory as a first-class citizen of the type system.

### 1.1. Deterministic Execution Model
- **No Volatile/Persistent Impedance Mismatch**: Memory allocation inside `@persistent entity` declarations maps to persistent virtual memory addresses backed by direct access (DAX).
- **Unit Return on Mutation**: Assignment operations (`=`, `+=`, `-=`, etc.) return the unit type `()`, preventing unintentional assignment expressions.
- **Atomic Confinement**: Any mutation of persistent entities must occur inside an `atomic { ... }` block, compiling directly to hardware cache line writebacks (`clwb`) and memory fences (`sfence`).

### 1.2. Dual Storage Paradigm & Quantum Era Preparedness
Engine Lang provides developers with explicit control over persistent memory representation:
- **Transparent SLS (`@persistent(storage: "engine.db")`)**: Default zero-overhead direct graph representation aligned to 64 bytes. Optimal for rapid prototyping, sub-microsecond local analytics, and embedded IoT edge nodes.
- **Blinded UUIDv7 Quantum-Resistant Graph (`@persistent(storage: "uuid7_blinded", cipher: "pqc_kyber1024")`)**: Memory layout and persistent extensions resolve entities strictly through 128-bit RFC 9562 `UUIDv7` tokens. Graph edges and node topologies are cryptographically blinded with NIST FIPS 203/204 Post-Quantum Lattice algorithms (`ML-KEM-1024` and `ML-DSA-87`), preventing quantum side-channel profiling and memory snooping.

### 1.3. Level-5 Human Ergonomics vs Deterministic Agent Machine Rigor
- **Level-5 Human Simplicity**: Zero database setup, zero connection strings, zero migrations; clean and declarative syntax accessible to novice programmers, allowing "Hello World" in 3 lines while maintaining expressive power.
- **AI Agent Precision**: LL(1) deterministic grammar without backtracking, automated Z3 SMT formal proof verification of all contracts and invariants, and hardware compilation directly to LLVM IR, bare-metal UEFI unikernels, or quantum-safe distributed networks.

---

## 2. Complete Formal Grammar (EBNF)

```ebnf
Module            ::= "module" QualifiedIdentifier ";" ImportList TopLevelDeclList ;

QualifiedIdentifier ::= Identifier ( "::" Identifier )* ;

ImportList        ::= ( "use" QualifiedIdentifier ( "::" ( "{" IdentifierList "}" | "*" | Identifier ) )? ";" )* ;

IdentifierList    ::= Identifier ( "," Identifier )* ;

TopLevelDeclList  ::= ( Directive* TopLevelDecl )* ;

TopLevelDecl      ::= DatasourceDecl
                    | RegisterDecl
                    | TypeAliasDecl
                    | EntityDecl
                    | EnumDecl
                    | TraitDecl
                    | ImplDecl
                    | ActionDecl
                    | BoundaryDecl ;

DatasourceDecl    ::= "datasource" Identifier "{" DatasourcePropertyList "}" ;
DatasourcePropertyList ::= ( Identifier ":" Expression ","? )* ;

RegisterDecl      ::= Directive* "register" Identifier "{" RegisterFieldList "}" ;
RegisterFieldList ::= ( Identifier ":" RegisterType ","? )* ;
RegisterType      ::= "Bit" | "Byte" | "Word" | "DWord" | "QWord" ;

Directive         ::= "@" Identifier ( "(" DirectiveArgs? ")" )? ;

DirectiveArgs     ::= DirectiveArg ( "," DirectiveArg )* ;
DirectiveArg      ::= ( Identifier ":" )? Expression ;

TypeAliasDecl     ::= "type" Identifier ( "<" TypeParamList ">" )? "=" TypeRef ( "where" RefinementExpr )? ";" ;

EntityDecl        ::= "entity" Identifier ( "<" TypeParamList ">" )? "{" EntityFieldList InvariantBlock? "}" ;

EntityFieldList   ::= ( Directive* Identifier ":" TypeRef ( "=" Expression )? ( "where" RefinementExpr )? ","? )* ;

InvariantBlock    ::= "invariant" "{" Expression "}" ;

EnumDecl          ::= "enum" Identifier ( "<" TypeParamList ">" )? "{" EnumVariantList "}" ;

EnumVariantList   ::= ( Identifier ( "(" TypeRefList ")" | "{" StructFieldList "}" )? ","? )* ;

TraitDecl         ::= "pub"? "trait" Identifier ( "<" TypeParamList ">" )? "{" TraitItemList "}" ;

TraitItemList     ::= ( TypeAliasDecl | ActionSignature | ContractBlock )* ;

ContractBlock     ::= "contract" "{" ContractExprList "}" ;

ImplDecl          ::= "impl" ( "<" TypeParamList ">" )? Identifier "for" TypeRef "{" ImplItemList "}" ;

ActionDecl        ::= "pub"? "action" Identifier ( "<" TypeParamList ">" )? "(" ParamList? ")" ( "->" TypeRef )? Block ;

ActionSignature   ::= "fn" Identifier ( "<" TypeParamList ">" )? "(" ParamList? ")" ( "->" TypeRef )? ";" ;

BoundaryDecl      ::= "boundary" Identifier "{" RouteDeclList "}" ;

RouteDeclList     ::= ( Directive* "route" RouteMethod StringLiteral "->" Identifier ";" )* ;

RouteMethod       ::= "GET" | "POST" | "PUT" | "DELETE" | "QUERY" ;

TypeRef           ::= QualifiedIdentifier ( "<" TypeRefList ">" )?
                    | "Vector" "(" IntegerLiteral ")"
                    | "Decimal" "(" IntegerLiteral "," IntegerLiteral ")"
                    | "&" "mut"? TypeRef
                    | "(" TypeRefList? ")" ;

TypeRefList       ::= TypeRef ( "," TypeRef )* ;

ParamList         ::= Param ( "," Param )* ;
Param             ::= Identifier ":" TypeRef ;

RefinementExpr    ::= Expression ;

Block             ::= "{" Statement* "}" ;

Statement         ::= LetStmt
                    | AtomicStmt
                    | AssignStmt
                    | ReturnStmt
                    | IfStmt
                    | MatchStmt
                    | ExprStmt ;

LetStmt           ::= "let" "mut"? Identifier ( ":" TypeRef )? "=" Expression ";" ;

AtomicStmt        ::= "atomic" ( "(" Expression ")" )? Block ;

AssignStmt        ::= TargetExpr AssignOp Expression ";" ;

AssignOp          ::= "=" | "+=" | "-=" | "*=" | "/=" ;

ReturnStmt        ::= "return" Expression? ";" ;

IfStmt            ::= "if" Expression Block ( "else" ( IfStmt | Block ) )? ;

MatchStmt         ::= "match" Expression "{" MatchArmList "}" ;

MatchArmList      ::= ( Pattern "=>" ( Expression ";" | Block ) )* ;

Pattern           ::= Identifier ( "(" PatternList? ")" )?
                    | Literal
                    | "_" ;

PatternList       ::= Pattern ( "," Pattern )* ;

ExprStmt          ::= Expression ";" ;

Expression        ::= PipelineExpr ;

PipelineExpr      ::= CausalExpr ( ( "|>" | "~>" | "->" ) CausalExpr )* ;

CausalExpr        ::= LogicalOrExpr ;

LogicalOrExpr     ::= LogicalAndExpr ( "||" LogicalAndExpr )* ;

LogicalAndExpr    ::= BitwiseOrExpr ( "&&" BitwiseOrExpr )* ;

BitwiseOrExpr     ::= BitwiseXorExpr ( "|" BitwiseXorExpr )* ;

BitwiseXorExpr    ::= BitwiseAndExpr ( "^" BitwiseAndExpr )* ;

BitwiseAndExpr    ::= RelationalExpr ( "&" RelationalExpr )* ;

RelationalExpr    ::= CosineMetricExpr ( ( "==" | "!=" | "<" | "<=" | ">" | ">=" ) CosineMetricExpr )? ;

CosineMetricExpr  ::= ShiftExpr ( "<=>" ShiftExpr )* ;

ShiftExpr         ::= AdditiveExpr ( ( "<<" | ">>" ) AdditiveExpr )* ;

AdditiveExpr      ::= MultiplicativeExpr ( ( "+" | "-" | "@+" | "@-" ) MultiplicativeExpr )* ;

MultiplicativeExpr ::= ExponentExpr ( ( "*" | "/" | "%" | "@*" ) ExponentExpr )* ;

ExponentExpr      ::= UnaryExpr ( "**" UnaryExpr )* ;

UnaryExpr         ::= ( "-" | "!" | "~" | "&" | "*" ) UnaryExpr
                    | PostfixExpr ;

PostfixExpr       ::= PrimaryExpr ( "?" | "?!" | "." Identifier | "?." Identifier | "[" Expression "]" | "(" ArgList? ")" )* ;

PrimaryExpr       ::= QualifiedIdentifier
                    | Literal
                    | "(" Expression ")"
                    | VectorLiteral
                    | StructConstructor ;

VectorLiteral     ::= "[" ( Expression ( "," Expression )* )? "]" ;

StructConstructor ::= QualifiedIdentifier "{" ( Identifier ":" Expression ( "," Identifier ":" Expression )* )? "}" ;

ArgList           ::= Expression ( "," Expression )* ;

Literal           ::= IntegerLiteral | FloatLiteral | DecimalLiteral | StringLiteral | BooleanLiteral | "None" ;
```

---

## 3. Semantic Directives Reference

Engine Lang uses structured compiler directives (`@name(args)`) to control storage layout, security enforcement, and lazy schema migration.

### 3.1. Persistence: `@persistent`
- **Target**: `entity` declarations.
- **Semantics**: Allocates instances in Region 3 of the `.db` Single-Level Store. All fields are crash-consistent. Modifying an instance requires an `atomic` block.

```engine
@persistent
entity Customer {
    id: UUIDv7,
    name: Text,
    credit: Decimal(18, 4)
}
```

### 3.2. Data Residency: `@locality`
- **Target**: `entity` or module declarations.
- **Semantics**: Constrains which physical mesh nodes are allowed to replicate or persist instances of this entity, ensuring GDPR, LGPD, and sovereign boundary compliance.

```engine
@persistent
@locality(region: "sa-east-1", allowed_nodes: ["sp-node-01", "sp-node-02"])
entity BrazilianTaxRecord {
    cpf: Text,
    aliquot: Decimal(5, 2)
}
```

### 3.3. Multi-Tenant Cryptographic Isolation: `@TenantGuard`
- **Target**: `entity` or `action` declarations.
- **Semantics**: Enforces cryptographically isolated tenant keys in the `EntityHeader` (Offset `0x10`). Queries without a valid tenant token automatically fail compilation or abort at runtime with an access trap.

```engine
@persistent
@TenantGuard(strict: true)
entity TenantInvoice {
    invoice_num: Text,
    total: Decimal(18, 4)
}
```

---

## 4. Refinement Types and SMT Verification

Refinement types attach mathematical predicates to primitive or user-defined types using the `where` keyword. The integrated Z3 SMT solver validates these contracts at compile-time.

```engine
type Port = Int where value >= 1 && value <= 65535;
type NormalizedScore = Float where value >= 0.0 && value <= 1.0;
type NonEmptyText = Text where len(value) > 0;
```

When an instance of a refined type is constructed or mutated, the compiler generates a verification condition (VC). If the VC cannot be proven unconditionally via SMT, compilation halts with a detailed diagnostic and a counterexample model:

```
[SMT-VERIFY-ERR] Invariant violation in module 'Billing::Engine'
  Location: src/billing.eng:42:15
  Failed Refinement: 'value >= 0.0000'
  Counterexample Model:
    value = -0.0001
  Path Condition:
    invoice.discount = 120.0000
    base_price = 100.0000
```

---

## 5. Algebraic Data Types and Pattern Matching

Engine Lang provides first-class sum types (`enum`) and product types (`struct`/`entity`).

```engine
enum TransactionStatus {
    Pending,
    Committed(EpochTimestamp),
    RolledBack(ErrorCode),
    Suspended { reason: Text, retry_after_ms: Int }
}

action evaluate_status(status: TransactionStatus) -> Text {
    return match status {
        TransactionStatus::Pending => "Processing",
        TransactionStatus::Committed(epoch) => "Committed at epoch",
        TransactionStatus::RolledBack(code) => "Failed with code",
        TransactionStatus::Suspended { reason, retry_after_ms } => "Suspended"
    };
}
```

### 5.1. Pattern Matching Rules
- **Exhaustiveness**: Every `match` statement must cover all possible variants of the enumerated type.
- **Irrefutability**: Destructuring bindings in `let` statements must be irrefutable.

---

## 6. The `boundary` Foreign Interface

Engine Lang replaces external web server frameworks and reverse proxies with the built-in `boundary` block. A boundary exposes internal causal graph entities directly to external network clients (HTTP, gRPC, PostgreSQL Wire Protocol):

```engine
boundary PublicGateway {
    @locality(public: true)
    route GET "/api/v1/vehicles/:id" -> fetch_vehicle_by_id;
}
```

---

## 7. Polyglot Datasources & External Persistence

Engine Lang provides the embedded Single-Level Store (`engine.db`) as its zero-config default. However, developers maintain complete freedom to target existing corporate databases via declarative `datasource` definitions:

```engine
datasource WarehouseDb {
    driver: postgres,  // postgres | mysql | sqlite | clickhouse
    uri: env("DATABASE_URL"),
    mode: write_through
}

@datasource(WarehouseDb, table: "plant_telemetry")
@persistent
entity SensorNode {
    id: UUIDv7,
    reading: Float where value >= 0.0,
    invariant { self.reading <= 500.0 }
}
```

---

## 8. Industrial Hardware MMIO & Silicon Codegen

For embedded microcontrollers, PLCs, and bare-metal environments, Engine Lang provides memory-mapped register declarations:

```engine
@mmio(address: 0x40001000, size: 32)
register ActuatorCtrl {
    enable: Bit,
    trip_level: Byte,
    flow_quota: Word,
}
```

During ahead-of-time compilation via `enginec emit-ir` and `enginec emit-asm`, the compiler emits target-native instructions with direct cache-line writeback intrinsics (`clwb` + `sfence` on x86_64, `dc cvac` + `dmb ishst` on ARM64) ensuring crash-proof durability on bare silicon.

---

## 9. Planetary Causal Mesh & Bare-Metal Sovereign OS

### 9.1. Planetary Causal Mesh Protocol (`engine-wire`)
Distributed multi-node deployments synchronize via Delta-CRDT vector clocks without central coordinators (`enginec mesh`). State updates propagate causally, resolving concurrent mutations through monotonic vector clocks.

### 9.2. Sovereign Bare-Metal Unikernel (`engine-os`)
The compiler synthesizes a freestanding UEFI 2.10 PE32+ image (`engine-boot.efi`) via `enginec boot-image`:
- Boots directly from bare silicon firmware in **$< 15\text{ ms}$**.
- Maps physical NVRAM / CXL into a flat 64-bit Single-Level Store (`0x0020_0000`).
- Eliminates the traditional operating system kernel, page cache, and file system layers.

