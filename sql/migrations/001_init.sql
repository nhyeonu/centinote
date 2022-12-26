CREATE TABLE users ( 
    uuid CHAR(36) NOT NULL,
	username VARCHAR(64) NOT NULL,
	password_hash CHAR(96) NOT NULL,
    PRIMARY KEY (uuid)
);

CREATE TABLE sessions (
    user_uuid CHAR(36) NOT NULL,
    token CHAR(64) NOT NULL,
    FOREIGN KEY (user_uuid) REFERENCES users(uuid)
);

CREATE TABLE journals (
    uuid CHAR(36) NOT NULL,
    user_uuid CHAR(36) NOT NULL,
    created TIMESTAMP NOT NULL,
    timezone_offset INTEGER NOT NULL,
    title TEXT NOT NULL,
    body TEXT NOT NULL,
    FOREIGN KEY (user_uuid) REFERENCES users(uuid)
);
