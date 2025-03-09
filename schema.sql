DROP TABLE IF EXISTS goals;
DROP TYPE IF EXISTS goal_recurrence;
DROP TABLE IF EXISTS sessions;
DROP TABLE IF EXISTS envelopes;
DROP TABLE IF EXISTS users;

CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    subject TEXT NOT NULL UNIQUE,
    email TEXT NOT NULL UNIQUE,
    preferences JSONB
);

CREATE TABLE sessions (
    id SERIAL PRIMARY KEY,
    user_id INTEGER REFERENCES users(id),
    expiration TIMESTAMP WITH TIME ZONE NOT NULL,
    csrf TEXT NOT NULL
);

CREATE TABLE envelopes (
    id SERIAL PRIMARY KEY,
    user_id INTEGER REFERENCES users(id),
    name TEXT NOT NULL,
    amount DECIMAL(10, 2) NOT NULL
);

CREATE TYPE goal_recurrence AS ENUM ('daily', 'weekly', 'monthly', 'yearly', 'never');

CREATE TABLE goals (
    id SERIAL PRIMARY KEY,
    user_id INTEGER REFERENCES users(id),
    name TEXT NOT NULL,
    target DECIMAL(10, 2) NOT NULL,
    target_date DATE NOT NULL,
    recurrence goal_recurrence NOT NULL
);