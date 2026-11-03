use std::ops::RangeInclusive;
use toml_edit::{Table, Value};

pub(super) struct Root<'a> {
    table: &'a Table,
    text: &'a str,
}

impl<'a> Root<'a> {
    pub(super) fn new(table: &'a Table, text: &'a str) -> Result<Self, String> {
        const ALLOWED: &[&str] = &[
            "schema_version",
            "id",
            "name",
            "summary",
            "engine",
            "indexers",
            "project_limit",
            "clang_jobs",
            "bootstrap_python",
        ];
        for (key, item) in table.iter() {
            if item.as_table().is_some_and(Table::is_dotted) {
                return Err("Dotted personality root settings are unsupported".into());
            }
            if item.is_table() || item.is_array_of_tables() {
                continue;
            }
            let spelling = table
                .key(key)
                .and_then(|key| key.span())
                .map(|span| &text[span]);
            if !ALLOWED.contains(&key) || spelling != Some(key) {
                return Err(format!(
                    "Unknown or unsupported personality root setting: {key}"
                ));
            }
        }
        Ok(Self { table, text })
    }

    pub(super) fn validate_identity(&self, personality: super::Personality) -> Result<(), String> {
        self.integer("schema_version", 1..=1)?;
        if self.string("id")? != personality.id() {
            return Err("Invalid personality identity".into());
        }
        let engine = if personality == super::Personality::Default {
            "automatic"
        } else {
            "sel4"
        };
        if self.string("engine")? != engine {
            return Err("Personality engine does not match the selected pipeline".into());
        }
        Ok(())
    }

    pub(super) fn bootstrap_python(&self) -> Result<Vec<String>, String> {
        if self.table.contains_key("bootstrap_python") {
            self.strings("bootstrap_python")
        } else {
            Ok([
                "python3.11",
                "python3.12",
                "python3.13",
                "python3.14",
                "python3",
            ]
            .map(str::to_owned)
            .into())
        }
    }

    fn value(&self, key: &str) -> Result<&Value, String> {
        self.table
            .get(key)
            .and_then(toml_edit::Item::as_value)
            .ok_or_else(|| format!("Missing personality setting: {key}"))
    }

    fn single_line_string(&self, value: &Value) -> Result<String, String> {
        let text = value
            .span()
            .map(|span| &self.text[span])
            .unwrap_or_default();
        let string = value.as_str().filter(|_| {
            !text.starts_with("\"\"\"") && !text.starts_with("'''") && !text.contains(['\n', '\r'])
        });
        string
            .map(str::to_owned)
            .ok_or_else(|| "Expected a single-line quoted personality string".into())
    }

    pub(super) fn string(&self, key: &str) -> Result<String, String> {
        self.single_line_string(self.value(key)?)
    }

    pub(super) fn strings(&self, key: &str) -> Result<Vec<String>, String> {
        self.value(key)?
            .as_array()
            .ok_or_else(|| format!("Expected a string array: {key}"))?
            .iter()
            .map(|value| self.single_line_string(value))
            .collect()
    }

    pub(super) fn integer(&self, key: &str, range: RangeInclusive<usize>) -> Result<usize, String> {
        let value = self.value(key)?;
        let integer = value
            .span()
            .and_then(|span| self.text[span].parse::<usize>().ok());
        integer.filter(|n| range.contains(n)).ok_or_else(|| {
            format!(
                "Expected decimal {key} in {}–{}",
                range.start(),
                range.end()
            )
        })
    }
}
