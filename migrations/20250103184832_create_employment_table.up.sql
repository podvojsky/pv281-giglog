CREATE TYPE "employment_state" AS ENUM ('pending', 'accepted', 'rejected', 'done');
CREATE TABLE IF NOT EXISTS "employment" (
    "id" SERIAL PRIMARY KEY,
    "rating" INT NOT NULL,
    "state" "employment_state" NOT NULL,
    "user_id" INT NOT NULL REFERENCES "user"("id") ON DELETE CASCADE,
    "position_id" INT NOT NULL REFERENCES "job_position"("id") ON DELETE CASCADE
);
