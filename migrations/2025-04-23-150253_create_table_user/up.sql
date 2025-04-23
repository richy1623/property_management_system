-- Your SQL goes here
CREATE TABLE
    "user" (
        user_id SERIAL PRIMARY KEY,
        name TEXT NOT NULL,
        cellphone_number TEXT,
        email_address TEXT,
        nrc TEXT,
        password_hash TEXT NOT NULL
    );
