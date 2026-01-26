//! Generic tests for the I18nDisplayable trait
//!
//! Tests that the trait provides consistent translation behavior
//! across different model types.

use ritmo_db::i18n_trait::I18nDisplayable;
use ritmo_db::i18n_utils::set_locale;
use ritmo_db::{Role, RunningLanguages};

/// Test that the trait translate() method works correctly for Role
#[test]
fn test_trait_translate_role() {
    set_locale("en");

    let role = Role {
        id: Some(1),
        key: "role.author".to_string(),
        created_at: 1234567890,
    };

    // Test trait method directly
    assert_eq!(role.translate(), "Author");

    // Test that display_name() delegates to trait
    assert_eq!(role.display_name(), role.translate());

    // Switch locale
    set_locale("it");
    assert_eq!(role.translate(), "Autore");
    assert_eq!(role.display_name(), role.translate());
}

/// Test that the trait translate() method works correctly for RunningLanguages
#[test]
fn test_trait_translate_running_languages() {
    set_locale("en");

    let lang = RunningLanguages {
        id: Some(1),
        name: "Italian".to_string(),
        role: "language_role.original".to_string(),
        iso_code_2char: Some("it".to_string()),
        iso_code_3char: Some("ita".to_string()),
        created_at: Some(1234567890),
        updated_at: Some(1234567890),
    };

    // Test trait method directly
    assert_eq!(lang.translate(), "Original Language");

    // Test that display_role() delegates to trait
    assert_eq!(lang.display_role(), lang.translate());

    // Switch locale
    set_locale("it");
    assert_eq!(lang.translate(), "Lingua Originale");
    assert_eq!(lang.display_role(), lang.translate());
}

/// Test trait behavior across multiple instances
#[test]
fn test_trait_multiple_instances() {
    set_locale("en");

    let roles = vec![
        Role {
            id: Some(1),
            key: "role.author".to_string(),
            created_at: 1234567890,
        },
        Role {
            id: Some(2),
            key: "role.translator".to_string(),
            created_at: 1234567890,
        },
        Role {
            id: Some(3),
            key: "role.editor".to_string(),
            created_at: 1234567890,
        },
    ];

    let expected_en = vec!["Author", "Translator", "Editor"];

    for (role, expected) in roles.iter().zip(expected_en.iter()) {
        assert_eq!(role.translate(), *expected);
    }

    // Switch to Italian
    set_locale("it");
    let expected_it = vec!["Autore", "Traduttore", "Editore"];

    for (role, expected) in roles.iter().zip(expected_it.iter()) {
        assert_eq!(role.translate(), *expected);
    }
}

/// Test that i18n_key() returns the expected key
#[test]
fn test_trait_i18n_key() {
    let role = Role {
        id: Some(1),
        key: "role.author".to_string(),
        created_at: 1234567890,
    };

    assert_eq!(role.i18n_key(), "role.author");

    let lang = RunningLanguages {
        id: Some(1),
        name: "English".to_string(),
        role: "language_role.original".to_string(),
        iso_code_2char: Some("en".to_string()),
        iso_code_3char: Some("eng".to_string()),
        created_at: Some(1234567890),
        updated_at: Some(1234567890),
    };

    assert_eq!(lang.i18n_key(), "language_role.original");
}

/// Test that i18n_namespace() returns the expected namespace
#[test]
fn test_trait_i18n_namespace() {
    let role = Role {
        id: Some(1),
        key: "role.author".to_string(),
        created_at: 1234567890,
    };

    // Default namespace is "db"
    assert_eq!(role.i18n_namespace(), "db");

    let lang = RunningLanguages {
        id: Some(1),
        name: "English".to_string(),
        role: "language_role.original".to_string(),
        iso_code_2char: Some("en".to_string()),
        iso_code_3char: Some("eng".to_string()),
        created_at: Some(1234567890),
        updated_at: Some(1234567890),
    };

    assert_eq!(lang.i18n_namespace(), "db");
}

/// Test generic function that accepts any I18nDisplayable
#[test]
fn test_trait_generic_function() {
    set_locale("en");

    // Generic function that works with any I18nDisplayable
    fn get_translation<T: I18nDisplayable>(item: &T) -> String {
        item.translate()
    }

    let role = Role {
        id: Some(1),
        key: "role.author".to_string(),
        created_at: 1234567890,
    };

    let lang = RunningLanguages {
        id: Some(1),
        name: "Italian".to_string(),
        role: "language_role.original".to_string(),
        iso_code_2char: Some("it".to_string()),
        iso_code_3char: Some("ita".to_string()),
        created_at: Some(1234567890),
        updated_at: Some(1234567890),
    };

    assert_eq!(get_translation(&role), "Author");
    assert_eq!(get_translation(&lang), "Original Language");
}

/// Test that trait handles missing translations gracefully
#[test]
fn test_trait_missing_translation() {
    set_locale("en");

    let role = Role {
        id: Some(1),
        key: "role.nonexistent".to_string(),
        created_at: 1234567890,
    };

    // When translation is missing, rust-i18n returns the key
    // The translate() method should handle this gracefully
    let result = role.translate();
    assert!(
        result.contains("role.nonexistent") || result.contains("db.role.nonexistent"),
        "Expected missing key in result, got: {}",
        result
    );
}
