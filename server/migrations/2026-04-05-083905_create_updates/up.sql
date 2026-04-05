-- Your SQL goes here
CREATE TABLE updates (
    id UUID PRIMARY KEY,
    entry_id UUID NOT NULL,
    created TIMESTAMP NOT NULL,
    aes_key BYTEA NOT NULL,
    sig BYTEA NOT NULL,
    FOREIGN KEY (entry_id)
        REFERENCES entries(id)
        ON DELETE CASCADE
);