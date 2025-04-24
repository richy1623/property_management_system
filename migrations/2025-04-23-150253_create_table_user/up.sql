-- Your SQL goes here
CREATE TABLE
    "user" (
        user_id SERIAL PRIMARY KEY,
        user_name TEXT NOT NULL,
        name TEXT NOT NULL,
        cellphone_number TEXT,
        email_address TEXT,
        nrc TEXT,
        password_hash TEXT NOT NULL
    );

-- TODO username unique
