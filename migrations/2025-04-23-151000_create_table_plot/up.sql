-- Your SQL goes here
CREATE TABLE
    plot (
        plot_id SERIAL PRIMARY KEY,
        estate_id INTEGER NOT NULL,
        plot_number TEXT NOT NULL,
        length NUMERIC,
        width NUMERIC,
        CONSTRAINT fk_estate FOREIGN KEY (estate_id) REFERENCES estate (estate_id)
    );

-- Create an index on plot_number
CREATE INDEX idx__plot__plot_number ON plot (plot_number);

-- TODO is plot number unique
-- TODO collapse plot_id and estate_id into PRIMARY KEY
