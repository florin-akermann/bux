//! Helpers shared by the parser's behaviour tests.
//!
//! [`render`] prints a parse tree as one line per node, `<indent><node> <start>..<end>`, which
//! is the format of a `tests/spec/parser/<name>.ast` file. Reading a rendered tree beside its
//! source is how a parser behaviour is stated here.

use std::fmt::Write as _;

use lumen_ast::InstanceDeclaration;
use lumen_ast::{Arguments, Block, Expr, ExprKind, IfExpr, Item, MatchExpr, Pattern, PatternKind};
use lumen_ast::{DeriveDeclaration, ExternDeclaration, Function, Import, RecordField};
use lumen_ast::{Signature, TraitDeclaration};
use lumen_ast::{Span, Statement, StatementKind, TypeDefinition, TypeRef, TypeRefKind};
use lumen_ast::{TypeDeclaration, TypeParameter, Variant, VariantPayload};
use lumen_parser::{ParseError, parse};

/// The rendered tree of `source` below its first `above` lines, unindented by `depth` levels.
///
/// A test about one construct wraps it in the smallest item that can hold it, then reads only
/// the part it is about.
pub fn inner(source: &str, above: usize, depth: usize) -> Vec<String> {
    shape(source)
        .split_off(above)
        .iter()
        .map(|line| line[depth * 2..].to_owned())
        .collect()
}

/// The rendered tree with the spans dropped, for a behaviour that is about shape alone.
///
/// Spans are specified by the executable examples under `tests/spec/parser/`, whose `.ast`
/// files keep them, and by the tests that are about spans.
pub fn shape(source: &str) -> Vec<String> {
    render(source).lines().map(without_span).collect()
}

fn without_span(line: &str) -> String {
    match line.rsplit_once(' ') {
        Some((head, tail)) if is_span(tail) => head.to_owned(),
        _ => line.to_owned(),
    }
}

fn is_span(text: &str) -> bool {
    text.split_once("..").is_some_and(|(start, end)| {
        !start.is_empty()
            && !end.is_empty()
            && start.bytes().all(|byte| byte.is_ascii_digit())
            && end.bytes().all(|byte| byte.is_ascii_digit())
    })
}

/// The parse tree of `source`, rendered one node per line.
pub fn render(source: &str) -> String {
    let program =
        parse(source).unwrap_or_else(|error| panic!("{source:?} parses: {}", error.message()));
    let mut tree = Tree::default();
    for item in &program.items {
        item_node(&mut tree, 0, item);
    }
    tree.text
}

/// The error `source` fails with, rendered as a `tests/spec/parser/<name>.error` file is.
pub fn render_error(source: &str) -> String {
    let error = parse(source)
        .err()
        .unwrap_or_else(|| panic!("{source:?} does not parse"));
    parse_error(&error)
}

fn parse_error(error: &ParseError) -> String {
    let span = error.span();
    let mut rendered = format!("{}..{} {}\n", span.start(), span.end(), error.message());
    if let Some(help) = error.help() {
        let _ = writeln!(rendered, "help: {help}");
    }
    rendered
}

/// The rendered tree, built one indented line at a time.
#[derive(Default)]
struct Tree {
    text: String,
}

impl Tree {
    fn node(&mut self, depth: usize, label: &str, span: Span) {
        let indent = "  ".repeat(depth);
        let _ = writeln!(
            self.text,
            "{indent}{label} {}..{}",
            span.start(),
            span.end()
        );
    }

    /// A node that groups others and has no span of its own, such as `result`.
    fn group(&mut self, depth: usize, label: &str) {
        let indent = "  ".repeat(depth);
        let _ = writeln!(self.text, "{indent}{label}");
    }
}

fn item_node(tree: &mut Tree, depth: usize, item: &Item) {
    match item {
        Item::Import(import) => import_node(tree, depth, import),
        Item::Type(declaration) => type_declaration_node(tree, depth, declaration),
        Item::Trait(declaration) => trait_node(tree, depth, declaration),
        Item::Instance(declaration) => instance_node(tree, depth, declaration),
        Item::Derive(declaration) => derive_node(tree, depth, declaration),
        Item::Function(function) => function_node(tree, depth, function),
        Item::Extern(declaration) => extern_node(tree, depth, declaration),
    }
}

