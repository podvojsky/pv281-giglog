CREATE TABLE IF NOT EXISTS "event_manager_relation" (
    "user_id" INT NOT NULL REFERENCES "user"("id") ON DELETE CASCADE,
    "event_id" INT NOT NULL REFERENCES "event"("id") ON DELETE CASCADE,
    CONSTRAINT "unique_user_event" UNIQUE ("user_id", "event_id")
);
