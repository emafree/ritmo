-- Seed della tabella page_fields.
-- Eseguito dopo la creazione dello schema.
-- DELETE + INSERT garantisce uno stato sempre deterministico e aggiornabile.

DELETE FROM page_fields;

INSERT INTO page_fields (page, field_key, data_kind, sort_order, relation_type, target_table, target_field) VALUES
    -- book_page
    ('book_page', 'field-original-title',   'string', 10, 'direct',   NULL,             NULL),
    ('book_page', 'field-publication-date', 'date',   20, 'direct',   NULL,             NULL),
    ('book_page', 'field-isbn',             'string', 30, 'direct',   NULL,             NULL),
    ('book_page', 'field-publisher',        'string', 40, 'fk',       'publishers',     'publisher_id'),
    ('book_page', 'field-notes',            'string', 50, 'direct',   NULL,             NULL),
    ('book_page', 'field-tags',             'string', 55, 'junction', 'x_books_tags',   NULL),
    ('book_page', 'field-people',           'person', 60, 'junction', 'x_books_people', NULL),

    -- content_page
    ('content_page', 'field-title',    'string', 10, 'direct',   NULL,             NULL),
    ('content_page', 'field-author',   'person', 20, 'junction', 'x_books_people', NULL),
    ('content_page', 'field-language', 'enum',   30, 'direct',   NULL,             NULL),
    ('content_page', 'field-people',   'person', 40, 'junction', 'x_books_people', NULL),

    -- people_page
    ('people_page', 'field-display-name', 'string', 10, 'direct', NULL, NULL),
    ('people_page', 'field-nationality',  'string', 20, 'direct', NULL, NULL),
    ('people_page', 'field-birth-date',   'date',   30, 'direct', NULL, NULL),
    ('people_page', 'field-death-date',   'date',   40, 'direct', NULL, NULL);
