-- Your SQL goes here
CREATE TABLE
    plot (
        plot_id SERIAL PRIMARY KEY,
        estate_id INTEGER NOT NULL,
        plot_number TEXT NOT NULL,
        phase TEXT NOT NULL,
        length INTEGER NOT NULL,
        width INTEGER NOT NULL,
        CONSTRAINT fk_estate FOREIGN KEY (estate_id) REFERENCES estate (estate_id) ON DELETE CASCADE
    );

-- Create an index on plot_number
CREATE UNIQUE INDEX idx__plot__unique__plot_number_and_estate_id ON plot (plot_number, estate_id);
