CREATE TYPE "salary_currency" AS ENUM ('CZK', 'EUR');
CREATE TABLE IF NOT EXISTS "job_position" (
    "id" SERIAL PRIMARY KEY,
    "name" TEXT NOT NULL,
    "description" TEXT,
    "salary" REAL NOT NULL,
    "capacity" INT NOT NULL,
    "instructions_html" TEXT NOT NULL,
    "is_opened_for_registration" BOOLEAN NOT NULL,
    "currency" "salary_currency" NOT NULL,
    "event_id" INT NOT NULL REFERENCES "event" ("id") ON DELETE CASCADE,
    "position_category_id" INT NOT NULL REFERENCES "position_category" ("id") ON DELETE CASCADE
);
