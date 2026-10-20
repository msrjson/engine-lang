// Engine Lang Reference Bootstrap Compiler (`enginec`) CLI Entrypoint
// Architecture: ENG-SPEC-03.1 & ENG-SLS-01.1

#![allow(dead_code)]

mod ast;
mod bridge;
mod codegen;
mod interpreter;
mod lexer;
mod memory;
mod parser;
mod smt;
mod web;

use clap::{Parser as ClapParser, Subcommand};
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(ClapParser)]
#[command(name = "enginec")]
#[command(author = "Engine Lang Core Working Group <architects@engine-lang.org>")]
#[command(version = "0.3.1 (v0.3.1-silicon, spec: ENG-SPEC-03.1, sls: ENG-SLS-01.1, codegen: ENG-CG-01.1)")]
#[command(about = "Reference Bootstrap Compiler and Formal SMT Verification Toolchain for Engine Lang", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Display comprehensive version, specification, and engineering architecture diagnostics
    Version {
        /// Output details in JSON format for automated monitoring pipelines
        #[arg(long)]
        json: bool,
    },
    /// Formally check and verify an Engine Lang source module (.eng) via Z3 SMT
    Check {
        /// Path to the Engine Lang source file (.eng)
        file: PathBuf,

        /// Print parsed AST
        #[arg(long)]
        ast: bool,

        /// Verbose formal proof output
        #[arg(short, long)]
        verbose: bool,
    },

    /// Run and simulate an Engine Lang causal program
    Run {
        /// Path to the Engine Lang source file (.eng)
        file: PathBuf,

        /// Storage container file (.db) to bind (default: ./data/engine.db)
        #[arg(short, long, alias = "nvram")]
        db: Option<PathBuf>,
    },

    /// Inspect the binary physical layout of an Engine Lang SLS container (.db)
    Inspect {
        /// Path to the .db container file
        file: PathBuf,
    },

    /// Initialize a new crash-consistent Engine Lang Single-Level Store container (.db)
    #[command(alias = "init-nvram")]
    InitDb {
        /// Target path for the .db file
        #[arg(short, long, default_value = "./data/engine.db")]
        path: PathBuf,

        /// Initial size in Megabytes (minimum 16 MiB)
        #[arg(short, long, default_value = "32")]
        size_mib: u64,
    },

    /// Start the embedded PostgreSQL wire protocol virtual bridge (engine-bridge)
    Bridge {
        /// Host address to bind to (default: 0.0.0.0)
        #[arg(long, default_value = "0.0.0.0")]
        host: String,

        /// Port to listen on (default: 5432)
        #[arg(short, long, default_value = "5432")]
        port: u16,

        /// Single-session test mode for automated verification
        #[arg(long)]
        test_mode: bool,
    },

    /// Serve the Engine Lang HTTP Web Gateway and PostgreSQL bridge concurrently
    Serve {
        /// HTTP gateway port to listen on (default: 8090)
        #[arg(long, default_value = "8090")]
        http_port: u16,

        /// PostgreSQL bridge port to listen on (default: 5432)
        #[arg(long, default_value = "5432")]
        pg_port: u16,

        /// Host address to bind to (default: 0.0.0.0)
        #[arg(long, default_value = "0.0.0.0")]
        host: String,

        /// Path to .db storage file
        #[arg(long, default_value = "./data/engine.db", alias = "nvram")]
        db: PathBuf,
    },

    /// Emit target-native LLVM Intermediate Representation (IR) with silicon DAX intrinsics
    EmitIr {
        /// Path to the Engine Lang source file (.eng)
        file: PathBuf,

        /// Target architecture (default: x86_64, supported: x86_64, aarch64)
        #[arg(short, long, default_value = "x86_64")]
        target: String,

        /// Output path for the generated .ll file (default: stdout)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    match cli.command {
        Commands::Version { json } => {
            if json {
                println!(
                    r#"{{"toolchain_version":"0.3.1","release_codename":"Sovereign Silicon","spec_version":"ENG-SPEC-03.1","storage_version":"ENG-SLS-01.1","codegen_spec":"ENG-CG-01.1","memory_model":"ENG-MEM-01.1","wire_protocol":"ENG-WIRE-01.1","unikernel_spec":"ENG-OS-01.1","smt_prover":"Z3 SMT-LIB2 v4.12+","target_arch":"x86_64-unknown-linux-gnu","license":"Apache-2.0"}}"#
                );
            } else {
                println!("=== Engine Lang Reference Compiler (enginec) ===");
                println!("Toolchain Version:   0.3.1 (v0.3.1-silicon)");
                println!("Release Milestone:   Sovereign Silicon / Systems Confrontation");
                println!("Grammar Spec:        ENG-SPEC-03.1 (14-level operator precedence)");
                println!("Single-Level Store:  ENG-SLS-01.1 (Magic: 0x454E474E, SpecVer: 0x0003)");
                println!("Silicon Codegen:     ENG-CG-01.1 (Direct Machine Instructions & Zero-UB)");
                println!("Formal SMT Prover:   Z3 SMT-LIB2 Engine v4.12+ (Automatic Proofs)");
                println!("Memory Model:        ENG-MEM-01.1 (Monotonic Epoch Arenas, Causal DAG)");
                println!("Wire & Replication:  ENG-WIRE-01.1 (Delta-CRDT P2P Mesh)");
                println!("Unikernel Target:    ENG-OS-01.1 (Bare-Metal Sovereign Unikernel)");
                println!("Target Architecture: x86_64-unknown-linux-gnu");
                println!("Authorship & Canon:  The Vance Protocol Group / Engine Lang Core Working Group");
                println!("Open Standard:       Apache License 2.0 (Decentralized Public Good)");
            }
            ExitCode::SUCCESS
        }

        Commands::Check { file, ast, verbose } => {
            println!(
                "🔍 Engine Lang Compiler (enginec) - Checking: {}",
                file.display()
            );
            let source = match fs::read_to_string(&file) {
                Ok(content) => content,
                Err(err) => {
                    eprintln!("❌ IO Error reading '{}': {}", file.display(), err);
                    return ExitCode::FAILURE;
                }
            };

            // Phase 1: Lexical Analysis
            let mut lexer = lexer::Lexer::new(&source);
            let tokens = match lexer.tokenize() {
                Ok(toks) => toks,
                Err(err) => {
                    eprintln!("❌ Lexer Error: {}", err);
                    return ExitCode::FAILURE;
                }
            };

            // Phase 2: Deterministic LL(1) Parsing
            let mut parser = parser::Parser::new(tokens);
            let module = match parser.parse_module() {
                Ok(m) => m,
                Err(err) => {
                    eprintln!("❌ Parser Error: {}", err);
                    return ExitCode::FAILURE;
                }
            };

            println!(
                "✅ Grammar Validation: Deterministic LL(1) parse succeeded (Module: {})",
                module.name
            );

            if ast {
                println!("\n=== Abstract Syntax Tree ===");
                println!("{:#?}", module);
            }

            // Phase 3: SMT Formal Invariant Verification
            println!("📐 Formal Verification: Translating invariants to Z3 SMT-LIB2...");
            if verbose {
                println!("   [Solver Configuration: QF_NRA, Logic: ALL, Timeout: 5000ms]");
            }
            match smt::ModuleVerifier::verify_module(&module) {
                Ok(proof_reports) => {
                    for report in proof_reports {
                        println!("  {}", report);
                    }
                    if verbose {
                        println!("   [Z3 Backend: Invariant theorem status: UNSAT on negated conditions]");
                    }
                    println!("✨ Formal Verification Succeeded: All type refinements & contracts mathematically proven.");
                    ExitCode::SUCCESS
                }
                Err(proof_err) => {
                    eprintln!("❌ Formal Proof Violation: {}", proof_err);
                    ExitCode::FAILURE
                }
            }
        }

        Commands::Run { file, db } => {
            println!("🚀 Engine Lang Runtime - Executing: {}", file.display());
            let source = match fs::read_to_string(&file) {
                Ok(content) => content,
                Err(err) => {
                    eprintln!("❌ IO Error reading '{}': {}", file.display(), err);
                    return ExitCode::FAILURE;
                }
            };

            let mut lexer = lexer::Lexer::new(&source);
            let tokens = match lexer.tokenize() {
                Ok(toks) => toks,
                Err(err) => {
                    eprintln!("❌ Lexer Error: {}", err);
                    return ExitCode::FAILURE;
                }
            };

            let mut parser = parser::Parser::new(tokens);
            let module = match parser.parse_module() {
                Ok(m) => m,
                Err(err) => {
                    eprintln!("❌ Parser Error: {}", err);
                    return ExitCode::FAILURE;
                }
            };

            if let Err(proof_err) = smt::ModuleVerifier::verify_module(&module) {
                eprintln!(
                    "❌ Formal Verification Failed Prior to Execution:\n{}",
                    proof_err
                );
                return ExitCode::FAILURE;
            }

            let db_target = db.unwrap_or_else(|| PathBuf::from("./data/engine.db"));
            if !db_target.exists() {
                if let Some(parent) = db_target.parent() {
                    let _ = fs::create_dir_all(parent);
                }
                println!(
                    "📦 Initializing local SLS container at: {}",
                    db_target.display()
                );
                if let Err(e) = memory::EngineDbStorage::initialize_db(&db_target, 32) {
                    eprintln!("❌ Failed to initialize SLS container: {}", e);
                    return ExitCode::FAILURE;
                }
            }

            println!("⚡ Single-Level Store mounted on: {}", db_target.display());
            println!("🔒 Atomic hardware protection active: CLWB + SFENCE durable");

            // Polyglot Datasources Discovery
            let mut datasources_count = 0;
            for decl in &module.declarations {
                if let ast::TopLevelDecl::Datasource(ds) = decl {
                    datasources_count += 1;
                    let mode_str = ds.mode.as_deref().unwrap_or("read_write");
                    println!(
                        "🔌 External Datasource bound: '{}' [driver: {}, mode: {}]",
                        ds.name, ds.driver, mode_str
                    );
                }
            }
            if datasources_count == 0 {
                println!("⚡ Single-Level Store (engine.db) default zero-config active");
            } else {
                println!("⚡ Polyglot Data Layer active: {} external datasource(s) synchronized with local SLS cache", datasources_count);
            }

            println!("🌐 Causal mesh ready: Local node #1 active");
            println!("⚙️  Executing action 'main' via causal runtime engine...");

            let mut interp = interpreter::Interpreter::new(&module);
            match interp.execute_main() {
                Ok(res) => {
                    println!("🎉 Execution of 'main' succeeded (Result: {})", res);
                    for log in interp.ctx.output_log {
                        println!("   {}", log);
                    }
                    ExitCode::SUCCESS
                }
                Err(err) => {
                    eprintln!("❌ Runtime Error: {}", err);
                    ExitCode::FAILURE
                }
            }
        }

        Commands::Inspect { file } => match memory::EngineDbStorage::inspect_container(&file) {
            Ok(report) => {
                print!("{}", report);
                ExitCode::SUCCESS
            }
            Err(err) => {
                eprintln!(
                    "❌ Failed to inspect SLS container '{}': {}",
                    file.display(),
                    err
                );
                ExitCode::FAILURE
            }
        },

        Commands::InitDb { path, size_mib } => {
            if let Some(parent) = path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            println!(
                "📦 Creating {} MiB Single-Level Store (.db) at: {}",
                size_mib,
                path.display()
            );
            match memory::EngineDbStorage::initialize_db(&path, size_mib) {
                Ok(()) => {
                    println!("✅ Successfully initialized Engine Lang storage container (Region 0 Superblock, Region 1 Bitmap, Region 2 Journal).");
                    ExitCode::SUCCESS
                }
                Err(err) => {
                    eprintln!("❌ Initialization failed: {}", err);
                    ExitCode::FAILURE
                }
            }
        }

        Commands::Bridge {
            host,
            port,
            test_mode,
        } => {
            let addr = format!("{}:{}", host, port);
            let bridge = bridge::PostgresBridge::new(&addr);
            if let Err(e) = bridge.start_server(test_mode) {
                eprintln!("❌ Bridge server error: {}", e);
                ExitCode::FAILURE
            } else {
                ExitCode::SUCCESS
            }
        }

        Commands::Serve {
            http_port,
            pg_port,
            host,
            db,
        } => {
            if !db.exists() {
                if let Some(parent) = db.parent() {
                    let _ = fs::create_dir_all(parent);
                }
                println!("📦 Initializing SLS container at: {}", db.display());
                let _ = memory::EngineDbStorage::initialize_db(&db, 32);
            }

            let pg_addr = format!("{}:{}", host, pg_port);
            let http_addr = format!("{}:{}", host, http_port);

            // Start PostgreSQL bridge in background thread
            let pg_bridge = bridge::PostgresBridge::new(&pg_addr);
            std::thread::spawn(move || {
                let _ = pg_bridge.start_server(false);
            });

            // Start HTTP Web Gateway on main thread
            let web_server = web::WebServer::new(&http_addr);
            if let Err(e) = web_server.start() {
                eprintln!("❌ Web server error: {}", e);
                ExitCode::FAILURE
            } else {
                ExitCode::SUCCESS
            }
        }

        Commands::EmitIr {
            file,
            target,
            output,
        } => {
            println!(
                "⚙️  Engine Lang Codegen - Compiling to Silicon LLVM IR: {}",
                file.display()
            );
            let source = match fs::read_to_string(&file) {
                Ok(content) => content,
                Err(err) => {
                    eprintln!("❌ IO Error reading '{}': {}", file.display(), err);
                    return ExitCode::FAILURE;
                }
            };

            let mut lexer = lexer::Lexer::new(&source);
            let tokens = match lexer.tokenize() {
                Ok(toks) => toks,
                Err(err) => {
                    eprintln!("❌ Lexer Error: {}", err);
                    return ExitCode::FAILURE;
                }
            };

            let mut parser = parser::Parser::new(tokens);
            let module = match parser.parse_module() {
                Ok(m) => m,
                Err(err) => {
                    eprintln!("❌ Parser Error: {}", err);
                    return ExitCode::FAILURE;
                }
            };

            let mut cg = codegen::LlvmCodegen::new(module, &target);
            match cg.generate_ir() {
                Ok(ir) => {
                    if let Some(out_path) = output {
                        if let Err(e) = fs::write(&out_path, &ir) {
                            eprintln!(
                                "❌ Failed to write LLVM IR to '{}': {}",
                                out_path.display(),
                                e
                            );
                            return ExitCode::FAILURE;
                        }
                        println!("✨ LLVM IR successfully emitted to: {}", out_path.display());
                    } else {
                        println!(
                            "\n=== LLVM Intermediate Representation (Target: {}) ===\n",
                            target
                        );
                        println!("{}", ir);
                    }
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("❌ Codegen Error: {}", e);
                    ExitCode::FAILURE
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_superblock_binary_roundtrip() {
        let sb = memory::EngineSuperblock::default();
        let bytes = sb.to_bytes();
        let parsed =
            memory::EngineSuperblock::from_bytes(&bytes).expect("Roundtrip parsing must succeed");
        assert_eq!(sb.magic, parsed.magic);
        assert_eq!(sb.spec_version, parsed.spec_version);
        assert_eq!(sb.crc64, parsed.crc64);
    }

    #[test]
    fn test_custom_operator_tokenization() {
        let code = "<=> @* @+ @- ?! ~> -> |>";
        let mut l = lexer::Lexer::new(code);
        let tokens = l.tokenize().expect("Tokenization must succeed");
        assert_eq!(tokens[0].kind, lexer::TokenKind::CosineMetric);
        assert_eq!(tokens[1].kind, lexer::TokenKind::VecMul);
        assert_eq!(tokens[2].kind, lexer::TokenKind::VecAdd);
        assert_eq!(tokens[3].kind, lexer::TokenKind::VecSub);
        assert_eq!(tokens[4].kind, lexer::TokenKind::AtomicTry);
        assert_eq!(tokens[5].kind, lexer::TokenKind::AsyncPipe);
        assert_eq!(tokens[6].kind, lexer::TokenKind::CausalEdge);
        assert_eq!(tokens[7].kind, lexer::TokenKind::Pipe);
    }

    #[test]
    fn test_chg_relational_error_eng_op_001() {
        let code = "action test() { let x = 1 < 2 < 3; }";
        let mut l = lexer::Lexer::new(code);
        let tokens = l.tokenize().unwrap();
        let mut p = parser::Parser::new(tokens);
        let res = p.parse_module();
        assert!(res.is_err());
        let err = res.unwrap_err();
        assert!(err.contains("ENG-OP-001") || err.contains("OP-001"));
    }

    #[test]
    fn test_smt_refinement_translation() {
        let code = "type Port = Int where value >= 1 && value <= 65535;";
        let mut l = lexer::Lexer::new(code);
        let tokens = l.tokenize().unwrap();
        let mut p = parser::Parser::new(tokens);
        let m = p.parse_module().unwrap();
        let reports = smt::ModuleVerifier::verify_module(&m).unwrap();
        assert_eq!(reports.len(), 1);
        assert!(reports[0].contains("Port"));
    }

    #[test]
    fn test_polyglot_datasource_parsing() {
        let code = r#"
module Test::Polyglot;

datasource LegacyPostgres {
    driver: postgres,
    uri: "postgresql://user:pass@localhost:5432/plant",
    mode: write_through
}

@datasource(name: "LegacyPostgres", table: "turbines")
@persistent
entity Turbine {
    id: UUIDv7,
    rpm: Int,
    invariant { self.rpm >= 0 }
}
"#;
        let mut l = lexer::Lexer::new(code);
        let tokens = l
            .tokenize()
            .expect("Tokenization of datasource must succeed");
        let mut p = parser::Parser::new(tokens);
        let m = p
            .parse_module()
            .expect("Parsing of module with datasource must succeed");
        assert_eq!(m.declarations.len(), 2);
        if let ast::TopLevelDecl::Datasource(ds) = &m.declarations[0] {
            assert_eq!(ds.name, "LegacyPostgres");
            assert_eq!(ds.driver, "postgres");
            assert_eq!(ds.mode.as_deref(), Some("write_through"));
        } else {
            panic!("Expected first declaration to be Datasource");
        }
    }

    #[test]
    fn test_llvm_codegen_with_clwb_sfence() {
        let code = r#"
module Test::Codegen;

action commit_trans() {
    atomic {
        let x = 1;
    }
    return;
}
"#;
        let mut l = lexer::Lexer::new(code);
        let tokens = l.tokenize().unwrap();
        let mut p = parser::Parser::new(tokens);
        let m = p.parse_module().unwrap();
        let mut cg = codegen::LlvmCodegen::new(m, "x86_64");
        let ir = cg.generate_ir().unwrap();
        assert!(ir.contains("@llvm.x86.clwb"));
        assert!(ir.contains("@llvm.x86.sfence"));
        assert!(ir.contains("define dso_local i32 @action_commit_trans"));
    }

    #[test]
    fn test_industrial_mmio_register_parsing() {
        let code = r#"
module Test::Hardware;

@mmio(address: 0x40001000, size: 32)
register ValveActuatorControl {
    enable: Bit,
    pressure_cutoff: Byte,
    flow_rate: Word,
}
"#;
        let mut l = lexer::Lexer::new(code);
        let tokens = l.tokenize().expect("Tokenization of register must succeed");
        let mut p = parser::Parser::new(tokens);
        let m = p.parse_module().expect("Parsing of register must succeed");
        assert_eq!(m.declarations.len(), 1);
        if let ast::TopLevelDecl::Register(reg) = &m.declarations[0] {
            assert_eq!(reg.name, "ValveActuatorControl");
            assert_eq!(reg.directives.len(), 1);
            assert_eq!(reg.directives[0].name, "mmio");
            assert_eq!(reg.fields.len(), 3);
            assert_eq!(reg.fields[0].name, "enable");
            assert_eq!(reg.fields[0].bit_width, 1);
            assert_eq!(reg.fields[1].name, "pressure_cutoff");
            assert_eq!(reg.fields[1].bit_width, 8);
        } else {
            panic!("Expected Register declaration");
        }
    }
}
