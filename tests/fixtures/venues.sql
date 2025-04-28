DELETE FROM "event_manager_relation";
DELETE FROM "worked_hours";
DELETE FROM "employment";
DELETE FROM "job_position";
DELETE FROM "event";
DELETE FROM "position_category";
DELETE FROM "user";
DELETE FROM "venue";

INSERT INTO "venue" ("id", "name", "state", "postal_code", "town", "street_name", "street_number", "description")
VALUES
    (1, 'Plzeň Plaza','Česká republika','301 00','Plzeň','Radčická','3', 'mfiteátr Plzeň Plaza je venkovní multifunkční prostor v Plzni, který se nachází v blízkosti nákupního centra Plaza. Tento amfiteátr slouží jako místo pro pořádání různorodých kulturních a společenských akcí, jako jsou koncerty, divadelní představení, letní kina, festivaly nebo firemní akce.'),
    (2, 'Brněnské výstaviště', 'Česká republika','603 00','Brno','Výstaviště','405/1', 'Brněnské výstaviště je rozsáhlý areál v Brně, známý pořádáním mezinárodních veletrhů, výstav a kulturních akcí. Bylo otevřeno v roce 1928 a zahrnuje unikátní funkcionalistickou architekturu. Nabízí moderní pavilony, velkorysé výstavní plochy a výbornou dostupnost, čímž patří k nejvýznamnějším výstavním centrům v Evropě.'),
    (3, 'Letiště Hradec Králové', 'Česká republika','503 41','Hradec Králové','Letiště','38', 'Letiště Hradec Králové je známé jako dějiště významných kulturních a společenských akcí. Pravidelně hostí festival Rock for People, letecké dny a další velké venkovní akce. Díky svým rozlehlým plochám a dobré dostupnosti je ideálním místem pro pořádání hudebních a společenských událostí.'),
    (4, 'Katedrála', 'Česká republika', '301 00', 'Plzeň', 'Neew', '33', Null);

SELECT setval(pg_get_serial_sequence('"venue"', 'id'), MAX("id")) FROM "venue";
