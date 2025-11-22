-- Your SQL goes here
CREATE TABLE
    ownership_artifact (
        ownership_id INTEGER NOT NULL REFERENCES ownership (ownership_id) ON DELETE CASCADE,
        artifact_id INTEGER NOT NULL REFERENCES artifact (artifact_id) ON DELETE CASCADE,
        PRIMARY KEY (ownership_id, artifact_id)
    );

-- Create an index on ownership_id
CREATE INDEX idx__ownership_artifact__ownership_id ON ownership_artifact (ownership_id);
