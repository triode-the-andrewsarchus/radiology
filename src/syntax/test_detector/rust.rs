//! Rust attributes and three-valued cfg evaluation, ported from the Swift rules.
use super::*;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Truth {
    Yes,
    No,
    Unknown,
}

impl Truth {
    fn not(self) -> Self {
        match self {
            Self::Yes => Self::No,
            Self::No => Self::Yes,
            Self::Unknown => Self::Unknown,
        }
    }
}

fn predicate(tokens: &[Node<'_>], detector: &Detector<'_>, testing: bool, depth: usize) -> Truth {
    if depth > 64 {
        return Truth::Unknown;
    }
    let Some(head) = tokens.first().filter(|node| node.kind() == "identifier") else {
        return Truth::Unknown;
    };
    let name = detector.text(*head);
    if tokens.len() == 1 {
        return match (name, testing) {
            ("test", true) => Truth::Yes,
            ("test" | "doctest", false) => Truth::No,
            _ => Truth::Unknown,
        };
    }
    if tokens.len() != 2 || tokens[1].kind() != "token_tree" {
        return Truth::Unknown;
    }
    let groups = groups(tokens[1]);
    let values: Vec<_> = groups
        .iter()
        .map(|group| predicate(group, detector, testing, depth + 1))
        .collect();
    match name {
        "all" if values.contains(&Truth::No) => Truth::No,
        "all" if values.iter().all(|value| *value == Truth::Yes) => Truth::Yes,
        "any" if values.contains(&Truth::Yes) => Truth::Yes,
        "any" if values.iter().all(|value| *value == Truth::No) => Truth::No,
        "not" if values.len() == 1 => values[0].not(),
        _ => Truth::Unknown,
    }
}

fn groups(tree: Node<'_>) -> Vec<Vec<Node<'_>>> {
    let mut cursor = tree.walk();
    let tokens: Vec<_> = tree.children(&mut cursor).collect();
    if tokens.len() < 2 {
        return Vec::new();
    }
    let mut groups = vec![Vec::new()];
    for token in &tokens[1..tokens.len() - 1] {
        if token.kind() == "," {
            groups.push(Vec::new());
        } else {
            groups.last_mut().expect("initial group").push(*token);
        }
    }
    if groups.last().is_some_and(Vec::is_empty) {
        groups.pop();
    }
    groups
}

fn attribute_or_comment(node: Node<'_>) -> bool {
    matches!(node.kind(), "attribute_item" | "inner_attribute_item") || is_comment(node)
}

impl Detector<'_> {
    fn rust_attribute_test_only(&self, item: Node<'_>, allow_test_attribute: bool) -> bool {
        let Some(attribute) = children(item)
            .into_iter()
            .find(|node| node.kind() == "attribute")
        else {
            return false;
        };
        let parts = children(attribute);
        let Some(path) = parts.first() else {
            return false;
        };
        let path: String = self
            .text(*path)
            .chars()
            .filter(|character| !character.is_whitespace())
            .collect();
        if path != "cfg" {
            return allow_test_attribute
                && (path.split("::").last() == Some("test")
                    || matches!(
                        path.split("::").next(),
                        Some("rstest" | "test_case" | "wasm_bindgen_test" | "quickcheck")
                    ));
        }
        let Some(arguments) = attribute.child_by_field_name("arguments") else {
            return false;
        };
        let predicates = groups(arguments);
        predicates.len() == 1
            && predicate(&predicates[0], self, false, 0) == Truth::No
            && predicate(&predicates[0], self, true, 0) != Truth::No
    }

    pub(super) fn rust_body_test_only(&self, body: Node<'_>) -> bool {
        for node in children(body) {
            match node.kind() {
                "inner_attribute_item" => {
                    if self.rust_attribute_test_only(node, false) {
                        return true;
                    }
                }
                _ if is_comment(node) => {}
                _ => return false,
            }
        }
        false
    }

    pub(super) fn rust_regions(&self, root: Node<'_>, output: &mut Vec<TestRegion>) {
        walk_regions(root, |siblings, index| {
            let node = siblings[index];
            if attribute_or_comment(node) {
                return false;
            }
            let mut first = index;
            let mut cursor = index;
            let mut test = false;
            while cursor > 0
                && attribute_or_comment(siblings[cursor - 1])
                && siblings[cursor - 1].kind() != "inner_attribute_item"
            {
                cursor -= 1;
                if siblings[cursor].kind() == "attribute_item" {
                    first = cursor;
                    test |= self.rust_attribute_test_only(siblings[cursor], true);
                }
            }
            test |= node.kind() == "mod_item"
                && node
                    .child_by_field_name("body")
                    .is_some_and(|body| self.rust_body_test_only(body));
            if test {
                output.push(self.region(siblings, first, index));
            }
            !test
        });
    }
}
