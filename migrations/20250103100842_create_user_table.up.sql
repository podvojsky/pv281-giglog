CREATE TYPE "gender_type" AS ENUM ('male', 'female', 'other');
CREATE TYPE "user_role" AS ENUM ('employee', 'organizer', 'admin');
CREATE TABLE IF NOT EXISTS "user" (
    "id" SERIAL PRIMARY KEY,
    "first_name" TEXT NOT NULL,
    "last_name" TEXT NOT NULL,
    "username" TEXT UNIQUE NOT NULL,
    "gender" "gender_type" NOT NULL,
    "birth_date" DATE NOT NULL,
    "email" TEXT UNIQUE NOT NULL,
    "phone" TEXT NOT NULL,
    "password_hash" TEXT NOT NULL,
    "role" "user_role" NOT NULL,
    "tax_rate" REAL NOT NULL,
    "avatar_url" TEXT
);
