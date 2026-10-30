//! Kotlin annotation and test-class recognition against the native grammar.
use super::*;

const FUNCTION_ANNOTATIONS: &[&str] = &[
    "Test",
    "ParameterizedTest",
    "RepeatedTest",
    "TestFactory",
    "TestTemplate",
    "BeforeTest",
    "AfterTest",
    "BeforeEach",
    "AfterEach",
    "BeforeAll",
    "AfterAll",
];
const CLASS_ANNOTATIONS: &[&str] = &["RunWith", "ExtendWith", "TestInstance"];
const SPEC_BASES: &[&str] = &[
    "StringSpec",
    "FunSpec",
    "DescribeSpec",
    "BehaviorSpec",
    "ShouldSpec",
    "WordSpec",
    "FreeSpec",
    "FeatureSpec",
    "ExpectSpec",
    "AnnotationSpec",
];

impl Detector<'_> {
    pub(super) fn kotlin_regions(&self, root: Node<'_>, output: &mut Vec<TestRegion>) {
        walk_regions(root, self.control, |siblings, index| {
            let node = siblings[index];
            let test = match node.kind() {
                "class_declaration" | "object_declaration" => self.kotlin_test_class(node),
                "function_declaration" => self.kotlin_annotation(node, FUNCTION_ANNOTATIONS),
                _ => false,
            };
            if test {
                output.push(self.region(siblings, index, index));
            }
            !test
        });
    }

    fn kotlin_annotation(&self, declaration: Node<'_>, names: &[&str]) -> bool {
        children(declaration)
            .into_iter()
            .filter(|node| node.kind() == "modifiers")
            .flat_map(children)
            .filter(|node| node.kind() == "annotation")
            .any(|annotation| {
                let text = self.text(annotation).trim_start_matches('@');
                let path = text.split(['(', ' ', '\n', '\r']).next().unwrap_or("");
                names.contains(&path.rsplit('.').next().unwrap_or(""))
            })
    }

    fn kotlin_test_class(&self, node: Node<'_>) -> bool {
        if self.kotlin_annotation(node, CLASS_ANNOTATIONS) {
            return true;
        }
        for child in children(node) {
            if self.control.is_cancelled() {
                break;
            }
            match child.kind() {
                "delegation_specifiers" => {
                    let mut pending = vec![child];
                    while let Some(node) = pending.pop() {
                        if self.control.is_cancelled() {
                            break;
                        }
                        if node.kind() == "user_type" {
                            let name = children(node)
                                .into_iter()
                                .rfind(|node| node.kind() == "identifier");
                            if name.is_some_and(|name| SPEC_BASES.contains(&self.text(name))) {
                                return true;
                            }
                        } else {
                            pending.extend(children(node));
                        }
                    }
                }
                "class_body" => {
                    for member in children(child) {
                        if self.control.is_cancelled() {
                            break;
                        }
                        let declaration = if member.kind() == "class_member_declaration" {
                            children(member).first().copied().unwrap_or(member)
                        } else {
                            member
                        };
                        if declaration.kind() == "function_declaration"
                            && self.kotlin_annotation(declaration, FUNCTION_ANNOTATIONS)
                        {
                            return true;
                        }
                    }
                }
                _ => {}
            }
        }
        false
    }
}
