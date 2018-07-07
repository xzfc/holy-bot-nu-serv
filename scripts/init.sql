/* vim: sw=4 ts=4 et

Columns name convention:
    id     -- internal ids
    tg_id  -- telegram id
    rnd_id -- random unique textual id
    name   -- display name
    kind   -- 0 - tg; 1 - mx

*/

CREATE TABLE IF NOT EXISTS users (
    id       INTEGER NOT NULL PRIMARY KEY,
    rnd_id   TEXT    NOT NULL UNIQUE,
    name     TEXT    NOT NULL,
    kind     INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS users_tg (
    id       INTEGER NOT NULL PRIMARY KEY,
    tg_id    INTEGER NOT NULL UNIQUE,

    FOREIGN KEY (id) REFERENCES users(id)
);

CREATE TABLE IF NOT EXISTS users_mx (
    id       INTEGER NOT NULL PRIMARY KEY,
    mx_id    TEXT    NOT NULL UNIQUE,

    FOREIGN KEY (id) REFERENCES users(id)
);


CREATE TABLE IF NOT EXISTS chats (
    id       INTEGER NOT NULL PRIMARY KEY,
    rnd_id   TEXT    NOT NULL UNIQUE,
    alias    TEXT,
    name     TEXT    NOT NULL,
    kind     INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS chats_tg (
    id       INTEGER NOT NULL PRIMARY KEY,
    tg_id    INTEGER NOT NULL UNIQUE,

    FOREIGN KEY (id) REFERENCES chats(id)
);

CREATE TABLE IF NOT EXISTS chats_mx (
    id       INTEGER NOT NULL PRIMARY KEY,
    mx_id    TEXT    NOT NULL UNIQUE,

    FOREIGN KEY (id) REFERENCES chats(id)
);

CREATE TABLE IF NOT EXISTS messages (
    chat_id  INTEGER NOT NULL,
    user_id  INTEGER NOT NULL,
    hour     INTEGER NOT NULL,
    count    INTEGER NOT NULL,

    PRIMARY KEY (chat_id, user_id, hour),
    FOREIGN KEY (user_id) REFERENCES users(id),
    FOREIGN KEY (chat_id) REFERENCES chats(id)
);


CREATE TABLE IF NOT EXISTS replies (
    chat_id  INTEGER NOT NULL,
    from_uid INTEGER NOT NULL,
    to_uid   INTEGER NOT NULL,
    count    INTEGER NOT NULL,

    PRIMARY KEY (chat_id, from_uid, to_uid),
    FOREIGN KEY (chat_id) REFERENCES users(id)
);


CREATE TABLE IF NOT EXISTS kv (
    name     TEXT    NOT NULL PRIMARY KEY,
    value
);



/*

CREATE INDEX IF NOT EXISTS users_i0
ON users ( full_name );

CREATE UNIQUE INDEX IF NOT EXISTS chats_u0
ON chats ( random_id );

CREATE INDEX IF NOT EXISTS messages_i0
ON messages ( chat_id, day );

CREATE INDEX IF NOT EXISTS messages_i1
ON messages ( chat_id, (day+4) % 7 );

CREATE INDEX IF NOT EXISTS messages_i2
ON messages ( chat_id, user_id );
*/
