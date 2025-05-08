-- Could alter the table but we don't have the names for subs anyway so we
-- might as well drop it
DROP TABLE subscription;

CREATE TABLE subscription (
    id INTEGER PRIMARY KEY,
    endpoint TEXT NOT NULL,
    name TEXT NOT NULL,
    expiration_time INTEGER,
    auth_encr BLOB NOT NULL,
    salt BLOB NOT NULL,
    tag BLOB NOT NULL,
    p256dh BLOB NOT NULL,
    inserted DATE DEFAULT CURRENT_TIMESTAMP NOT NULL
);
