-- Your SQL goes here
CREATE TABLE entries (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL,
    FOREIGN KEY (user_id)
        REFERENCES users(id)
        ON DELETE CASCADE
);