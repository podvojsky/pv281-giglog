CREATE TABLE IF NOT EXISTS "venue" (
    "id" SERIAL PRIMARY KEY,
    "name" TEXT NOT NULL,
    "description" TEXT,
    "state" TEXT NOT NULL,
    "postal_code" TEXT NOT NULL,
    "town" TEXT NOT NULL,
    "street_name" TEXT NOT NULL,
    "street_number" TEXT NOT NULL,
    "address_url" TEXT
);
