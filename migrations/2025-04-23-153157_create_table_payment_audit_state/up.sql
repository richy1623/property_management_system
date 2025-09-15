-- Your SQL goes here
CREATE TABLE
    payment_state_audit (
        audit_id SERIAL PRIMARY KEY,
        payment_id INTEGER NOT NULL,
        previous_state payment_state NOT NULL,
        new_state payment_state NOT NULL,
        change_timestamp TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
        changed_by_user INTEGER NOT NULL,
        comment TEXT NOT NULL,
        CONSTRAINT fk_payment FOREIGN KEY (payment_id) REFERENCES payment (payment_id) ON DELETE CASCADE,
        CONSTRAINT fk_changed_by FOREIGN KEY (changed_by_user) REFERENCES "user" (user_id)
    );

CREATE INDEX idx__payment_state_audit__payment_id ON payment_state_audit (payment_id);