fn extern_node(tree: &mut Tree, depth: usize, declaration: &ExternDeclaration) {
    let reaches = declaration.reaches.written();
    tree.node(
        depth,
        &format!("extern {reaches} {}", declaration.name.text),
        declaration.span,
    );
    if let Some(named) = declaration.reaches.named() {
        tree.node(depth + 1, &format!("java {}", named.text), named.span);
    }
    for parameter in &declaration.parameters {
        tree.node(
            depth + 1,
            &format!("parameter {}", parameter.name.text),
            parameter.span,
        );
        if let Some(type_ref) = &parameter.type_ref {
            type_ref_node(tree, depth + 2, type_ref);
        }
    }
    tree.group(depth + 1, "result");
    type_ref_node(tree, depth + 2, &declaration.result);
}

fn trait_node(tree: &mut Tree, depth: usize, declaration: &TraitDeclaration) {
    tree.node(
        depth,
        &format!("trait {}", declaration.name.text),
        declaration.span,
    );
    tree.node(
        depth + 1,
        &format!("type-parameter {}", declaration.parameter.text),
        declaration.parameter.span,
    );
    for method in &declaration.methods {
        signature_node(tree, depth + 1, method);
    }
}

fn signature_node(tree: &mut Tree, depth: usize, method: &Signature) {
    tree.node(
        depth,
        &format!("signature {}", method.name.text),
        method.span,
    );
    for parameter in &method.parameters {
        tree.node(
            depth + 1,
            &format!("parameter {}", parameter.name.text),
            parameter.span,
        );
        if let Some(type_ref) = &parameter.type_ref {
            type_ref_node(tree, depth + 2, type_ref);
        }
    }
    if let Some(result) = &method.result {
        tree.group(depth + 1, "result");
        type_ref_node(tree, depth + 2, result);
    }
}

fn instance_node(tree: &mut Tree, depth: usize, declaration: &InstanceDeclaration) {
    let label = format!(
        "instance {}<{}>",
        declaration.trait_name.text, declaration.for_type.text
    );
    tree.node(depth, &label, declaration.span);
    for method in &declaration.methods {
        function_node(tree, depth + 1, method);
    }
}

fn derive_node(tree: &mut Tree, depth: usize, declaration: &DeriveDeclaration) {
    let named: Vec<&str> = declaration
        .traits
        .iter()
        .map(|one| one.text.as_str())
        .collect();
    let label = format!(
        "derive {} for {}",
        named.join(", "),
        declaration.for_type.text
    );
    tree.node(depth, &label, declaration.span);
}

fn import_node(tree: &mut Tree, depth: usize, import: &Import) {
    tree.node(
        depth,
        &format!("import {}", import.module.text),
        import.span,
    );
}

fn type_declaration_node(tree: &mut Tree, depth: usize, declaration: &TypeDeclaration) {
    let label = format!("type {}", declaration.name.text);
    tree.node(depth, &label, declaration.span);
    for parameter in &declaration.parameters {
        tree.node(
            depth + 1,
            &format!("type-parameter {}", parameter.text),
            parameter.span,
        );
    }
    match &declaration.definition {
        TypeDefinition::Record(fields) => record_fields(tree, depth + 1, fields),
        TypeDefinition::Variants(variants) => {
            for variant in variants {
                variant_node(tree, depth + 1, variant);
            }
        }
        TypeDefinition::Foreign(class) => {
            tree.node(depth + 1, &format!("java {}", class.text), class.span);
        }
    }
}

fn variant_node(tree: &mut Tree, depth: usize, variant: &Variant) {
    tree.node(
        depth,
        &format!("variant {}", variant.name.text),
        variant.span,
    );
    match &variant.payload {
        VariantPayload::None => {}
        VariantPayload::Tuple(types) => {
            for type_ref in types {
                type_ref_node(tree, depth + 1, type_ref);
            }
        }
        VariantPayload::Record(fields) => record_fields(tree, depth + 1, fields),
    }
}

