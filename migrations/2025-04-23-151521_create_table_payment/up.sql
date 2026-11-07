-- Your SQL goes here
CREATE TYPE payment_state AS ENUM ('PENDING_APPROVAL', 'APPROVED', 'REJECTED');

CREATE TABLE
    payment (
        payment_id SERIAL PRIMARY KEY,
        ownership_id INTEGER NOT NULL REFERENCES ownership (ownership_id) ON DELETE CASCADE,
        date DATE NOT NULL,
        amount INTEGER NOT NULL,
        state payment_state NOT NULL,
        comment TEXT NOT NULL,
        reference TEXT NOT NULL,
        last_changed_by_user INTEGER NOT NULL REFERENCES "user" (user_id) ON DELETE CASCADE,
        proof_of_payment_artifact INTEGER REFERENCES artifact (artifact_id) ON DELETE SET NULL
    );

CREATE INDEX idx__payment__ownership_id ON payment (ownership_id);

CREATE INDEX idx__payment__state_hash ON payment USING HASH (state);
