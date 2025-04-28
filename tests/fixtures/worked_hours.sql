DELETE FROM "event_manager_relation";
DELETE FROM "worked_hours";
DELETE FROM "employment";
DELETE FROM "job_position";
DELETE FROM "event";
DELETE FROM "position_category";
DELETE FROM "user";
DELETE FROM "venue";


INSERT INTO "user"
("id", "first_name", "last_name", "gender", "birth_date", "username", "email", "phone", "role", "tax_rate", "password_hash")
VALUES
    (1,'Josefka','Buba','female','4/11/2001','pepe232','joko@nba.com','7151703730','admin',0.15,'$argon2d$v=19$m=12,t=3,p=1$ZXk0ajEzc2k2Zm0wMDAwMA$cMhkzibYVqUSgrkoZ9BP8w'),
    (2,'Radek','Srejch','male','5/12/2000','brember','brember@mail.com','2212605075','employee',0.15,'$argon2d$v=19$m=12,t=3,p=1$ZXk0ajEzc2k2Zm0wMDAwMA$cMhkzibYVqUSgrkoZ9BP8w'),
    (3,'Lukáš','Nadvojský','female','7/31/2001','lasicak','lasicak@mail.com','6284663453','admin',0.15,'$argon2d$v=19$m=12,t=3,p=1$ZXk0ajEzc2k2Zm0wMDAwMA$cMhkzibYVqUSgrkoZ9BP8w'),
    (4,'Michal','Uherácký','male','2/25/2003','fousek','michuh@mail.com','8302944138','organizer',0.15,'$argon2d$v=19$m=12,t=3,p=1$ZXk0ajEzc2k2Zm0wMDAwMA$cMhkzibYVqUSgrkoZ9BP8w');

SELECT setval(pg_get_serial_sequence('"user"', 'id'), MAX("id")) FROM "user";

INSERT INTO "venue" ("id", "name", "state", "postal_code", "town", "street_name", "street_number", "description") VALUES
    (1, 'Amfiteátr Jihlava', 'Česká republika','586 01','Jihlava','Březinovy sady','4733/16', 'V malebném údolí řeky Jihlavy v těsném sousedství ze zologickou zahradou přímo v centru Jihlavy se nachází areál Amfiteátru a parku Malý Heulos, který je místem odpočinku, relaxace a zábavy pro celou rodinu. Areál je ideální pro pořádání venkovních kulturních a společenských akcí. Zázemí pod plátnem umožňuje pořádat celoročně klubovou a schůzovní činnost. Součástí areálu je moderní dětské hřiště se spoustou atrakcí.');
SELECT setval(pg_get_serial_sequence('"venue"', 'id'), MAX("id")) FROM "venue";

INSERT INTO "event" (
    "id",
    "name",
    "date_start",
    "date_end",
    "img_url",
    "description",
    "is_draft",
    "venue_id",
    "owner_id"
)
VALUES
    (
        1,
        'HIMLHERGOTFEST',
        '1/1/2025',
        '1/2/2025',
        'https://ticketstream-images.s3.eu-central-1.amazonaws.com/event/2024/08/azeib1c6b6_himlhergotfest2025-1080x1080.png',
        'Echtšlágrgruppe TRAUTENBERK slibuje playlist plný osvědčených pecek i nových songů, koncert plný nasazení a potu. Aničku svůdnější víc, než kdy jindy, Zemského radu přísnějšího, než kdy jindy a zbytek kapely hlasitější, než kdy jindy! V Čechách stále stoupající hvězda a festivalová stálice nenechá tvoji taneční kyčel v klidu a vykouzlí Ti dlouhý úsměv na tváři. Tak doraž na koncert!',
        False,
        1,
        1
    );

SELECT setval(pg_get_serial_sequence('"event"', 'id'), MAX("id")) FROM "event";

INSERT INTO "position_category" ("id", "name") VALUES (1, 'Technická podpora');

SELECT setval(pg_get_serial_sequence('"position_category"', 'id'), MAX("id")) FROM "position_category";

INSERT INTO "job_position" (
    "id", "event_id", "position_category_id", "salary", "currency", "capacity", "name", "description", "is_opened_for_registration", "instructions_html")
VALUES
    (1,1, 1, 150, 'CZK', 5, 'Stánek s hotdogy', 'Prodej hotdogů a dalších rychlých jídel návštěvníkům', True, 'Co vznikne zkřížením komára a mouchy? - Komouš.');

SELECT setval(pg_get_serial_sequence('"job_position"', 'id'), MAX("id")) FROM "job_position";

INSERT INTO "employment" ("id", "user_id", "position_id", "rating", "state") VALUES
    (1, 1, 1, 8, 'pending'),
    (2, 2, 1, 4, 'rejected'),
    (3, 3, 1, 7, 'accepted'),
    (4, 4, 1, 7, 'accepted');

SELECT setval(pg_get_serial_sequence('"employment"', 'id'), MAX("id")) FROM "employment";

INSERT INTO "worked_hours" ("id", "employment_id", "hours_worked", "date") VALUES
    (1, 1, 5.1, '1/2/2025'),
    (2, 2, 7.5, '1/2/2025'),
    (3, 3, 9.2, '1/2/2025');

SELECT setval(pg_get_serial_sequence('"worked_hours"', 'id'), MAX("id")) FROM "worked_hours";