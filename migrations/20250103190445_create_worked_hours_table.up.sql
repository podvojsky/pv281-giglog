CREATE TABLE IF NOT EXISTS "worked_hours" (
    "id" SERIAL PRIMARY KEY,
    "date" DATE NOT NULL,
    "hours_worked" REAL NOT NULL,
    "employment_id" INT NOT NULL REFERENCES "employment"("id") ON DELETE CASCADE
);
