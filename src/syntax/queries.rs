//! Independently cached query roles over the grammars linked by the Rust reader.

use super::SupportedGrammar;
use std::{collections::HashMap, sync::OnceLock};
use tree_sitter::Query;

#[derive(Clone, Copy)]
pub(super) enum QueryRole {
    Calls,
    Folds,
    Occurrences,
}
type Queries = HashMap<SupportedGrammar, Result<Query, String>>;

pub(super) fn query(grammar: SupportedGrammar, role: QueryRole) -> Result<&'static Query, String> {
    static CALLS: OnceLock<Queries> = OnceLock::new();
    static FOLDS: OnceLock<Queries> = OnceLock::new();
    static OCCURRENCES: OnceLock<Queries> = OnceLock::new();
    let cache = match role {
        QueryRole::Calls => &CALLS,
        QueryRole::Folds => &FOLDS,
        QueryRole::Occurrences => &OCCURRENCES,
    };
    use SupportedGrammar::*;
    cache
        .get_or_init(|| {
            [
                Rust, Python, Go, JavaScript, TypeScript, Tsx, Swift, Kotlin, C,
            ]
            .into_iter()
            .map(|grammar| {
                let source = match role {
                    QueryRole::Calls => call_source(grammar),
                    QueryRole::Folds => fold_source(grammar),
                    QueryRole::Occurrences => occurrence_source(grammar),
                };
                (
                    grammar,
                    Query::new(&grammar.language(), &source).map_err(|error| error.to_string()),
                )
            })
            .collect()
        })
        .get(&grammar)
        .expect("registered reader grammar")
        .as_ref()
        .map_err(Clone::clone)
}

fn call_source(grammar: SupportedGrammar) -> String {
    use SupportedGrammar::*;
    let source = match grammar {
        Rust => include_str!("../../Sources/Radiology/Queries/rust/calls.scm"),
        Python => include_str!("../../Sources/Radiology/Queries/python/calls.scm"),
        Go => include_str!("../../Sources/Radiology/Queries/go/calls.scm"),
        JavaScript | TypeScript | Tsx => include_str!("../../Sources/Radiology/Queries/ecmascript/calls.scm"),
        Swift => include_str!("../../Sources/Radiology/Queries/swift/calls.scm"),
        Kotlin => "(call_expression . (identifier) @call.name) @call\n(call_expression (navigation_expression (identifier) @call.name .)) @call\n(call_expression) @call.any",
        C => "(call_expression function: (identifier) @call.name) @call\n(call_expression function: (field_expression field: (field_identifier) @call.name)) @call\n(call_expression) @call.any",
    };
    format!("{source}\n{}", scope_source(grammar))
}

fn scope_source(grammar: SupportedGrammar) -> String {
    use SupportedGrammar::*;
    let scopes = match grammar {
        Rust => include_str!("../../Sources/Radiology/Queries/rust/scopes.scm"),
        Python => include_str!("../../Sources/Radiology/Queries/python/scopes.scm"),
        Go => include_str!("../../Sources/Radiology/Queries/go/scopes.scm"),
        JavaScript | TypeScript | Tsx => {
            include_str!("../../Sources/Radiology/Queries/ecmascript/scopes.scm")
        }
        Swift => include_str!("../../Sources/Radiology/Queries/swift/scopes.scm"),
        Kotlin => include_str!("../../Sources/Radiology/Queries/kotlin/scopes.scm"),
        C => "(function_definition) @scope.function",
    };
    let scopes = scopes
        .lines()
        .filter(|line| line.contains("@scope.function") || line.contains("@scope.closure"))
        .collect::<Vec<_>>()
        .join("\n");
    let protocol_scope = if grammar == Swift {
        "(protocol_function_declaration) @scope.function"
    } else {
        ""
    };
    format!("{scopes}\n{protocol_scope}")
}

