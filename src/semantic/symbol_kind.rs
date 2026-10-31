//! Shared SCIP kind categories for semantic classification and symbol presentation.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SymbolCategory {
    Type,
    Function,
    Value,
    Module,
    Other,
}

impl SymbolCategory {
    pub(crate) fn from_kind(kind: i32) -> Self {
        // Keep the category contract aligned with Swift's SemanticSymbolKind.
        match kind {
            7 | 11 | 21 | 42 | 49 | 53 | 54 | 55 | 56 | 59 | 3 | 84 | 85 | 10 | 57 | 58 | 36 => {
                Self::Type
            }
            17 | 26 | 9 | 18 | 45 | 66 | 67 | 68 | 69 | 70 | 71 | 72 | 74 | 76 | 80 | 25 | 34 => {
                Self::Function
            }
            8 | 12 | 15 | 41 | 61 | 37 | 77 | 79 | 81 | 82 | 44 | 52 | 60 | 22 => Self::Value,
            29 | 30 | 35 | 16 | 64 => Self::Module,
            _ => Self::Other,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wire_kind_categories_preserve_unknowns_and_callable_variants() {
        for (kind, category, name) in [
            (7, SymbolCategory::Type, "Class"),
            (84, SymbolCategory::Type, "Extension"),
            (17, SymbolCategory::Function, "Function"),
            (26, SymbolCategory::Function, "Method"),
            (25, SymbolCategory::Function, "Macro"),
            (61, SymbolCategory::Value, "Variable"),
            (35, SymbolCategory::Module, "Package"),
            (0, SymbolCategory::Other, "Symbol"),
            (-7, SymbolCategory::Other, "Kind -7"),
            (9001, SymbolCategory::Other, "Kind 9001"),
        ] {
            assert_eq!(SymbolCategory::from_kind(kind), category);
            assert_eq!(crate::semantic::context::kind_name(kind), name);
        }
    }
}
