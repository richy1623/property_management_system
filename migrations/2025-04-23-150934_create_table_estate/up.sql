-- Your SQL goes here
CREATE TABLE
    estate (
        estate_id SERIAL PRIMARY KEY,
        name TEXT NOT NULL,
        location TEXT NOT NULL,
        district TEXT NOT NULL
    );
