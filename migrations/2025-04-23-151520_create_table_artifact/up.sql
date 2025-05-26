-- Your SQL goes here
CREATE TABLE
    artifact (
        artifact_id SERIAL PRIMARY KEY,
        file_display_name TEXT NOT NULL,
        file_path TEXT NOT NULL,
        upload_date DATE NOT NULL,
        uploaded_by INTEGER NOT NULL REFERENCES "user" (user_id) ON DELETE CASCADE
    );
