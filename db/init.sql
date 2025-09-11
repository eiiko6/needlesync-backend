CREATE TABLE IF NOT EXISTS projects (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    completed BOOLEAN DEFAULT FALSE,
    time INT NOT NULL
);

-- For test purposes
INSERT INTO projects (name, completed, time) VALUES
('First Project', false, 120),
('Second Project', true, 90);

