//! Python module conventions and ECMAScript framework calls/bindings.
use super::*;
use std::collections::HashSet;

const TEST_MODULES: &[&str] = &[
    "vitest",
    "bun:test",
    "@jest/globals",
    "node:test",
    "mocha",
    "@playwright/test",
    "jest",
    "uvu",
    "ava",
];
const TEST_CALLS: &[&str] = &[
    "describe",
    "it",
    "test",
    "suite",
    "context",
    "xit",
    "xdescribe",
    "fit",
    "fdescribe",
    "beforeEach",
    "afterEach",
    "beforeAll",
    "afterAll",
];

impl Detector<'_> {
    pub(super) fn python_regions(&self, root: Node<'_>, output: &mut Vec<TestRegion>) {
        let siblings = children(root);
        let pytest = siblings.iter().any(|node| self.imports_pytest(*node));
        for (index, node) in siblings.iter().copied().enumerate() {
            if self.control.is_cancelled() {
                break;
            }
            if self.python_test(node, pytest) {
                output.push(self.region(&siblings, index, index));
            }
        }
    }

    fn imports_pytest(&self, node: Node<'_>) -> bool {
        match node.kind() {
            "import_from_statement" => node
                .child_by_field_name("module_name")
                .is_some_and(|name| self.text(name).split('.').next() == Some("pytest")),
            "import_statement" => children(node).iter().any(|name| {
                let target = name.child_by_field_name("name").unwrap_or(*name);
                self.text(target).split('.').next() == Some("pytest")
            }),
            _ => false,
        }
    }

    fn python_test(&self, mut node: Node<'_>, pytest: bool) -> bool {
        if node.kind() == "decorated_definition" {
            if pytest
                && children(node).iter().any(|child| {
                    child.kind() == "decorator"
                        && (self.text(*child).starts_with("@pytest.fixture")
                            || self.text(*child).starts_with("@pytest.mark."))
                })
            {
                return true;
            }
            let Some(definition) = node.child_by_field_name("definition") else {
                return false;
            };
            node = definition;
        }
        let name = node
            .child_by_field_name("name")
            .map(|name| self.text(name))
            .unwrap_or("");
        match node.kind() {
            "function_definition" => pytest && name.starts_with("test_"),
            "class_definition" => {
                self.python_test_class(node) || (pytest && name.starts_with("Test"))
            }
            _ => false,
        }
    }

    fn python_test_class(&self, node: Node<'_>) -> bool {
        node.child_by_field_name("superclasses")
            .is_some_and(|bases| {
                children(bases).iter().any(|base| {
                    matches!(base.kind(), "identifier" | "attribute")
                        && self
                            .text(*base)
                            .split('.')
                            .next_back()
                            .is_some_and(|name| name.ends_with("TestCase"))
                })
            })
    }

    pub(super) fn script_regions(&self, root: Node<'_>, output: &mut Vec<TestRegion>) {
        let siblings = children(root);
        let mut local = HashSet::new();
        let mut imported = HashSet::new();
        for node in &siblings {
            if self.control.is_cancelled() {
                break;
            }
            self.script_bindings(*node, &mut local, &mut imported);
        }
        for (index, node) in siblings.iter().copied().enumerate() {
            if self.control.is_cancelled() {
                break;
            }
            let test = match node.kind() {
                "if_statement" => {
                    node.child_by_field_name("alternative").is_none()
                        && node
                            .child_by_field_name("condition")
                            .is_some_and(|condition| {
                                self.text(condition)
                                    .chars()
                                    .filter(|character| !character.is_whitespace())
                                    .collect::<String>()
                                    == "(import.meta.vitest)"
                            })
                }
                "expression_statement" => children(node)
                    .first()
                    .is_some_and(|call| self.script_test_call(*call, &local, &imported)),
                _ => false,
            };
            if test {
                output.push(self.region(&siblings, index, index));
            }
        }
    }

    fn script_test_call(
        &self,
        call: Node<'_>,
        local: &HashSet<String>,
        imported: &HashSet<String>,
    ) -> bool {
        if call.kind() != "call_expression" {
            return false;
        }
        let callback = call.child_by_field_name("arguments").is_some_and(|args| {
            children(args).iter().any(|node| {
                matches!(
                    node.kind(),
                    "arrow_function" | "function_expression" | "function" | "generator_function"
                )
            })
        });
        if !callback {
            return false;
        }
        let Some(path) = self.callee_path(call) else {
            return false;
        };
        let Some(root) = path.first() else {
            return false;
        };
        if root == "Deno" {
            return path.get(1).is_some_and(|part| part == "test") && !local.contains(root);
        }
        TEST_CALLS.contains(&root.as_str()) && (imported.contains(root) || !local.contains(root))
    }

    fn callee_path(&self, call: Node<'_>) -> Option<Vec<String>> {
        let mut node = call.child_by_field_name("function")?;
        let mut path = Vec::new();
        loop {
            match node.kind() {
                "identifier" => {
                    path.push(self.text(node).to_owned());
                    path.reverse();
                    return Some(path);
                }
                "member_expression" => {
                    path.push(self.text(node.child_by_field_name("property")?).to_owned());
                    node = node.child_by_field_name("object")?;
                }
                "call_expression" => node = node.child_by_field_name("function")?,
                _ => return None,
            }
        }
    }

    fn script_bindings(
        &self,
        node: Node<'_>,
        local: &mut HashSet<String>,
        imported: &mut HashSet<String>,
    ) {
        match node.kind() {
            "import_statement" => {
                let known = node.child_by_field_name("source").is_some_and(|source| {
                    TEST_MODULES.contains(&self.text(source).trim_matches(['\'', '"', '`']))
                });
                let target = if known { imported } else { local };
                if let Some(clause) = children(node)
                    .into_iter()
                    .find(|node| node.kind() == "import_clause")
                {
                    self.import_names(clause, target);
                }
            }
            "lexical_declaration" | "variable_declaration" => {
                for declaration in children(node) {
                    if self.control.is_cancelled() {
                        break;
                    }
                    let known = declaration
                        .child_by_field_name("value")
                        .is_some_and(|value| self.test_require(value));
                    if let Some(name) = declaration.child_by_field_name("name") {
                        self.pattern_names(name, if known { imported } else { local });
                    }
                }
            }
            "function_declaration" | "generator_function_declaration" | "class_declaration" => {
                if let Some(name) = node.child_by_field_name("name") {
                    local.insert(self.text(name).to_owned());
                }
            }
            "export_statement" => {
                for child in children(node) {
                    if self.control.is_cancelled() {
                        break;
                    }
                    self.script_bindings(child, local, imported);
                }
            }
            _ => {}
        }
    }

    fn import_names(&self, root: Node<'_>, names: &mut HashSet<String>) {
        let mut pending = vec![root];
        while let Some(node) = pending.pop() {
            if self.control.is_cancelled() {
                break;
            }
            match node.kind() {
                "import_specifier" => {
                    if let Some(name) = node
                        .child_by_field_name("alias")
                        .or_else(|| node.child_by_field_name("name"))
                    {
                        names.insert(self.text(name).to_owned());
                    }
                }
                "identifier" => {
                    names.insert(self.text(node).to_owned());
                }
                _ => pending.extend(children(node)),
            }
        }
    }

    fn pattern_names(&self, root: Node<'_>, names: &mut HashSet<String>) {
        let mut pending = vec![root];
        while let Some(node) = pending.pop() {
            if self.control.is_cancelled() {
                break;
            }
            if matches!(
                node.kind(),
                "identifier" | "shorthand_property_identifier_pattern"
            ) {
                names.insert(self.text(node).to_owned());
            } else {
                pending.extend(children(node));
            }
        }
    }

    fn test_require(&self, node: Node<'_>) -> bool {
        node.kind() == "call_expression"
            && node
                .child_by_field_name("function")
                .is_some_and(|callee| self.text(callee) == "require")
            && node
                .child_by_field_name("arguments")
                .and_then(|args| children(args).first().copied())
                .is_some_and(|argument| {
                    argument.kind() == "string"
                        && TEST_MODULES
                            .contains(&self.text(argument).trim_matches(['\'', '"', '`']))
                })
    }
}
