-- Users table
CREATE TABLE IF NOT EXISTS users (
    id SERIAL PRIMARY KEY,
    username TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    email TEXT
);

-- Projects table linked to users
CREATE TABLE IF NOT EXISTS projects (
    id SERIAL PRIMARY KEY,
    user_id INT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    completed BOOLEAN DEFAULT FALSE,
    time INT NOT NULL
);

-- Example users
INSERT INTO users (username, password_hash, email) VALUES
('alice', 'password1', 'alice@example.com'),
('bob', 'password2', 'bob@example.com');

-- Example projects for users
INSERT INTO projects (user_id, name, completed, time) VALUES
(1, 'Alice Project 1', false, 120),
(1, 'Alice Project 2', true, 560),
(2, 'Bob Project 1', false, 300);