fn fold_source(grammar: SupportedGrammar) -> String {
    use SupportedGrammar::*;
    let (folds, symbols) = match grammar {
        Rust => (include_str!("../../Sources/Radiology/Queries/rust/folds.scm"), include_str!("../../Sources/Radiology/Queries/rust/symbols.scm")),
        Python => (include_str!("../../Sources/Radiology/Queries/python/folds.scm"), include_str!("../../Sources/Radiology/Queries/python/symbols.scm")),
        Go => (include_str!("../../Sources/Radiology/Queries/go/folds.scm"), include_str!("../../Sources/Radiology/Queries/go/symbols.scm")),
        JavaScript | TypeScript | Tsx => (include_str!("../../Sources/Radiology/Queries/ecmascript/folds.scm"), include_str!("../../Sources/Radiology/Queries/ecmascript/symbols.scm")),
        Swift => (include_str!("../../Sources/Radiology/Queries/swift/folds.scm"), include_str!("../../Sources/Radiology/Queries/swift/symbols.scm")),
        Kotlin => (include_str!("../../Sources/Radiology/Queries/kotlin/folds.scm"), include_str!("../../Sources/Radiology/Queries/kotlin/symbols.scm")),
        C => ("(compound_statement) @fold.block\n(function_definition body: (compound_statement) @fold.function)\n(field_declaration_list) @fold.type\n(enumerator_list) @fold.type\n(initializer_list) @fold.block\n(comment) @fold.comment",
            "(function_definition declarator: (_) @name) @symbol.function\n(struct_specifier name: (type_identifier) @name) @symbol.type\n(enum_specifier name: (type_identifier) @name) @symbol.enum\n(enumerator name: (identifier) @name) @symbol.case\n(type_definition declarator: (_) @name) @symbol.typealias\n(translation_unit (declaration declarator: (_) @name) @symbol.variable)"),
    };
    if grammar == Kotlin {
        // kotlin-ng uses different identifiers, blocks, imports and comment names.
        let folds = folds
            .replace("(statements)", "(block)")
            .replace("(control_structure_body)", "(block)")
            .replace(
                "(import_list) @fold.imports",
                "(source_file (import) @fold.imports.member)",
            )
            .replace("(multiline_comment)", "(block_comment)");
        let symbols = symbols
            .replace("(type_identifier)", "(identifier)")
            .replace("(simple_identifier)", "(identifier)");
        format!("{folds}\n{symbols}")
    } else {
        format!("{folds}\n{symbols}")
    }
}

fn occurrence_source(grammar: SupportedGrammar) -> String {
    use SupportedGrammar::*;
    let highlights = match grammar {
        Rust => include_str!("../../Sources/Radiology/Queries/rust/highlights.scm"),
        Python => include_str!("../../Sources/Radiology/Queries/python/highlights.scm"),
        Go => include_str!("../../Sources/Radiology/Queries/go/highlights.scm"),
        JavaScript | TypeScript | Tsx => include_str!("../../Sources/Radiology/Queries/ecmascript/highlights.scm"),
        Swift => include_str!("../../Sources/Radiology/Queries/swift/highlights.scm"),
        // Native grammar names differ from Swift's fwcd Kotlin grammar. These
        // captures preserve the same exclusion classes, not semantic bindings.
        Kotlin => "[(line_comment) (block_comment) (shebang)] @comment\n[(string_literal) (multiline_string_literal) (character_literal)] @string\n[(number_literal) (float_literal)] @number\n[\"fun\" \"return\" \"return@\" \"import\" \"if\" \"else\" \"when\" \"for\" \"do\" \"while\" \"try\" \"catch\" \"throw\" \"finally\" \"val\" \"var\" \"class\" \"object\" \"interface\" \"enum\" \"typealias\" \"is\" \"in\" \"as\" \"as?\"] @keyword\n[(class_modifier) (member_modifier) (function_modifier) (property_modifier) (platform_modifier) (variance_modifier) (parameter_modifier) (visibility_modifier) (reification_modifier) (inheritance_modifier)] @keyword.modifier",
        C => "(comment) @comment\n[(string_literal) (char_literal)] @string\n(number_literal) @number\n[\"if\" \"else\" \"for\" \"while\" \"do\" \"return\" \"break\" \"continue\" \"switch\" \"case\" \"default\" \"struct\" \"union\" \"enum\" \"typedef\" \"const\" \"static\" \"extern\" \"volatile\" \"sizeof\" \"goto\"] @keyword",
    };
    let highlights = if grammar == Swift {
        highlights.replace("\"nil\"", "(nil_literal)")
    } else {
        highlights.to_owned()
    };
    let overlay = match grammar {
        TypeScript | Tsx => {
            include_str!("../../Sources/Radiology/Queries/typescript/highlights.scm")
        }
        _ => "",
    };
    format!("{}\n{highlights}\n{overlay}", scope_source(grammar))
}
