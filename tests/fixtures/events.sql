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
(1,'Josefka','Buba','female','4/11/2001','pepe232','joko@nba.com','7151703730','admin',0.15,'$argon2d$v=19$m=12,t=3,p=1$ZXk0ajEzc2k2Zm0wMDAwMA$cMhkzibYVqUSgrkoZ9BP8w');
SELECT setval(pg_get_serial_sequence('"user"', 'id'), MAX("id")) FROM "user";

INSERT INTO "venue" ("id", "name", "state", "postal_code", "town", "street_name", "street_number", "description") VALUES
(1, 'Amfiteátr Jihlava', 'Česká republika','586 01','Jihlava','Březinovy sady','4733/16', 'V malebném údolí řeky Jihlavy v těsném sousedství ze zologickou zahradou přímo v centru Jihlavy se nachází areál Amfiteátru a parku Malý Heulos, který je místem odpočinku, relaxace a zábavy pro celou rodinu. Areál je ideální pro pořádání venkovních kulturních a společenských akcí. Zázemí pod plátnem umožňuje pořádat celoročně klubovou a schůzovní činnost. Součástí areálu je moderní dětské hřiště se spoustou atrakcí.'),
(2, 'Plzeň Plaza','Česká republika','301 00','Plzeň','Radčická','3', 'mfiteátr Plzeň Plaza je venkovní multifunkční prostor v Plzni, který se nachází v blízkosti nákupního centra Plaza. Tento amfiteátr slouží jako místo pro pořádání různorodých kulturních a společenských akcí, jako jsou koncerty, divadelní představení, letní kina, festivaly nebo firemní akce.'),
(3, 'Berlin Arena', 'Germany', '10243', 'Berlin', 'Eichenstraße', '4', 'Berlin Arena je moderní multifunkční prostor v Berlíně, který hostí koncerty, sportovní události, firemní akce a festivaly. Nachází se v blízkosti řeky Sprévy a nabízí flexibilní prostor pro různé typy akcí.');
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
),
(
    2,
    'Rocky in 2025',
    '5/25/2025',
    '5/25/2025',
    'https://cdn.siteone.io/srv.siteone.cz/imgproxy/LhCs_LjiIr027zmUTqCgc-JgrB4-Dx33eI3QWyD0xoI/w:860/h:740/rt:fill/g:no:0:0/f:avif/q:70/aHR0cHM6Ly93d3cubmVrZGVuZWNvLmN6Ly9jbXMtYXNzZXRzL3JvY2staW4tMjAyNV8yMDI0LTExLTA1LTA3MzAwNl9vc3pxLmpwZw.avif',
    'Zažijte to nejlepší z domácí rockové scény v jeden den na jednom pódiu přímo u vás! Ve vašem městě, ve vašem amfiteátru se vystřídají zvučná jména, s důrazem na profesionální zázemí, špičkovou techniku a maximální komfort pro návštěvníky.',
    False,
    1,
    1
),
(
    3,
    'Feets for love',
    '7/2/2025',
    '7/5/2025',
    'https://i3.cn.cz/1720345702_PULKRABEK_JAN-240707-004325-_BMD3310_UVODNI.JPG',
    'Beats for Love je festival elektronické taneční hudby pořádaný v jádru industriální národní kulturní památky. Prostředí Dolních Vítkovic plné železných kulis se na čtyři dny zaplní spoustou vynikající hudby a bohatého doprovodného programu. Vzniká tak jedinečná atmosféra plná zábavy a zážitků.',
    False,
    2,
    1
),
(
    4,
    'Berlin Music Fest',
    '8/15/2025',
    '8/18/2025',
    'https://example.com/berlin-music-fest.jpg',
    'Berlin Music Fest je jedinečný hudební festival konaný v Berlin Areně, který spojuje různé žánry hudby, od elektroniky po rock. Nabízí nezapomenutelnou atmosféru s vystoupeními světově známých umělců, skvělým jídlem a doprovodnými aktivitami.',
    True,
    3,
    1
);

SELECT setval(pg_get_serial_sequence('"event"', 'id'), MAX("id")) FROM "event";
