-- Your SQL goes here
CREATE TYPE artifact_service_type AS ENUM (
    'USER',
    'ESTATE',
    'PLOT',
    'OWNERSHIP',
    'PAYMENT'
);

CREATE TABLE
    artifact (
        artifact_id SERIAL PRIMARY KEY,
        file_display_name TEXT NOT NULL,
        file_path TEXT NOT NULL,
        upload_date DATE NOT NULL,
        uploaded_by INTEGER NOT NULL REFERENCES "user" (user_id) ON DELETE CASCADE,
        service_type artifact_service_type NOT NULL,
        service_reference_id INTEGER NOT NULL
    );

CREATE INDEX idx__artifact__service_type_service_reference_id ON artifact (service_type, service_reference_id);
