DROP TABLE sessions;

CREATE TABLE sessions (
    uuid CHAR(36) NOT NULL,
    user_uuid CHAR(36) NOT NULL,
    expiry TIMESTAMP NOT NULL,
    token CHAR(64) NOT NULL,
    FOREIGN KEY (user_uuid) REFERENCES users(uuid)
);
