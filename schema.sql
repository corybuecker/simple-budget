DROP TABLE IF EXISTS goals;
DROP TYPE IF EXISTS "Recurrence";
DROP TABLE IF EXISTS sessions;
DROP TABLE IF EXISTS envelopes;
DROP TABLE IF EXISTS accounts;
DROP TABLE IF EXISTS users;

CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    subject TEXT NOT NULL UNIQUE,
    email TEXT NOT NULL UNIQUE,
    preferences JSONB
);

CREATE TABLE sessions (
    id UUID PRIMARY KEY NOT NULL,
    user_id INTEGER REFERENCES users(id) NOT NULL,
    expiration TIMESTAMP WITH TIME ZONE NOT NULL,
    csrf TEXT NOT NULL
);

CREATE TABLE envelopes (
    id SERIAL PRIMARY KEY,
    user_id INTEGER REFERENCES users(id) NOT NULL,
    name TEXT NOT NULL,
    amount DOUBLE PRECISION NOT NULL
);

CREATE TYPE "Recurrence" AS ENUM ('Daily', 'Weekly', 'Monthly', 'Quarterly', 'Yearly', 'Never');

CREATE TABLE goals (
    id SERIAL PRIMARY KEY,
    user_id INTEGER REFERENCES users(id) NOT NULL,
    name TEXT NOT NULL,
    target DOUBLE PRECISION NOT NULL,
    target_date TIMESTAMP WITH TIME ZONE NOT NULL,
    recurrence "Recurrence" NOT NULL
);

CREATE TABLE accounts (
    id SERIAL PRIMARY KEY,
    user_id INTEGER REFERENCES users(id) NOT NULL,
    name TEXT NOT NULL,
    amount DOUBLE PRECISION NOT NULL,
    debt BOOLEAN NOT NULL
);