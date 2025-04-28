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
