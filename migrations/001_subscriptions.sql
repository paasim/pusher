CREATE TABLE IF NOT EXISTS subscription (
    id INTEGER PRIMARY KEY,
    endpoint TEXT NOT NULL,
    expiration_time INTEGER,
    auth_encr BLOB NOT NULL,
    tag BLOB NOT NULL,
    salt BLOB NOT NULL,
    p256dh BLOB NOT NULL,
    inserted DATE DEFAULT CURRENT_TIMESTAMP NOT NULL
);
