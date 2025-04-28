CREATE TABLE IF NOT EXISTS "event" (
    "id" SERIAL PRIMARY KEY,
    "name" TEXT NOT NULL,
    "date_start" DATE NOT NULL,
    "date_end" DATE NOT NULL,
    "img_url" TEXT NOT NULL,
    "description" TEXT,
    "is_draft" BOOLEAN NOT NULL,
    "venue_id" INT NOT NULL REFERENCES "venue" ("id"),
    "owner_id" INT NOT NULL REFERENCES "user" ("id") ON DELETE CASCADE
);
