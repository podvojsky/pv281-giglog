INSERT INTO "position_category" ("id", "name") VALUES
    (1, 'Bezpečnost a kontrola'),
    (2, 'Organizace a logistika'),
    (3, 'Obsluha a zákaznický servis'),
    (4, 'Úklid'),
    (5, 'Zdravotní služby'),
    (6, 'Občerstvení a prodej'),
    (7, 'Technická podpora a údržba');

SELECT setval(pg_get_serial_sequence('"position_category"', 'id'), MAX("id")) FROM "position_category";
