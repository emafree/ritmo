//! Integration tests for i18n functionality in Type and Format models
//!
//! Tests that Type and Format models correctly translate
//! their display names based on the current locale.

use ritmo_db::i18n_trait::I18nDisplayable;
use ritmo_db::i18n_utils::set_locale;
use ritmo_db::{Format, Type};

// ============================================================================
// Type Model Tests
// ============================================================================

#[test]
fn test_type_display_name_english() {
    set_locale("en");

    let type_record = Type {
        id: Some(1),
        key: "type.novel".to_string(),
        description: None,
        created_at: 1234567890,
    };

    assert_eq!(type_record.display_name(), "Novel");
}

#[test]
fn test_type_display_name_italian() {
    set_locale("it");

    let type_record = Type {
        id: Some(1),
        key: "type.novel".to_string(),
        description: None,
        created_at: 1234567890,
    };

    assert_eq!(type_record.display_name(), "Romanzo");
}

#[test]
fn test_all_type_translations() {
    set_locale("en");

    let types = vec![
        ("type.novel", "Novel"),
        ("type.short_story", "Short Story"),
        ("type.essay", "Essay"),
        ("type.poetry", "Poetry"),
        ("type.article", "Article"),
    ];

    for (key, expected_en) in types.iter() {
        let type_record = Type {
            id: Some(1),
            key: key.to_string(),
            description: None,
            created_at: 1234567890,
        };
        assert_eq!(type_record.display_name(), *expected_en);
    }

    // Switch to Italian
    set_locale("it");

    let types_it = vec![
        ("type.novel", "Romanzo"),
        ("type.short_story", "Racconto"),
        ("type.essay", "Saggio"),
        ("type.poetry", "Poesia"),
        ("type.article", "Articolo"),
    ];

    for (key, expected_it) in types_it.iter() {
        let type_record = Type {
            id: Some(1),
            key: key.to_string(),
            description: None,
            created_at: 1234567890,
        };
        assert_eq!(type_record.display_name(), *expected_it);
    }
}

#[test]
fn test_type_translate_method() {
    set_locale("en");

    let type_record = Type {
        id: Some(1),
        key: "type.novel".to_string(),
        description: None,
        created_at: 1234567890,
    };

    // Test trait method directly
    assert_eq!(type_record.translate(), "Novel");

    // Test that display_name() delegates to trait
    assert_eq!(type_record.display_name(), type_record.translate());

    // Switch locale
    set_locale("it");
    assert_eq!(type_record.translate(), "Romanzo");
    assert_eq!(type_record.display_name(), type_record.translate());
}

#[test]
fn test_type_i18n_key() {
    let type_record = Type {
        id: Some(1),
        key: "type.novel".to_string(),
        description: None,
        created_at: 1234567890,
    };

    assert_eq!(type_record.i18n_key(), "type.novel");
}

// ============================================================================
// Format Model Tests
// ============================================================================

#[test]
fn test_format_display_name_english() {
    set_locale("en");

    let format = Format {
        id: Some(1),
        key: "format.epub".to_string(),
        description: None,
        created_at: 1234567890,
    };

    assert_eq!(format.display_name(), "EPUB (ebook)");
}

#[test]
fn test_format_display_name_italian() {
    set_locale("it");

    let format = Format {
        id: Some(1),
        key: "format.epub".to_string(),
        description: None,
        created_at: 1234567890,
    };

    assert_eq!(format.display_name(), "EPUB (ebook)");
}

#[test]
fn test_all_format_translations() {
    set_locale("en");

    let formats = vec![
        ("format.epub", "EPUB (ebook)"),
        ("format.pdf", "PDF Document"),
        ("format.mobi", "MOBI (Kindle)"),
        ("format.azw3", "AZW3 (Kindle)"),
        ("format.txt", "Text File"),
    ];

    for (key, expected_en) in formats.iter() {
        let format = Format {
            id: Some(1),
            key: key.to_string(),
            description: None,
            created_at: 1234567890,
        };
        assert_eq!(format.display_name(), *expected_en);
    }

    // Switch to Italian
    set_locale("it");

    let formats_it = vec![
        ("format.epub", "EPUB (ebook)"),
        ("format.pdf", "Documento PDF"),
        ("format.mobi", "MOBI (Kindle)"),
        ("format.azw3", "AZW3 (Kindle)"),
        ("format.txt", "File di Testo"),
    ];

    for (key, expected_it) in formats_it.iter() {
        let format = Format {
            id: Some(1),
            key: key.to_string(),
            description: None,
            created_at: 1234567890,
        };
        assert_eq!(format.display_name(), *expected_it);
    }
}

#[test]
fn test_format_translate_method() {
    set_locale("en");

    let format = Format {
        id: Some(1),
        key: "format.pdf".to_string(),
        description: None,
        created_at: 1234567890,
    };

    // Test trait method directly
    assert_eq!(format.translate(), "PDF Document");

    // Test that display_name() delegates to trait
    assert_eq!(format.display_name(), format.translate());

    // Switch locale
    set_locale("it");
    assert_eq!(format.translate(), "Documento PDF");
    assert_eq!(format.display_name(), format.translate());
}

#[test]
fn test_format_i18n_key() {
    let format = Format {
        id: Some(1),
        key: "format.epub".to_string(),
        description: None,
        created_at: 1234567890,
    };

    assert_eq!(format.i18n_key(), "format.epub");
}

// ============================================================================
// Generic Trait Tests
// ============================================================================

#[test]
fn test_generic_function_with_type_and_format() {
    set_locale("en");

    // Generic function that works with any I18nDisplayable
    fn get_translation<T: I18nDisplayable>(item: &T) -> String {
        item.translate()
    }

    let type_record = Type {
        id: Some(1),
        key: "type.novel".to_string(),
        description: None,
        created_at: 1234567890,
    };

    let format = Format {
        id: Some(1),
        key: "format.epub".to_string(),
        description: None,
        created_at: 1234567890,
    };

    assert_eq!(get_translation(&type_record), "Novel");
    assert_eq!(get_translation(&format), "EPUB (ebook)");
}

#[test]
fn test_type_format_locale_switching() {
    // Start with English
    set_locale("en");

    let type_record = Type {
        id: Some(1),
        key: "type.novel".to_string(),
        description: None,
        created_at: 1234567890,
    };

    let format = Format {
        id: Some(1),
        key: "format.pdf".to_string(),
        description: None,
        created_at: 1234567890,
    };

    assert_eq!(type_record.display_name(), "Novel");
    assert_eq!(format.display_name(), "PDF Document");

    // Switch to Italian
    set_locale("it");
    assert_eq!(type_record.display_name(), "Romanzo");
    assert_eq!(format.display_name(), "Documento PDF");

    // Switch back to English
    set_locale("en");
    assert_eq!(type_record.display_name(), "Novel");
    assert_eq!(format.display_name(), "PDF Document");
}
