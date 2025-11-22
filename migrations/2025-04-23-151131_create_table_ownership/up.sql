-- Your SQL goes here
CREATE TYPE ownership_state AS ENUM (
    'PENDING_CONTRACT',
    'ACTIVE_PAYMENT',
    'PAID',
    'DEFAULTED'
);

CREATE TABLE
    ownership (
        ownership_id SERIAL PRIMARY KEY,
        user_id INTEGER NOT NULL REFERENCES "user" (user_id) ON DELETE CASCADE,
        plot_id INTEGER NOT NULL REFERENCES plot (plot_id) ON DELETE CASCADE,
        state ownership_state NOT NULL,
        total_sale_value INTEGER DEFAULT 0 NOT NULL,
        monthly_repayment_amount INTEGER DEFAULT 0 NOT NULL,
        contract_start_date DATE
    );

CREATE INDEX idx__ownership__user_id ON ownership (user_id);
