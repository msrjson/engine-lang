// Continuum Recursive-Descent Parser Implementation
// Architecture: CTM-SPEC-02.1 LL(1) Deterministic Parser

use crate::ast::*;
use crate::lexer::{Token, TokenKind};

pub struct Parser {
    tokens: Vec<Token>,
    cursor: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, cursor: 0 }
    }

    fn current(&self) -> &Token {
        if self.cursor < self.tokens.len() {
            &self.tokens[self.cursor]
        } else {
            self.tokens.last().expect("Token stream must have at least EOF")
        }
    }

    fn peek(&self) -> &Token {
        if self.cursor + 1 < self.tokens.len() {
            &self.tokens[self.cursor + 1]
        } else {
            self.tokens.last().expect("Token stream must have at least EOF")
        }
    }

    fn advance(&mut self) -> Token {
        let tok = self.current().clone();
        if self.cursor < self.tokens.len() - 1 {
            self.cursor += 1;
        }
        tok
    }

    fn check(&self, kind: &TokenKind) -> bool {
        &self.current().kind == kind
    }

    fn match_token(&mut self, kind: &TokenKind) -> bool {
        if self.check(kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn expect(&mut self, kind: &TokenKind) -> Result<Token, String> {
        if self.check(kind) {
            Ok(self.advance())
        } else {
            let curr = self.current();
            Err(format!(
                "Parse error at {}: expected {:?}, found {:?}",
                curr.loc, kind, curr.kind
            ))
        }
    }

    fn expect_identifier(&mut self) -> Result<(String, SourceLocation), String> {
        let curr = self.current().clone();
        if let TokenKind::Identifier(name) = curr.kind {
            self.advance();
            Ok((name, curr.loc))
        } else {
            Err(format!(
                "Parse error at {}: expected identifier, found {:?}",
                curr.loc, curr.kind
            ))
        }
    }

    // =========================================================================
    // Module and Top-Level Declarations
    // =========================================================================

    pub fn parse_module(&mut self) -> Result<Module, String> {
        let mut module_name = String::from("Main");

        if self.match_token(&TokenKind::Module) {
            let (name, _) = self.parse_qualified_identifier()?;
            module_name = name;
            self.expect(&TokenKind::Semicolon)?;
        }

        let mut imports = Vec::new();
        while self.check(&TokenKind::Use) {
            imports.push(self.parse_import()?);
        }

        let mut declarations = Vec::new();
        while !self.check(&TokenKind::Eof) {
            declarations.push(self.parse_top_level_decl()?);
        }

        Ok(Module {
            name: module_name,
            imports,
            declarations,
        })
    }

    fn parse_import(&mut self) -> Result<Import, String> {
        let start_tok = self.expect(&TokenKind::Use)?;
        let mut path = Vec::new();
        let (first, _) = self.expect_identifier()?;
        path.push(first);

        while self.match_token(&TokenKind::DoubleColon) {
            if self.match_token(&TokenKind::Star) {
                self.expect(&TokenKind::Semicolon)?;
                return Ok(Import {
                    path,
                    items: ImportItems::All,
                    loc: start_tok.loc,
                });
            } else if self.match_token(&TokenKind::LBrace) {
                let mut items = Vec::new();
                while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::Eof) {
                    let (item, _) = self.expect_identifier()?;
                    items.push(item);
                    if !self.match_token(&TokenKind::Comma) {
                        break;
                    }
                }
                self.expect(&TokenKind::RBrace)?;
                self.expect(&TokenKind::Semicolon)?;
                return Ok(Import {
                    path,
                    items: ImportItems::Specific(items),
                    loc: start_tok.loc,
                });
            } else {
                let (segment, _) = self.expect_identifier()?;
                path.push(segment);
            }
        }

        self.expect(&TokenKind::Semicolon)?;
        let last_item = path.pop().unwrap();
        Ok(Import {
            path,
            items: ImportItems::Single(last_item),
            loc: start_tok.loc,
        })
    }

    fn parse_directives(&mut self) -> Result<Vec<Directive>, String> {
        let mut directives = Vec::new();
        while self.match_token(&TokenKind::At) {
            let (name, loc) = self.expect_identifier()?;
            let mut args = Vec::new();

            if self.match_token(&TokenKind::LParen) {
                while !self.check(&TokenKind::RParen) && !self.check(&TokenKind::Eof) {
                    let mut arg_name = None;
                    if let TokenKind::Identifier(key) = &self.current().kind {
                        if self.peek().kind == TokenKind::Colon {
                            arg_name = Some(key.clone());
                            self.advance(); // identifier
                            self.advance(); // colon
                        }
                    }
                    let expr = self.parse_expression()?;
                    args.push((arg_name, expr));
                    if !self.match_token(&TokenKind::Comma) {
                        break;
                    }
                }
                self.expect(&TokenKind::RParen)?;
            }

            directives.push(Directive { name, args, loc });
        }
        Ok(directives)
    }

    fn parse_top_level_decl(&mut self) -> Result<TopLevelDecl, String> {
        let directives = self.parse_directives()?;

        if self.check(&TokenKind::Type) {
            return self.parse_type_alias_decl().map(TopLevelDecl::TypeAlias);
        }
        if self.check(&TokenKind::Entity) {
            return self.parse_entity_decl(directives).map(TopLevelDecl::Entity);
        }
        if self.check(&TokenKind::Struct) {
            return self.parse_struct_decl(directives).map(TopLevelDecl::Struct);
        }
        if self.check(&TokenKind::Enum) {
            return self.parse_enum_decl(directives).map(TopLevelDecl::Enum);
        }
        if self.check(&TokenKind::Trait) || (self.check(&TokenKind::Pub) && self.peek().kind == TokenKind::Trait) {
            return self.parse_trait_decl().map(TopLevelDecl::Trait);
        }
        if self.check(&TokenKind::Impl) {
            return self.parse_impl_decl().map(TopLevelDecl::Impl);
        }
        if self.check(&TokenKind::Action) || (self.check(&TokenKind::Pub) && self.peek().kind == TokenKind::Action) {
            return self.parse_action_decl().map(TopLevelDecl::Action);
        }
        if self.check(&TokenKind::Boundary) {
            return self.parse_boundary_decl().map(TopLevelDecl::Boundary);
        }

        Err(format!(
            "Parse error at {}: unexpected top-level token {:?}",
            self.current().loc,
            self.current().kind
        ))
    }

    fn parse_type_alias_decl(&mut self) -> Result<TypeAliasDecl, String> {
        let start = self.expect(&TokenKind::Type)?;
        let (name, _) = self.expect_identifier()?;
        let type_params = self.parse_optional_type_params()?;

        self.expect(&TokenKind::Assign)?;
        let target_type = self.parse_type_ref()?;

        let mut refinement = None;
        if self.match_token(&TokenKind::Where) {
            refinement = Some(self.parse_expression()?);
        }

        self.expect(&TokenKind::Semicolon)?;

        Ok(TypeAliasDecl {
            name,
            type_params,
            target_type,
            refinement,
            loc: start.loc,
        })
    }

    fn parse_entity_decl(&mut self, directives: Vec<Directive>) -> Result<EntityDecl, String> {
        let start = self.expect(&TokenKind::Entity)?;
        let (name, _) = self.expect_identifier()?;
        let type_params = self.parse_optional_type_params()?;

        self.expect(&TokenKind::LBrace)?;

        let mut fields = Vec::new();
        let mut invariant = None;

        while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::Eof) {
            if self.match_token(&TokenKind::Invariant) {
                self.expect(&TokenKind::LBrace)?;
                invariant = Some(self.parse_expression()?);
                self.expect(&TokenKind::RBrace)?;
            } else {
                let field_directives = self.parse_directives()?;
                let (field_name, field_loc) = self.expect_identifier()?;
                self.expect(&TokenKind::Colon)?;
                let field_type = self.parse_type_ref()?;

                let mut default_value = None;
                if self.match_token(&TokenKind::Assign) {
                    default_value = Some(self.parse_expression()?);
                }

                let mut refinement = None;
                if self.match_token(&TokenKind::Where) {
                    refinement = Some(self.parse_expression()?);
                }

                self.match_token(&TokenKind::Comma);

                fields.push(EntityField {
                    directives: field_directives,
                    name: field_name,
                    field_type,
                    default_value,
                    refinement,
                    loc: field_loc,
                });
            }
        }

        self.expect(&TokenKind::RBrace)?;

        Ok(EntityDecl {
            directives,
            name,
            type_params,
            fields,
            invariant,
            loc: start.loc,
        })
    }

    fn parse_struct_decl(&mut self, directives: Vec<Directive>) -> Result<StructDecl, String> {
        let start = self.expect(&TokenKind::Struct)?;
        let (name, _) = self.expect_identifier()?;
        let type_params = self.parse_optional_type_params()?;

        self.expect(&TokenKind::LBrace)?;
        let mut fields = Vec::new();

        while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::Eof) {
            let (field_name, field_loc) = self.expect_identifier()?;
            self.expect(&TokenKind::Colon)?;
            let field_type = self.parse_type_ref()?;
            self.match_token(&TokenKind::Comma);

            fields.push(StructField {
                name: field_name,
                field_type,
                loc: field_loc,
            });
        }

        self.expect(&TokenKind::RBrace)?;

        Ok(StructDecl {
            directives,
            name,
            type_params,
            fields,
            loc: start.loc,
        })
    }

    fn parse_enum_decl(&mut self, directives: Vec<Directive>) -> Result<EnumDecl, String> {
        let start = self.expect(&TokenKind::Enum)?;
        let (name, _) = self.expect_identifier()?;
        let type_params = self.parse_optional_type_params()?;

        self.expect(&TokenKind::LBrace)?;
        let mut variants = Vec::new();

        while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::Eof) {
            let (v_name, v_loc) = self.expect_identifier()?;
            let mut payload = EnumPayload::Unit;

            if self.match_token(&TokenKind::LParen) {
                let mut tuple_types = Vec::new();
                while !self.check(&TokenKind::RParen) && !self.check(&TokenKind::Eof) {
                    tuple_types.push(self.parse_type_ref()?);
                    if !self.match_token(&TokenKind::Comma) {
                        break;
                    }
                }
                self.expect(&TokenKind::RParen)?;
                payload = EnumPayload::Tuple(tuple_types);
            } else if self.match_token(&TokenKind::LBrace) {
                let mut struct_fields = Vec::new();
                while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::Eof) {
                    let (fname, floc) = self.expect_identifier()?;
                    self.expect(&TokenKind::Colon)?;
                    let ftype = self.parse_type_ref()?;
                    self.match_token(&TokenKind::Comma);
                    struct_fields.push(StructField {
                        name: fname,
                        field_type: ftype,
                        loc: floc,
                    });
                }
                self.expect(&TokenKind::RBrace)?;
                payload = EnumPayload::Struct(struct_fields);
            }

            self.match_token(&TokenKind::Comma);
            variants.push(EnumVariant {
                name: v_name,
                payload,
                loc: v_loc,
            });
        }

        self.expect(&TokenKind::RBrace)?;

        Ok(EnumDecl {
            directives,
            name,
            type_params,
            variants,
            loc: start.loc,
        })
    }

    fn parse_trait_decl(&mut self) -> Result<TraitDecl, String> {
        let is_pub = self.match_token(&TokenKind::Pub);
        let start = self.expect(&TokenKind::Trait)?;
        let (name, _) = self.expect_identifier()?;
        let type_params = self.parse_optional_type_params()?;

        self.expect(&TokenKind::LBrace)?;
        let mut items = Vec::new();
        let mut contract = None;

        while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::Eof) {
            if self.match_token(&TokenKind::Type) {
                let (tname, _) = self.expect_identifier()?;
                self.expect(&TokenKind::Semicolon)?;
                items.push(TraitItem::Type(tname));
            } else if self.match_token(&TokenKind::Fn) || self.match_token(&TokenKind::Action) {
                let (aname, aloc) = self.expect_identifier()?;
                let a_type_params = self.parse_optional_type_params()?;
                self.expect(&TokenKind::LParen)?;
                let params = self.parse_param_list()?;
                self.expect(&TokenKind::RParen)?;

                let mut return_type = None;
                if self.match_token(&TokenKind::CausalEdge) {
                    return_type = Some(self.parse_type_ref()?);
                }
                self.expect(&TokenKind::Semicolon)?;

                items.push(TraitItem::ActionSig(ActionSignature {
                    name: aname,
                    type_params: a_type_params,
                    params,
                    return_type,
                    loc: aloc,
                }));
            } else if self.match_token(&TokenKind::Contract) {
                let c_start = self.current().loc.clone();
                self.expect(&TokenKind::LBrace)?;
                let mut formulas = Vec::new();
                while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::Eof) {
                    formulas.push(self.parse_expression()?);
                    self.match_token(&TokenKind::Semicolon);
                }
                self.expect(&TokenKind::RBrace)?;
                contract = Some(ContractBlock {
                    formulas,
                    loc: c_start,
                });
            } else {
                return Err(format!(
                    "Unexpected item in trait at {}: {:?}",
                    self.current().loc,
                    self.current().kind
                ));
            }
        }

        self.expect(&TokenKind::RBrace)?;

        Ok(TraitDecl {
            is_pub,
            name,
            type_params,
            items,
            contract,
            loc: start.loc,
        })
    }

    fn parse_impl_decl(&mut self) -> Result<ImplDecl, String> {
        let start = self.expect(&TokenKind::Impl)?;
        let type_params = self.parse_optional_type_params()?;

        let mut trait_name = None;
        let first_type = self.parse_type_ref()?;

        let target_type = if self.match_token(&TokenKind::For) {
            trait_name = Some(first_type.to_string());
            self.parse_type_ref()?
        } else {
            first_type
        };

        self.expect(&TokenKind::LBrace)?;
        let mut items = Vec::new();

        while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::Eof) {
            if self.match_token(&TokenKind::Type) {
                let (tname, _) = self.expect_identifier()?;
                self.expect(&TokenKind::Assign)?;
                let tval = self.parse_type_ref()?;
                self.expect(&TokenKind::Semicolon)?;
                items.push(ImplItem::Type(tname, tval));
            } else if self.check(&TokenKind::Fn) || self.check(&TokenKind::Action) {
                let action = self.parse_action_decl()?;
                items.push(ImplItem::Action(action));
            } else {
                return Err(format!(
                    "Unexpected token in impl block at {}: {:?}",
                    self.current().loc,
                    self.current().kind
                ));
            }
        }

        self.expect(&TokenKind::RBrace)?;

        Ok(ImplDecl {
            type_params,
            trait_name,
            target_type,
            items,
            loc: start.loc,
        })
    }

    fn parse_action_decl(&mut self) -> Result<ActionDecl, String> {
        let is_pub = self.match_token(&TokenKind::Pub);
        let start = if self.match_token(&TokenKind::Action) {
            self.tokens[self.cursor - 1].clone()
        } else {
            self.expect(&TokenKind::Fn)?
        };

        let (name, _) = self.expect_identifier()?;
        let type_params = self.parse_optional_type_params()?;

        self.expect(&TokenKind::LParen)?;
        let params = self.parse_param_list()?;
        self.expect(&TokenKind::RParen)?;

        let mut return_type = None;
        if self.match_token(&TokenKind::CausalEdge) {
            return_type = Some(self.parse_type_ref()?);
        }

        let body = self.parse_block()?;

        Ok(ActionDecl {
            is_pub,
            name,
            type_params,
            params,
            return_type,
            body,
            loc: start.loc,
        })
    }

    fn parse_param_list(&mut self) -> Result<Vec<Param>, String> {
        let mut params = Vec::new();
        while !self.check(&TokenKind::RParen) && !self.check(&TokenKind::Eof) {
            let (name, loc) = self.expect_identifier()?;
            self.expect(&TokenKind::Colon)?;
            let param_type = self.parse_type_ref()?;
            params.push(Param {
                name,
                param_type,
                loc,
            });
            if !self.match_token(&TokenKind::Comma) {
                break;
            }
        }
        Ok(params)
    }

    fn parse_boundary_decl(&mut self) -> Result<BoundaryDecl, String> {
        let start = self.expect(&TokenKind::Boundary)?;
        let (name, _) = self.expect_identifier()?;
        self.expect(&TokenKind::LBrace)?;

        let mut routes = Vec::new();
        while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::Eof) {
            let directives = self.parse_directives()?;
            self.expect(&TokenKind::Route)?;

            let (method, _) = self.expect_identifier()?;
            let path = if let TokenKind::StringLiteral(s) = &self.current().kind {
                let p = s.clone();
                self.advance();
                p
            } else {
                return Err(format!("Expected path string literal at {}", self.current().loc));
            };

            self.expect(&TokenKind::CausalEdge)?;
            let (target_action, rloc) = self.expect_identifier()?;
            self.expect(&TokenKind::Semicolon)?;

            routes.push(BoundaryRoute {
                directives,
                method,
                path,
                target_action,
                loc: rloc,
            });
        }

        self.expect(&TokenKind::RBrace)?;

        Ok(BoundaryDecl {
            name,
            routes,
            loc: start.loc,
        })
    }

    // =========================================================================
    // Types
    // =========================================================================

    fn parse_type_ref(&mut self) -> Result<TypeRef, String> {
        if self.match_token(&TokenKind::Ampersand) {
            let is_mut = self.match_token(&TokenKind::Mut);
            let inner = self.parse_type_ref()?;
            return Ok(TypeRef::Reference(is_mut, Box::new(inner)));
        }

        if self.match_token(&TokenKind::LParen) {
            if self.match_token(&TokenKind::RParen) {
                return Ok(TypeRef::Unit);
            }
            let inner = self.parse_type_ref()?;
            self.expect(&TokenKind::RParen)?;
            return Ok(inner);
        }

        let (name, _) = self.parse_qualified_identifier()?;

        if name == "Vector" && self.match_token(&TokenKind::LParen) {
            let dim = if let TokenKind::Integer(n) = self.current().kind {
                self.advance();
                n as usize
            } else {
                return Err(format!("Expected integer dimension for Vector at {}", self.current().loc));
            };
            self.expect(&TokenKind::RParen)?;
            return Ok(TypeRef::Vector(dim));
        }

        if name == "Decimal" && self.match_token(&TokenKind::LParen) {
            let p = if let TokenKind::Integer(n) = self.current().kind {
                self.advance();
                n as usize
            } else {
                return Err(format!("Expected integer precision for Decimal at {}", self.current().loc));
            };
            self.expect(&TokenKind::Comma)?;
            let s = if let TokenKind::Integer(n) = self.current().kind {
                self.advance();
                n as usize
            } else {
                return Err(format!("Expected integer scale for Decimal at {}", self.current().loc));
            };
            self.expect(&TokenKind::RParen)?;
            return Ok(TypeRef::Decimal(p, s));
        }

        let mut type_args = Vec::new();
        if self.match_token(&TokenKind::Less) {
            while !self.check(&TokenKind::Greater) && !self.check(&TokenKind::Eof) {
                type_args.push(self.parse_type_ref()?);
                if !self.match_token(&TokenKind::Comma) {
                    break;
                }
            }
            self.expect(&TokenKind::Greater)?;
        }

        Ok(TypeRef::Named(name, type_args))
    }

    fn parse_optional_type_params(&mut self) -> Result<Vec<String>, String> {
        let mut params = Vec::new();
        if self.match_token(&TokenKind::Less) {
            while !self.check(&TokenKind::Greater) && !self.check(&TokenKind::Eof) {
                let (param, _) = self.expect_identifier()?;
                params.push(param);
                if !self.match_token(&TokenKind::Comma) {
                    break;
                }
            }
            self.expect(&TokenKind::Greater)?;
        }
        Ok(params)
    }

    fn parse_qualified_identifier(&mut self) -> Result<(String, SourceLocation), String> {
        let (first, loc) = self.expect_identifier()?;
        let mut full = first;

        while self.match_token(&TokenKind::DoubleColon) {
            let (next, _) = self.expect_identifier()?;
            full.push_str("::");
            full.push_str(&next);
        }

        Ok((full, loc))
    }

    // =========================================================================
    // Statements & Blocks
    // =========================================================================

    fn parse_block(&mut self) -> Result<Block, String> {
        let start = self.expect(&TokenKind::LBrace)?;
        let mut statements = Vec::new();

        while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::Eof) {
            statements.push(self.parse_statement()?);
        }

        self.expect(&TokenKind::RBrace)?;

        Ok(Block {
            statements,
            loc: start.loc,
        })
    }

    fn parse_statement(&mut self) -> Result<Statement, String> {
        if self.check(&TokenKind::Let) {
            return self.parse_let_statement();
        }
        if self.check(&TokenKind::Atomic) {
            return self.parse_atomic_statement();
        }
        if self.check(&TokenKind::Return) {
            return self.parse_return_statement();
        }
        if self.check(&TokenKind::If) {
            return self.parse_if_statement();
        }
        if self.check(&TokenKind::Match) {
            return self.parse_match_statement();
        }
        if self.check(&TokenKind::For) {
            return self.parse_for_statement();
        }

        // Assignment or Expr statement
        let expr = self.parse_expression()?;

        // Check for assignment operator
        let assign_op = match self.current().kind {
            TokenKind::Assign => Some(AssignOperator::Assign),
            TokenKind::AddAssign => Some(AssignOperator::AddAssign),
            TokenKind::SubAssign => Some(AssignOperator::SubAssign),
            TokenKind::MulAssign => Some(AssignOperator::MulAssign),
            TokenKind::DivAssign => Some(AssignOperator::DivAssign),
            _ => None,
        };

        if let Some(op) = assign_op {
            let op_loc = self.advance().loc;
            let val = self.parse_expression()?;
            self.expect(&TokenKind::Semicolon)?;
            return Ok(Statement::Assign {
                target: expr,
                op,
                value: val,
                loc: op_loc,
            });
        }

        self.expect(&TokenKind::Semicolon)?;
        Ok(Statement::Expr(expr))
    }

    fn parse_let_statement(&mut self) -> Result<Statement, String> {
        let start = self.expect(&TokenKind::Let)?;
        let is_mut = self.match_token(&TokenKind::Mut);
        let (name, _) = self.expect_identifier()?;

        let mut type_ref = None;
        if self.match_token(&TokenKind::Colon) {
            type_ref = Some(self.parse_type_ref()?);
        }

        self.expect(&TokenKind::Assign)?;
        let init = self.parse_expression()?;
        self.expect(&TokenKind::Semicolon)?;

        Ok(Statement::Let {
            is_mut,
            name,
            type_ref,
            init,
            loc: start.loc,
        })
    }

    fn parse_atomic_statement(&mut self) -> Result<Statement, String> {
        let start = self.expect(&TokenKind::Atomic)?;
        let mut arg = None;
        if self.match_token(&TokenKind::LParen) {
            arg = Some(self.parse_expression()?);
            self.expect(&TokenKind::RParen)?;
        }
        let body = self.parse_block()?;

        Ok(Statement::Atomic {
            arg,
            body,
            loc: start.loc,
        })
    }

    fn parse_return_statement(&mut self) -> Result<Statement, String> {
        let start = self.expect(&TokenKind::Return)?;
        let mut val = None;
        if !self.check(&TokenKind::Semicolon) {
            val = Some(self.parse_expression()?);
        }
        self.expect(&TokenKind::Semicolon)?;

        Ok(Statement::Return {
            value: val,
            loc: start.loc,
        })
    }

    fn parse_if_statement(&mut self) -> Result<Statement, String> {
        let start = self.expect(&TokenKind::If)?;
        let condition = self.parse_expression()?;
        let then_branch = self.parse_block()?;

        let mut else_branch = None;
        if self.match_token(&TokenKind::Else) {
            if self.check(&TokenKind::If) {
                let nested_if = self.parse_if_statement()?;
                else_branch = Some(Block {
                    statements: vec![nested_if],
                    loc: start.loc.clone(),
                });
            } else {
                else_branch = Some(self.parse_block()?);
            }
        }

        Ok(Statement::If {
            condition,
            then_branch,
            else_branch,
            loc: start.loc,
        })
    }

    fn parse_match_statement(&mut self) -> Result<Statement, String> {
        let start = self.expect(&TokenKind::Match)?;
        let target = self.parse_expression()?;
        self.expect(&TokenKind::LBrace)?;

        let mut arms = Vec::new();
        while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::Eof) {
            let (pat, pat_loc) = self.parse_pattern()?;
            self.expect(&TokenKind::FatArrow)?;
            let body = self.parse_expression()?;
            self.match_token(&TokenKind::Comma);

            arms.push(MatchArm {
                pattern: pat,
                body,
                loc: pat_loc,
            });
        }

        self.expect(&TokenKind::RBrace)?;

        Ok(Statement::Match {
            target,
            arms,
            loc: start.loc,
        })
    }

    fn parse_for_statement(&mut self) -> Result<Statement, String> {
        let start = self.expect(&TokenKind::For)?;
        let (var_name, _) = self.expect_identifier()?;
        if let TokenKind::Identifier(ref s) = self.current().kind {
            if s == "in" {
                self.advance();
            } else {
                return Err(format!("Expected 'in' in for loop, found '{}' at {}", s, self.current().loc));
            }
        } else {
            return Err(format!("Expected 'in' in for loop at {}", self.current().loc));
        }
        let iterable = self.parse_expression()?;
        let body = self.parse_block()?;

        Ok(Statement::For {
            var_name,
            iterable,
            body,
            loc: start.loc,
        })
    }

    fn parse_pattern(&mut self) -> Result<(Pattern, SourceLocation), String> {
        let curr = self.current().clone();
        if self.match_token(&TokenKind::Identifier(String::from("_"))) {
            return Ok((Pattern::Wildcard, curr.loc));
        }

        if let TokenKind::Identifier(name) = &curr.kind {
            let mut qual_name = name.clone();
            let loc = curr.loc.clone();
            self.advance();

            while self.match_token(&TokenKind::DoubleColon) {
                let (next, _) = self.expect_identifier()?;
                qual_name.push_str("::");
                qual_name.push_str(&next);
            }

            if self.match_token(&TokenKind::LParen) {
                let mut sub_pats = Vec::new();
                while !self.check(&TokenKind::RParen) && !self.check(&TokenKind::Eof) {
                    let (p, _) = self.parse_pattern()?;
                    sub_pats.push(p);
                    if !self.match_token(&TokenKind::Comma) {
                        break;
                    }
                }
                self.expect(&TokenKind::RParen)?;
                return Ok((Pattern::Variant(qual_name, sub_pats), loc));
            } else if self.match_token(&TokenKind::LBrace) {
                let mut field_pats = Vec::new();
                while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::Eof) {
                    let (fname, _) = self.expect_identifier()?;
                    let p = if self.match_token(&TokenKind::Colon) {
                        let (sub_p, _) = self.parse_pattern()?;
                        sub_p
                    } else {
                        Pattern::Identifier(fname.clone())
                    };
                    field_pats.push((fname, p));
                    if !self.match_token(&TokenKind::Comma) {
                        break;
                    }
                }
                self.expect(&TokenKind::RBrace)?;
                return Ok((Pattern::Struct(qual_name, field_pats), loc));
            }

            return Ok((Pattern::Identifier(qual_name), loc));
        }

        Err(format!("Expected pattern at {}: found {:?}", curr.loc, curr.kind))
    }

    // =========================================================================
    // Expression Parsing - 14-Level Precedence Hierarchy
    // =========================================================================

    pub fn parse_expression(&mut self) -> Result<Expr, String> {
        self.parse_lambda_or_pipeline()
    }

    fn parse_lambda_or_pipeline(&mut self) -> Result<Expr, String> {
        // Check for lambda: `x => expr`
        if let TokenKind::Identifier(param) = &self.current().kind {
            if self.peek().kind == TokenKind::FatArrow {
                let loc = self.current().loc.clone();
                let param_name = param.clone();
                self.advance(); // identifier
                self.advance(); // =>
                let body = self.parse_expression()?;
                return Ok(Expr {
                    kind: ExprKind::Closure {
                        params: vec![param_name],
                        body: Box::new(body),
                    },
                    loc,
                });
            }
        }

        self.parse_pipeline_level_13()
    }

    // Level 13: Pipeline, Backpipe, AsyncPipe, CausalEdge
    fn parse_pipeline_level_13(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_logical_or_level_12()?;

        loop {
            let op = match self.current().kind {
                TokenKind::Pipe => BinaryOperator::Pipeline,
                TokenKind::Backpipe => BinaryOperator::Backpipe,
                TokenKind::AsyncPipe => BinaryOperator::AsyncPipe,
                TokenKind::CausalEdge => BinaryOperator::CausalEdge,
                _ => break,
            };

            let op_loc = self.advance().loc;
            let right = self.parse_logical_or_level_12()?;
            left = Expr {
                kind: ExprKind::Binary {
                    left: Box::new(left),
                    op,
                    right: Box::new(right),
                },
                loc: op_loc,
            };
        }

        Ok(left)
    }

    // Level 12: Logical OR (||)
    fn parse_logical_or_level_12(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_logical_and_level_12()?;

        while self.match_token(&TokenKind::LogOr) {
            let op_loc = self.tokens[self.cursor - 1].loc.clone();
            let right = self.parse_logical_and_level_12()?;
            left = Expr {
                kind: ExprKind::Binary {
                    left: Box::new(left),
                    op: BinaryOperator::LogOr,
                    right: Box::new(right),
                },
                loc: op_loc,
            };
        }

        Ok(left)
    }

    // Level 12: Logical AND (&&)
    fn parse_logical_and_level_12(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_bitwise_or_level_11()?;

        while self.match_token(&TokenKind::LogAnd) {
            let op_loc = self.tokens[self.cursor - 1].loc.clone();
            let right = self.parse_bitwise_or_level_11()?;
            left = Expr {
                kind: ExprKind::Binary {
                    left: Box::new(left),
                    op: BinaryOperator::LogAnd,
                    right: Box::new(right),
                },
                loc: op_loc,
            };
        }

        Ok(left)
    }

    // Level 11: Bitwise OR (|)
    fn parse_bitwise_or_level_11(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_bitwise_xor_level_11()?;

        while self.match_token(&TokenKind::PipeChar) {
            let op_loc = self.tokens[self.cursor - 1].loc.clone();
            let right = self.parse_bitwise_xor_level_11()?;
            left = Expr {
                kind: ExprKind::Binary {
                    left: Box::new(left),
                    op: BinaryOperator::BitOr,
                    right: Box::new(right),
                },
                loc: op_loc,
            };
        }

        Ok(left)
    }

    // Level 11: Bitwise XOR (^)
    fn parse_bitwise_xor_level_11(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_bitwise_and_level_11()?;

        while self.match_token(&TokenKind::Caret) {
            let op_loc = self.tokens[self.cursor - 1].loc.clone();
            let right = self.parse_bitwise_and_level_11()?;
            left = Expr {
                kind: ExprKind::Binary {
                    left: Box::new(left),
                    op: BinaryOperator::BitXor,
                    right: Box::new(right),
                },
                loc: op_loc,
            };
        }

        Ok(left)
    }

    // Level 11: Bitwise AND (&)
    fn parse_bitwise_and_level_11(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_relational_level_10()?;

        while self.match_token(&TokenKind::Ampersand) {
            let op_loc = self.tokens[self.cursor - 1].loc.clone();
            let right = self.parse_relational_level_10()?;
            left = Expr {
                kind: ExprKind::Binary {
                    left: Box::new(left),
                    op: BinaryOperator::BitAnd,
                    right: Box::new(right),
                },
                loc: op_loc,
            };
        }

        Ok(left)
    }

    // Level 10: Relational (==, !=, <, <=, >, >=) - Non-chaining!
    fn parse_relational_level_10(&mut self) -> Result<Expr, String> {
        let left = self.parse_cosine_metric_level_9()?;

        let op = match self.current().kind {
            TokenKind::Equal => Some(BinaryOperator::Equal),
            TokenKind::NotEqual => Some(BinaryOperator::NotEqual),
            TokenKind::Less => Some(BinaryOperator::Less),
            TokenKind::LessEqual => Some(BinaryOperator::LessEqual),
            TokenKind::Greater => Some(BinaryOperator::Greater),
            TokenKind::GreaterEqual => Some(BinaryOperator::GreaterEqual),
            _ => None,
        };

        if let Some(bin_op) = op {
            let op_loc = self.advance().loc;
            let right = self.parse_cosine_metric_level_9()?;

            // Reject invalid chaining: a < b < c (CTM-OP-001)
            if matches!(
                self.current().kind,
                TokenKind::Equal
                    | TokenKind::NotEqual
                    | TokenKind::Less
                    | TokenKind::LessEqual
                    | TokenKind::Greater
                    | TokenKind::GreaterEqual
            ) {
                return Err(format!(
                    "Syntax Error [CTM-OP-001] at {}: Chained relational comparisons are prohibited. Use explicit conjunction: a < b && b < c",
                    self.current().loc
                ));
            }

            return Ok(Expr {
                kind: ExprKind::Binary {
                    left: Box::new(left),
                    op: bin_op,
                    right: Box::new(right),
                },
                loc: op_loc,
            });
        }

        Ok(left)
    }

    // Level 9: Cosine Metric (<=>)
    fn parse_cosine_metric_level_9(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_shift_level_8()?;

        while self.match_token(&TokenKind::CosineMetric) {
            let op_loc = self.tokens[self.cursor - 1].loc.clone();
            let right = self.parse_shift_level_8()?;
            left = Expr {
                kind: ExprKind::Binary {
                    left: Box::new(left),
                    op: BinaryOperator::CosineMetric,
                    right: Box::new(right),
                },
                loc: op_loc,
            };
        }

        Ok(left)
    }

    // Level 8: Bit Shift (<<, >>)
    fn parse_shift_level_8(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_additive_level_7()?;

        loop {
            let op = match self.current().kind {
                TokenKind::Shl => BinaryOperator::Shl,
                TokenKind::Shr => BinaryOperator::Shr,
                _ => break,
            };

            let op_loc = self.advance().loc;
            let right = self.parse_additive_level_7()?;
            left = Expr {
                kind: ExprKind::Binary {
                    left: Box::new(left),
                    op,
                    right: Box::new(right),
                },
                loc: op_loc,
            };
        }

        Ok(left)
    }

    // Level 7: Additive (+, -, @+, @-)
    fn parse_additive_level_7(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_multiplicative_level_6()?;

        loop {
            let op = match self.current().kind {
                TokenKind::Plus => BinaryOperator::Add,
                TokenKind::Minus => BinaryOperator::Sub,
                TokenKind::VecAdd => BinaryOperator::VecAdd,
                TokenKind::VecSub => BinaryOperator::VecSub,
                _ => break,
            };

            let op_loc = self.advance().loc;
            let right = self.parse_multiplicative_level_6()?;
            left = Expr {
                kind: ExprKind::Binary {
                    left: Box::new(left),
                    op,
                    right: Box::new(right),
                },
                loc: op_loc,
            };
        }

        Ok(left)
    }

    // Level 6: Multiplicative (*, /, %, @*)
    fn parse_multiplicative_level_6(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_exponent_level_5()?;

        loop {
            let op = match self.current().kind {
                TokenKind::Star => BinaryOperator::Mul,
                TokenKind::Slash => BinaryOperator::Div,
                TokenKind::Percent => BinaryOperator::Mod,
                TokenKind::VecMul => BinaryOperator::VecMul,
                _ => break,
            };

            let op_loc = self.advance().loc;
            let right = self.parse_exponent_level_5()?;
            left = Expr {
                kind: ExprKind::Binary {
                    left: Box::new(left),
                    op,
                    right: Box::new(right),
                },
                loc: op_loc,
            };
        }

        Ok(left)
    }

    // Level 5: Exponentiation (**) Right-to-Left
    fn parse_exponent_level_5(&mut self) -> Result<Expr, String> {
        let left = self.parse_unary_level_4()?;

        if self.match_token(&TokenKind::Exp) {
            let op_loc = self.tokens[self.cursor - 1].loc.clone();
            let right = self.parse_exponent_level_5()?; // recursive for right-associativity
            return Ok(Expr {
                kind: ExprKind::Binary {
                    left: Box::new(left),
                    op: BinaryOperator::Exp,
                    right: Box::new(right),
                },
                loc: op_loc,
            });
        }

        Ok(left)
    }

    // Level 4: Unary (- , !, ~, &, * )
    fn parse_unary_level_4(&mut self) -> Result<Expr, String> {
        let curr = self.current().clone();

        let op = match curr.kind {
            TokenKind::Minus => Some(UnaryOperator::Neg),
            TokenKind::Bang => Some(UnaryOperator::Not),
            TokenKind::Tilde => Some(UnaryOperator::BitNot),
            TokenKind::Ampersand => {
                if self.peek().kind == TokenKind::Mut {
                    self.advance(); // &
                    self.advance(); // mut
                    let inner = self.parse_unary_level_4()?;
                    return Ok(Expr {
                        kind: ExprKind::Unary {
                            op: UnaryOperator::RefMut,
                            expr: Box::new(inner),
                        },
                        loc: curr.loc,
                    });
                } else {
                    Some(UnaryOperator::Ref)
                }
            }
            TokenKind::Star => Some(UnaryOperator::Deref),
            _ => None,
        };

        if let Some(u_op) = op {
            self.advance();
            let operand = self.parse_unary_level_4()?;
            return Ok(Expr {
                kind: ExprKind::Unary {
                    op: u_op,
                    expr: Box::new(operand),
                },
                loc: curr.loc,
            });
        }

        self.parse_postfix_level_1_to_3()
    }

    // Levels 1-3: Postfix (., ?., [], (), ?, ?!)
    fn parse_postfix_level_1_to_3(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_primary()?;

        loop {
            // Level 3: Postfix ? (Try) and ?! (AtomicTry)
            if self.match_token(&TokenKind::AtomicTry) {
                let loc = self.tokens[self.cursor - 1].loc.clone();
                expr = Expr {
                    kind: ExprKind::AtomicTry(Box::new(expr)),
                    loc,
                };
                continue;
            }
            if self.match_token(&TokenKind::Try) {
                let loc = self.tokens[self.cursor - 1].loc.clone();
                expr = Expr {
                    kind: ExprKind::Try(Box::new(expr)),
                    loc,
                };
                continue;
            }

            // Level 1: Field Access (.) and Safe Field Access (?.)
            if self.match_token(&TokenKind::SafeDot) {
                let (field, floc) = self.expect_identifier()?;
                expr = Expr {
                    kind: ExprKind::SafeFieldAccess {
                        expr: Box::new(expr),
                        field,
                    },
                    loc: floc,
                };
                continue;
            }
            if self.match_token(&TokenKind::Dot) {
                let (field, floc) = self.expect_identifier()?;
                if self.check(&TokenKind::LParen) {
                    // Method call
                    self.advance(); // (
                    let args = self.parse_arg_list()?;
                    self.expect(&TokenKind::RParen)?;
                    expr = Expr {
                        kind: ExprKind::MethodCall {
                            receiver: Box::new(expr),
                            method: field,
                            args,
                        },
                        loc: floc,
                    };
                } else {
                    expr = Expr {
                        kind: ExprKind::FieldAccess {
                            expr: Box::new(expr),
                            field,
                        },
                        loc: floc,
                    };
                }
                continue;
            }

            // Level 2: Index Access ([])
            if self.match_token(&TokenKind::LBracket) {
                let index_expr = self.parse_expression()?;
                let rloc = self.expect(&TokenKind::RBracket)?.loc;
                expr = Expr {
                    kind: ExprKind::Index {
                        target: Box::new(expr),
                        index: Box::new(index_expr),
                    },
                    loc: rloc,
                };
                continue;
            }

            // Level 2: Function Call (())
            if self.match_token(&TokenKind::LParen) {
                let args = self.parse_arg_list()?;
                let rloc = self.expect(&TokenKind::RParen)?.loc;
                expr = Expr {
                    kind: ExprKind::Call {
                        callee: Box::new(expr),
                        args,
                    },
                    loc: rloc,
                };
                continue;
            }

            break;
        }

        Ok(expr)
    }

    fn parse_arg_list(&mut self) -> Result<Vec<Expr>, String> {
        let mut args = Vec::new();
        while !self.check(&TokenKind::RParen) && !self.check(&TokenKind::Eof) {
            args.push(self.parse_expression()?);
            if !self.match_token(&TokenKind::Comma) {
                break;
            }
        }
        Ok(args)
    }

    // Primary Expressions
    fn parse_primary(&mut self) -> Result<Expr, String> {
        let curr = self.current().clone();

        match &curr.kind {
            TokenKind::Integer(n) => {
                let val = *n;
                self.advance();
                Ok(Expr {
                    kind: ExprKind::Literal(Literal::Integer(val)),
                    loc: curr.loc,
                })
            }
            TokenKind::Float(f) => {
                let val = *f;
                self.advance();
                Ok(Expr {
                    kind: ExprKind::Literal(Literal::Float(val)),
                    loc: curr.loc,
                })
            }
            TokenKind::StringLiteral(s) => {
                let val = s.clone();
                self.advance();
                Ok(Expr {
                    kind: ExprKind::Literal(Literal::String(val)),
                    loc: curr.loc,
                })
            }
            TokenKind::True => {
                self.advance();
                Ok(Expr {
                    kind: ExprKind::Literal(Literal::Boolean(true)),
                    loc: curr.loc,
                })
            }
            TokenKind::False => {
                self.advance();
                Ok(Expr {
                    kind: ExprKind::Literal(Literal::Boolean(false)),
                    loc: curr.loc,
                })
            }
            TokenKind::NoneVal => {
                self.advance();
                Ok(Expr {
                    kind: ExprKind::Literal(Literal::None),
                    loc: curr.loc,
                })
            }
            TokenKind::LParen => {
                self.advance();
                if self.match_token(&TokenKind::RParen) {
                    return Ok(Expr {
                        kind: ExprKind::Literal(Literal::None),
                        loc: curr.loc,
                    });
                }
                let inner = self.parse_expression()?;
                self.expect(&TokenKind::RParen)?;
                Ok(inner)
            }
            TokenKind::LBracket => {
                // List literal or empty list
                self.advance();
                let mut elements = Vec::new();
                while !self.check(&TokenKind::RBracket) && !self.check(&TokenKind::Eof) {
                    elements.push(self.parse_expression()?);
                    if !self.match_token(&TokenKind::Comma) {
                        break;
                    }
                }
                self.expect(&TokenKind::RBracket)?;
                Ok(Expr {
                    kind: ExprKind::ListLiteral(elements),
                    loc: curr.loc,
                })
            }
            TokenKind::Identifier(name) => {
                let mut qual_name = name.clone();
                let loc = curr.loc.clone();
                self.advance();

                while self.match_token(&TokenKind::DoubleColon) {
                    let (next, _) = self.expect_identifier()?;
                    qual_name.push_str("::");
                    qual_name.push_str(&next);
                }

                // Check for struct instantiation: Name { field: expr, ... }
                if self.check(&TokenKind::LBrace) {
                    let mut is_struct_init = false;
                    if self.cursor + 2 < self.tokens.len() {
                        if let TokenKind::Identifier(_) = &self.tokens[self.cursor + 1].kind {
                            if self.tokens[self.cursor + 2].kind == TokenKind::Colon {
                                is_struct_init = true;
                            }
                        }
                    }

                    if is_struct_init {
                        self.advance(); // {
                            let mut fields = Vec::new();
                            while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::Eof) {
                                let (fname, _) = self.expect_identifier()?;
                                self.expect(&TokenKind::Colon)?;
                                let fval = self.parse_expression()?;
                                fields.push((fname, fval));
                                if !self.match_token(&TokenKind::Comma) {
                                    break;
                                }
                            }
                            self.expect(&TokenKind::RBrace)?;
                            return Ok(Expr {
                                kind: ExprKind::StructInit {
                                    name: qual_name,
                                    fields,
                                },
                                loc,
                            });
                        }
                    }

                if qual_name.contains("::") {
                    let parts = qual_name.split("::").map(String::from).collect();
                    Ok(Expr {
                        kind: ExprKind::Qualified(parts),
                        loc,
                    })
                } else {
                    Ok(Expr {
                        kind: ExprKind::Identifier(qual_name),
                        loc,
                    })
                }
            }
            _ => Err(format!(
                "Parse error at {}: unexpected token {:?} in expression",
                curr.loc, curr.kind
            )),
        }
    }
}
