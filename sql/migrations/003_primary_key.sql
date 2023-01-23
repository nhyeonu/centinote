ALTER TABLE journals RENAME TO journals_old;
CREATE TABLE journals (
    uuid CHAR(36) NOT NULL,
    user_uuid CHAR(36) NOT NULL,
    created TIMESTAMP NOT NULL,
    timezone_offset INTEGER NOT NULL,
    title TEXT NOT NULL,
    body TEXT NOT NULL,
    PRIMARY KEY (uuid),
    FOREIGN KEY (user_uuid) REFERENCES users(uuid)
);
INSERT INTO journals SELECT * FROM journals_old;
DROP TABLE journals_old;

DROP TABLE sessions;
CREATE TABLE sessions (
    uuid CHAR(36) NOT NULL,
    user_uuid CHAR(36) NOT NULL,
    expiry TIMESTAMP NOT NULL,
    token CHAR(64) NOT NULL,
    PRIMARY KEY (uuid),
    FOREIGN KEY (user_uuid) REFERENCES users(uuid)
);
