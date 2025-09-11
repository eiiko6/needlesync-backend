-- Users
INSERT INTO users (username, password_hash, email) VALUES
('alice', 'password1', 'alice@example.com'),
('bob', 'password2', 'bob@example.com');

-- Projects for users
INSERT INTO projects (user_id, name, completed, time) VALUES
(1, 'Alice Project 1', false, 120),
(1, 'Alice Project 2', true, 560),
(2, 'Bob Project 1', false, 300);
