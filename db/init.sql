CREATE TABLE IF NOT EXISTS projects (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    completed BOOLEAN DEFAULT FALSE,
    time INT NOT NULL
);

-- For test purposes
INSERT INTO projects (name, completed, time) VALUES
('First Project', false, 132),
('Second Project', false, 943),
('Third Project', false, 0),
('Fourth Project', true, 7456),
('Fifth Project', true, 9355);

