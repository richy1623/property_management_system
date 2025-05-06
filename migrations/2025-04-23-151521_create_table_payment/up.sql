-- Your SQL goes here
CREATE TYPE payment_state AS ENUM ('PENDING_APPROVAL', 'APPROVED', 'REJECTED');

CREATE TABLE
    payment (
        payment_id SERIAL PRIMARY KEY,
        ownership_id INTEGER NOT NULL,
        date DATE NOT NULL,
        amount INTEGER NOT NULL,
        state payment_state NOT NULL,
        comment TEXT NOT NULL,
        reference TEXT NOT NULL,
        last_changed_by_user INTEGER NOT NULL,
        proof_of_payment_path TEXT NOT NULL,
        CONSTRAINT fk_last_changed_by_user FOREIGN KEY (last_changed_by_user) REFERENCES "user" (user_id),
        CONSTRAINT fk_ownership FOREIGN KEY (ownership_id) REFERENCES ownership (ownership_id)
    );

CREATE INDEX idx__payment__ownership_id ON payment (ownership_id);

CREATE INDEX idx__payment__state_hash ON payment USING HASH (state);
