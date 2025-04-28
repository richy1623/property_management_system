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
        user_id INTEGER NOT NULL,
        plot_id INTEGER NOT NULL,
        state ownership_state NOT NULL,
        total_sale_value INTEGER DEFAULT 0 NOT NULL,
        monthly_repayment_amount INTEGER DEFAULT 0 NOT NULL,
        CONSTRAINT fk_person FOREIGN KEY (user_id) REFERENCES "user" (user_id),
        CONSTRAINT fk_plot FOREIGN KEY (plot_id) REFERENCES plot (plot_id)
    );

CREATE INDEX idx__ownership__user_id ON ownership (user_id);
