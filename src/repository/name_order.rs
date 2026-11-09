//! Shared locale-aware, numeric ordering for user-visible repository names.
use icu_collator::{
    options::{CollatorOptions, Strength},
    preferences::CollationNumericOrdering,
    Collator, CollatorBorrowed, CollatorPreferences,
};
use icu_locale_core::Locale;
use std::{cmp::Ordering, sync::LazyLock};

static FILE_NAMES: LazyLock<CollatorBorrowed<'static>> = LazyLock::new(|| {
    let locale = ["LC_ALL", "LC_COLLATE", "LANG"]
        .into_iter()
        .filter_map(|name| std::env::var(name).ok())
        .find(|value| !value.is_empty());
    collator(locale.as_deref())
});

fn collator(locale: Option<&str>) -> CollatorBorrowed<'static> {
    let locale = locale
        .unwrap_or("und")
        .split(['.', '@'])
        .next()
        .unwrap_or("und")
        .replace('_', "-")
        .parse::<Locale>()
        .unwrap_or(Locale::UNKNOWN);
    let mut preferences: CollatorPreferences = locale.into();
    preferences.numeric_ordering = Some(CollationNumericOrdering::True);
    let mut options = CollatorOptions::default();
    options.strength = Some(Strength::Secondary);
    Collator::try_new(preferences, options).expect("bundled filename collation data")
}

/// Compares display names, using original bytes to disambiguate equivalent names.
/// Sorting never changes the canonical path or the fuzzy search scoring tiers.
pub(crate) fn compare(left: &str, right: &str) -> Ordering {
    FILE_NAMES
        .compare(left, right)
        .then_with(|| left.cmp(right))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numeric_collation_handles_unicode_digits_long_numbers_and_equivalent_names() {
        for locale in [
            None,
            Some("en_US.UTF-8"),
            Some("C.UTF-8"),
            Some("not a locale"),
        ] {
            let order = collator(locale);
            for (left, right) in [
                ("file2.rs", "file10.rs"),
                ("café.rs", "Zulu.rs"),
                ("file٢.rs", "file١٠.rs"),
                ("file９.rs", "file１０.rs"),
                (
                    "file999999999999999999999999.rs",
                    "file1000000000000000000000000.rs",
                ),
            ] {
                assert_eq!(
                    order.compare(left, right),
                    Ordering::Less,
                    "{locale:?}: {left}/{right}"
                );
                assert_eq!(order.compare(right, left), Ordering::Greater);
            }
            assert_eq!(order.compare("a.rs", "A.rs"), Ordering::Equal);
            assert_eq!(order.compare("file2.rs", "file02.rs"), Ordering::Equal);
            assert_eq!(order.compare("café.rs", "cafe\u{301}.rs"), Ordering::Equal);
        }
        assert_ne!(compare("a.rs", "A.rs"), Ordering::Equal);
        assert_ne!(compare("file2.rs", "file02.rs"), Ordering::Equal);
        assert_eq!(compare("same.rs", "same.rs"), Ordering::Equal);
    }

    #[test]
    fn locale_tailoring_is_applied_to_the_collator() {
        assert_eq!(
            collator(Some("de_DE.UTF-8")).compare("ä.rs", "z.rs"),
            Ordering::Less
        );
        assert_eq!(
            collator(Some("sv_SE.UTF-8")).compare("ä.rs", "z.rs"),
            Ordering::Greater
        );
    }
}