fn function_node(tree: &mut Tree, depth: usize, function: &Function) {
    tree.node(
        depth,
        &format!("function {}", function.name.text),
        function.span,
    );
    for parameter in &function.type_parameters {
        type_parameter_node(tree, depth + 1, parameter);
    }
    for parameter in &function.parameters {
        tree.node(
            depth + 1,
            &format!("parameter {}", parameter.name.text),
            parameter.span,
        );
        if let Some(type_ref) = &parameter.type_ref {
            type_ref_node(tree, depth + 2, type_ref);
        }
    }
    if let Some(result) = &function.result {
        tree.group(depth + 1, "result");
        type_ref_node(tree, depth + 2, result);
    }
    block_node(tree, depth + 1, &function.body);
}

/// A `<T>` of a function, with the trait it is constrained by when the author wrote one.
fn type_parameter_node(tree: &mut Tree, depth: usize, parameter: &TypeParameter) {
    tree.node(
        depth,
        &format!("type-parameter {}", parameter.name.text),
        parameter.name.span,
    );
    if let Some(constraint) = &parameter.constraint {
        tree.node(
            depth + 1,
            &format!("constraint {}", constraint.name.text),
            constraint.span,
        );
        type_ref_node(tree, depth + 2, &constraint.argument);
    }
}

fn record_fields(tree: &mut Tree, depth: usize, fields: &[RecordField]) {
    for field in fields {
        tree.node(depth, &format!("field {}", field.name.text), field.span);
        type_ref_node(tree, depth + 1, &field.type_ref);
    }
}

fn type_ref_node(tree: &mut Tree, depth: usize, type_ref: &TypeRef) {
    match &type_ref.kind {
        TypeRefKind::Unit => tree.node(depth, "unit-type", type_ref.span),
        TypeRefKind::Named { path, arguments } => {
            tree.node(depth, &format!("named-type {path}"), type_ref.span);
            for argument in arguments {
                type_ref_node(tree, depth + 1, argument);
            }
        }
    }
}

fn block_node(tree: &mut Tree, depth: usize, block: &Block) {
    tree.node(depth, "block", block.span);
    for statement in &block.statements {
        statement_node(tree, depth + 1, statement);
    }
}

fn statement_node(tree: &mut Tree, depth: usize, statement: &Statement) {
    let span = statement.span;
    match &statement.kind {
        StatementKind::Binding {
            mutability,
            name,
            value,
        } => {
            tree.node(
                depth,
                &format!("binding {mutability:?} {}", name.text),
                span,
            );
            expr_node(tree, depth + 1, value);
        }
        StatementKind::Assign {
            target,
            operator,
            value,
        } => {
            tree.node(depth, &format!("assign {operator:?}"), span);
            tree.node(depth + 1, &format!("name {}", target.text), target.span);
            expr_node(tree, depth + 1, value);
        }
        StatementKind::Return(value) => {
            tree.node(depth, "return", span);
            if let Some(value) = value {
                expr_node(tree, depth + 1, value);
            }
        }
        StatementKind::Break => tree.node(depth, "break", span),
        StatementKind::Continue => tree.node(depth, "continue", span),
        StatementKind::For(loop_) => for_node(tree, depth, span, loop_),
        StatementKind::Discard(expr) => {
            tree.node(depth, "discard", span);
            expr_node(tree, depth + 1, expr);
        }
        StatementKind::Expr(expr) => expr_node(tree, depth, expr),
    }
}

fn for_node(tree: &mut Tree, depth: usize, span: Span, loop_: &lumen_ast::ForLoop) {
    match &loop_.header {
        lumen_ast::ForHeader::Forever => tree.node(depth, "for", span),
        lumen_ast::ForHeader::While(condition) => {
            tree.node(depth, "for-while", span);
            expr_node(tree, depth + 1, condition);
        }
        lumen_ast::ForHeader::In { binding, iterable } => {
            tree.node(depth, &format!("for-in {}", binding.text), span);
            expr_node(tree, depth + 1, iterable);
        }
    }
    block_node(tree, depth + 1, &loop_.body);
}

