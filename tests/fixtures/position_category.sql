DELETE FROM "event_manager_relation";
DELETE FROM "worked_hours";
DELETE FROM "employment";
DELETE FROM "job_position";
DELETE FROM "event";
DELETE FROM "position_category";
DELETE FROM "user";
DELETE FROM "venue";

INSERT INTO "position_category" ("id", "name") VALUES
    (1, 'Obsluha stánků'),
    (2, 'Koordinace parkoviště'),
    (3, 'Technická podpora');

SELECT setval(pg_get_serial_sequence('"position_category"', 'id'), MAX("id")) FROM "position_category";
