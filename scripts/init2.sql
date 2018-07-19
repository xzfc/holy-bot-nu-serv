/* vim: sw=4 ts=4 et

Columns name convention:
    id     -- internal ids
    kind   -- 0 - tg; 1 - mx
    ext_id -- external id (numeric telegram id or textual matrix id)
    rnd_id -- random unique textual id to be visible by API
    name   -- display name
*/

CREATE TABLE IF NOT EXISTS users (
    id       INTEGER NOT NULL PRIMARY KEY,
    kind     INTEGER NOT NULL,
    ext_id           NOT NULL,
    rnd_id   TEXT    NOT NULL,
    name     TEXT    NOT NULL,

    UNIQUE (kind, ext_id),
    UNIQUE (rnd_id)
);

CREATE TABLE IF NOT EXISTS chats (
    id       INTEGER NOT NULL PRIMARY KEY,
    kind     INTEGER NOT NULL,
    ext_id           NOT NULL,
    rnd_id   TEXT    NOT NULL,
    name     TEXT    NOT NULL,
    alias    TEXT,

    UNIQUE (kind, ext_id),
    UNIQUE (rnd_id),
    UNIQUE (alias) ON CONFLICT REPLACE
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


CREATE TABLE IF NOT EXISTS chats_mx (
    id         NUMBER PRIMARY KEY,
    sync_start TEXT NOT NULL,
    sync_end   TEXT NOT NULL,
    FOREIGN KEY(id) REFERENCES chats(id)
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
