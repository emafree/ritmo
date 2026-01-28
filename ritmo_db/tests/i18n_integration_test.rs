//! Integration tests for i18n functionality
//!
//! Tests that Role and RunningLanguages models correctly translate
//! their display names based on the current locale.

use ritmo_db::i18n_utils::set_locale;
use ritmo_db::{Role, RunningLanguages};
use serial_test::serial;

#[test]
#[serial]
fn test_role_display_name_english() {
    // Force reset to English (tests may run in any order)
    set_locale("en");

    // Create a role with key "role.author"
    let role = Role {
        id: Some(1),
        key: "role.author".to_string(),
        created_at: 1234567890,
    };

    // Should return "Author" in English
    assert_eq!(role.display_name(), "Author");
}

#[test]
#[serial]
fn test_role_display_name_italian() {
    // Force reset to Italian
    set_locale("it");

    // Create a role with key "role.author"
    let role = Role {
        id: Some(1),
        key: "role.author".to_string(),
        created_at: 1234567890,
    };

    // Should return "Autore" in Italian
    assert_eq!(role.display_name(), "Autore");
}

#[test]
#[serial]
fn test_all_role_translations() {
    // Start fresh with English
    set_locale("en");

    let roles = vec![
        ("role.author", "Author"),
        ("role.editor", "Editor"),
        ("role.translator", "Translator"),
        ("role.illustrator", "Illustrator"),
        ("role.contributor", "Contributor"),
        ("role.narrator", "Narrator"),
    ];

    for (key, expected_en) in roles.iter() {
        let role = Role {
            id: Some(1),
            key: key.to_string(),
            created_at: 1234567890,
        };
        assert_eq!(role.display_name(), *expected_en);
    }

    // Switch to Italian
    set_locale("it");

    let roles_it = vec![
        ("role.author", "Autore"),
        ("role.editor", "Editore"),
        ("role.translator", "Traduttore"),
        ("role.illustrator", "Illustratore"),
        ("role.contributor", "Collaboratore"),
        ("role.narrator", "Narratore"),
    ];

    for (key, expected_it) in roles_it.iter() {
        let role = Role {
            id: Some(1),
            key: key.to_string(),
            created_at: 1234567890,
        };
        assert_eq!(role.display_name(), *expected_it);
    }
}

#[test]
#[serial]
fn test_language_role_display_english() {
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

    assert_eq!(lang.display_role(), "Original Language");
}

#[test]
#[serial]
fn test_language_role_display_italian() {
    set_locale("it");

    let lang = RunningLanguages {
        id: Some(1),
        name: "Italian".to_string(),
        role: "language_role.original".to_string(),
        iso_code_2char: Some("it".to_string()),
        iso_code_3char: Some("ita".to_string()),
        created_at: Some(1234567890),
        updated_at: Some(1234567890),
    };

    assert_eq!(lang.display_role(), "Lingua Originale");
}

#[test]
#[serial]
fn test_all_language_role_translations() {
    set_locale("en");

    let language_roles = vec![
        ("language_role.original", "Original Language"),
        ("language_role.source", "Source Language"),
        ("language_role.actual", "Translation Language"),
    ];

    for (key, expected_en) in language_roles.iter() {
        let lang = RunningLanguages {
            id: Some(1),
            name: "Test".to_string(),
            role: key.to_string(),
            iso_code_2char: Some("xx".to_string()),
            iso_code_3char: Some("xxx".to_string()),
            created_at: Some(1234567890),
            updated_at: Some(1234567890),
        };
        assert_eq!(lang.display_role(), *expected_en);
    }

    // Switch to Italian
    set_locale("it");

    let language_roles_it = vec![
        ("language_role.original", "Lingua Originale"),
        ("language_role.source", "Lingua di Partenza"),
        ("language_role.actual", "Lingua di Traduzione"),
    ];

    for (key, expected_it) in language_roles_it.iter() {
        let lang = RunningLanguages {
            id: Some(1),
            name: "Test".to_string(),
            role: key.to_string(),
            iso_code_2char: Some("xx".to_string()),
            iso_code_3char: Some("xxx".to_string()),
            created_at: Some(1234567890),
            updated_at: Some(1234567890),
        };
        assert_eq!(lang.display_role(), *expected_it);
    }
}

#[test]
#[serial]
fn test_locale_switching() {
    // Start with English
    set_locale("en");

    let role = Role {
        id: Some(1),
        key: "role.author".to_string(),
        created_at: 1234567890,
    };

    assert_eq!(role.display_name(), "Author");

    // Switch to Italian
    set_locale("it");
    assert_eq!(role.display_name(), "Autore");

    // Switch back to English
    set_locale("en");
    assert_eq!(role.display_name(), "Author");
}
