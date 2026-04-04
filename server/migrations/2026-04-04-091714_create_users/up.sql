-- Your SQL goes here
CREATE TABLE users (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL,
    password BYTEA NOT NULL,
    salt TEXT NOT NULL,
    public_key BYTEA NOT NULL
);