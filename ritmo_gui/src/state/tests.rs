#[cfg(test)]
mod tests {
    use crate::state::{BooksFilterState, ContentsFilterState};
    use crate::events::{FilterField, FilterValue};
    use ritmo_db_core::filters::{BookFilters, ContentFilters};

    #[test]
    fn test_books_filter_state_creation() {
        let state = BooksFilterState::default();
        assert_eq!(state.get_filters().len(), 0);
    }

    #[test]
    fn test_add_book_filter() {
        let mut state = BooksFilterState::default();
        state.add_filter(FilterField::BookAuthor, FilterValue::Specific(vec!["Tolkien".to_string()]));
        
        assert_eq!(state.get_filters().len(), 1);
        assert_eq!(state.get_filters()[0].field, FilterField::BookAuthor);
    }

    #[test]
    fn test_remove_book_filter() {
        let mut state = BooksFilterState::default();
        state.add_filter(FilterField::BookAuthor, FilterValue::Specific(vec!["King".to_string()]));
        state.add_filter(FilterField::BookPublisher, FilterValue::Specific(vec!["Penguin".to_string()]));
        
        assert_eq!(state.get_filters().len(), 2);
        
        state.remove_filter(0);
        assert_eq!(state.get_filters().len(), 1);
        assert_eq!(state.get_filters()[0].field, FilterField::BookPublisher);
    }

    #[test]
    fn test_convert_to_book_filters() {
        let mut state = BooksFilterState::default();
        state.add_filter(FilterField::BookAuthor, FilterValue::Specific(vec!["Tolkien".to_string()]));
        state.add_filter(FilterField::BookFormat, FilterValue::Specific(vec!["epub".to_string()]));
        
        let book_filters = state.to_book_filters();
        assert_eq!(book_filters.authors.len(), 1);
        assert_eq!(book_filters.authors[0], "Tolkien");
        assert_eq!(book_filters.formats.len(), 1);
        assert_eq!(book_filters.formats[0], "epub");
    }

    #[test]
    fn test_convert_to_book_filters_or_within_row() {
        let mut state = BooksFilterState::default();
        state.add_filter(FilterField::BookAuthor, FilterValue::Specific(vec![
            "Tolkien".to_string(),
            "Asimov".to_string(),
        ]));
        
        let book_filters = state.to_book_filters();
        assert_eq!(book_filters.authors.len(), 2);
        assert!(book_filters.authors.contains(&"Tolkien".to_string()));
        assert!(book_filters.authors.contains(&"Asimov".to_string()));
    }

    #[test]
    fn test_convert_to_book_filters_and_between_rows() {
        let mut state = BooksFilterState::default();
        state.add_filter(FilterField::BookAuthor, FilterValue::Specific(vec!["Tolkien".to_string()]));
        state.add_filter(FilterField::BookFormat, FilterValue::Specific(vec!["epub".to_string()]));
        
        let book_filters = state.to_book_filters();
        // Different fields: AND semantics (both conditions present independently)
        assert_eq!(book_filters.authors, vec!["Tolkien"]);
        assert_eq!(book_filters.formats, vec!["epub"]);
    }

    #[test]
    fn test_contents_filter_state_creation() {
        let state = ContentsFilterState::default();
        assert_eq!(state.get_filters().len(), 0);
    }

    #[test]
    fn test_add_content_filter() {
        let mut state = ContentsFilterState::default();
        state.add_filter(FilterField::ContentAuthor, FilterValue::Specific(vec!["Asimov".to_string()]));
        
        assert_eq!(state.get_filters().len(), 1);
    }

    #[test]
    fn test_convert_to_content_filters() {
        let mut state = ContentsFilterState::default();
        state.add_filter(FilterField::ContentAuthor, FilterValue::Specific(vec!["Asimov".to_string()]));
        state.add_filter(FilterField::ContentType, FilterValue::Specific(vec!["novel".to_string()]));
        
        let content_filters = state.to_content_filters();
        assert_eq!(content_filters.authors.len(), 1);
        assert_eq!(content_filters.authors[0], "Asimov");
        assert_eq!(content_filters.content_types.len(), 1);
        assert_eq!(content_filters.content_types[0], "novel");
    }