fn expr_node(tree: &mut Tree, depth: usize, expr: &Expr) {
    let span = expr.span;
    match &expr.kind {
        ExprKind::Name(name) => tree.node(depth, &format!("name {}", name.text), span),
        ExprKind::Integer(value) => tree.node(depth, &format!("integer {value}"), span),
        ExprKind::String(value) => tree.node(depth, &format!("string {value:?}"), span),
        ExprKind::Bool(value) => tree.node(depth, &format!("bool {value}"), span),
        ExprKind::Unit => tree.node(depth, "unit", span),
        ExprKind::Unary { operator, operand } => {
            tree.node(depth, &format!("unary {operator:?}"), span);
            expr_node(tree, depth + 1, operand);
        }
        ExprKind::Binary {
            operator,
            left,
            right,
        } => {
            tree.node(depth, &format!("binary {operator:?}"), span);
            expr_node(tree, depth + 1, left);
            expr_node(tree, depth + 1, right);
        }
        ExprKind::Call { callee, arguments } => {
            tree.node(depth, "call", span);
            expr_node(tree, depth + 1, callee);
            argument_nodes(tree, depth + 1, arguments);
        }
        ExprKind::Field { receiver, name } => {
            tree.node(depth, &format!("field {}", name.text), span);
            expr_node(tree, depth + 1, receiver);
        }
        ExprKind::Try(inner) => {
            tree.node(depth, "try", span);
            expr_node(tree, depth + 1, inner);
        }
        ExprKind::List(elements) => {
            tree.node(depth, "list", span);
            for element in elements {
                expr_node(tree, depth + 1, element);
            }
        }
        ExprKind::Record { base, fields } => {
            tree.node(depth, &format!("record {base}"), span);
            for field in fields {
                tree.node(
                    depth + 1,
                    &format!("field-value {}", field.name.text),
                    field.span,
                );
                expr_node(tree, depth + 2, &field.value);
            }
        }
        ExprKind::If(if_expr) => if_node(tree, depth, span, if_expr),
        ExprKind::Match(match_expr) => match_node(tree, depth, span, match_expr),
    }
}

/// What a call passes, with each name printed where the call writes one.
fn argument_nodes(tree: &mut Tree, depth: usize, arguments: &Arguments) {
    match arguments {
        Arguments::Positional(values) => {
            for value in values {
                expr_node(tree, depth, value);
            }
        }
        Arguments::Named(written) => {
            for argument in written {
                let label = format!("argument {}", argument.name.text);
                tree.node(depth, &label, argument.span());
                expr_node(tree, depth + 1, &argument.value);
            }
        }
    }
}

fn if_node(tree: &mut Tree, depth: usize, span: Span, if_expr: &IfExpr) {
    tree.node(depth, "if", span);
    for branch in &if_expr.branches {
        tree.node(depth + 1, "branch", branch.span);
        expr_node(tree, depth + 2, &branch.condition);
        block_node(tree, depth + 2, &branch.block);
    }
    if let Some(otherwise) = &if_expr.otherwise {
        tree.group(depth + 1, "else");
        block_node(tree, depth + 2, otherwise);
    }
}

fn match_node(tree: &mut Tree, depth: usize, span: Span, match_expr: &MatchExpr) {
    tree.node(depth, "match", span);
    expr_node(tree, depth + 1, &match_expr.scrutinee);
    for arm in &match_expr.arms {
        tree.node(depth + 1, "arm", arm.span);
        pattern_node(tree, depth + 2, &arm.pattern);
        expr_node(tree, depth + 2, &arm.body);
    }
}

fn pattern_node(tree: &mut Tree, depth: usize, pattern: &Pattern) {
    let span = pattern.span;
    match &pattern.kind {
        PatternKind::Name(path) => tree.node(depth, &format!("pattern {path}"), span),
        PatternKind::Integer(value) => tree.node(depth, &format!("pattern-integer {value}"), span),
        PatternKind::String(value) => tree.node(depth, &format!("pattern-string {value:?}"), span),
        PatternKind::Bool(value) => tree.node(depth, &format!("pattern-bool {value}"), span),
        PatternKind::Tuple { path, elements } => {
            tree.node(depth, &format!("pattern-tuple {path}"), span);
            for element in elements {
                pattern_node(tree, depth + 1, element);
            }
        }
        PatternKind::Record { path, fields } => {
            tree.node(depth, &format!("pattern-record {path}"), span);
            for field in fields {
                tree.node(
                    depth + 1,
                    &format!("pattern-field {}", field.text),
                    field.span,
                );
            }
        }
        PatternKind::Wildcard => tree.node(depth, "pattern-wildcard", span),
        PatternKind::Or(alternatives) => {
            tree.node(depth, "pattern-or", span);
            for alternative in alternatives {
                pattern_node(tree, depth + 1, alternative);
            }
        }
    }
}
