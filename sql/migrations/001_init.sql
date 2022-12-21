CREATE TABLE users ( 
    Uuid CHAR(36) NOT NULL,
	Username VARCHAR(64) NOT NULL,
	HashedPassword CHAR(96) NOT NULL,
    PRIMARY KEY (Uuid)
);

CREATE TABLE sessions (
    UserUuid CHAR(36) NOT NULL,
    Token CHAR(64) NOT NULL,
    FOREIGN KEY (UserUuid) REFERENCES users(Uuid)
);

CREATE TABLE journals (
    Uuid CHAR(36) NOT NULL,
    UserUuid CHAR(36) NOT NULL,
    Title TEXT NOT NULL,
    Body TEXT NOT NULL,
    FOREIGN KEY (UserUuid) REFERENCES users(Uuid)
);