    #[test]
    fn test_convert_to_content_filters_or_within_row() {
        let mut state = ContentsFilterState::default();
        state.add_filter(FilterField::ContentType, FilterValue::Specific(vec![
            "novel".to_string(),
            "short story".to_string(),
        ]));
        
        let content_filters = state.to_content_filters();
        assert_eq!(content_filters.content_types.len(), 2);
        assert!(content_filters.content_types.contains(&"novel".to_string()));
        assert!(content_filters.content_types.contains(&"short story".to_string()));
    }

    #[test]
    fn test_clear_filters() {
        let mut state = BooksFilterState::default();
        state.add_filter(FilterField::BookAuthor, FilterValue::Specific(vec!["King".to_string()]));
        state.add_filter(FilterField::BookPublisher, FilterValue::Specific(vec!["Penguin".to_string()]));
        
        assert_eq!(state.get_filters().len(), 2);
        
        state.clear();
        assert_eq!(state.get_filters().len(), 0);
    }

    #[test]
    fn test_filter_serialization() {
        let mut state = BooksFilterState::default();
        state.add_filter(FilterField::BookAuthor, FilterValue::Specific(vec!["Tolkien".to_string()]));
        state.add_filter(FilterField::BookFormat, FilterValue::AlmenoUno);
        state.add_filter(FilterField::BookPublisher, FilterValue::Nessuno);
        
        // Serialize
        let json = state.to_json().expect("Should serialize");
        
        // Deserialize
        let restored = BooksFilterState::from_json(&json);
        
        // Verify
        assert_eq!(restored.get_filters().len(), 3);
        assert_eq!(restored.get_filters()[0].field, FilterField::BookAuthor);
        assert_eq!(restored.get_filters()[1].field, FilterField::BookFormat);
        assert_eq!(restored.get_filters()[2].field, FilterField::BookPublisher);
        
        match &restored.get_filters()[0].value {
            FilterValue::Specific(v) => assert_eq!(v, &vec!["Tolkien".to_string()]),
            _ => panic!("Expected Specific value"),
        }
        
        match &restored.get_filters()[1].value {
            FilterValue::AlmenoUno => {},
            _ => panic!("Expected AlmenoUno value"),
        }
        
        match &restored.get_filters()[2].value {
            FilterValue::Nessuno => {},
            _ => panic!("Expected Nessuno value"),
        }
    }

    #[test]
    fn test_filter_serialization_multi_value() {
        let mut state = BooksFilterState::default();
        state.add_filter(FilterField::BookAuthor, FilterValue::Specific(vec![
            "Tolkien".to_string(),
            "Asimov".to_string(),
        ]));
        
        let json = state.to_json().expect("Should serialize");
        let restored = BooksFilterState::from_json(&json);
        
        match &restored.get_filters()[0].value {
            FilterValue::Specific(v) => {
                assert_eq!(v.len(), 2);
                assert!(v.contains(&"Tolkien".to_string()));
                assert!(v.contains(&"Asimov".to_string()));
            }
            _ => panic!("Expected Specific value"),
        }
    }

    #[test]
    fn test_filter_serialization_backward_compat() {
        // Old format: SPECIFIC:value (no pipe) should deserialize as single-element Vec
        let old_json = r#"[{"field":"BookAuthor","value":"SPECIFIC:Tolkien"}]"#;
        let restored = BooksFilterState::from_json(old_json);
        
        assert_eq!(restored.get_filters().len(), 1);
        match &restored.get_filters()[0].value {
            FilterValue::Specific(v) => {
                assert_eq!(v.len(), 1);
                assert_eq!(v[0], "Tolkien");
            }
            _ => panic!("Expected Specific value"),
        }
    }
}
