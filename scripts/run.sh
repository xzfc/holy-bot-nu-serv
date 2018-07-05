#!/usr/bin/env zsh

init_db() {
    sqlite3 1.db '
CREATE TABLE IF NOT EXISTS messages (
    chat_id INTEGER,
    user_id INTEGER,
    day     INTEGER,
    hour    INTEGER,
    count   INTEGER,
    PRIMARY KEY (chat_id, user_id, day, hour)
);

CREATE INDEX IF NOT EXISTS messages_i0
ON messages ( chat_id, day );

CREATE INDEX IF NOT EXISTS messages_i1
ON messages ( chat_id, (day+4) % 7 );

CREATE INDEX IF NOT EXISTS messages_i2
ON messages ( chat_id, user_id );

CREATE TABLE IF NOT EXISTS users (
    user_id   INTEGER NOT NULL,
    full_name TEXT NOT NULL,
    random_id TEXT NOT NULL,
    PRIMARY KEY (user_id)
);

CREATE INDEX IF NOT EXISTS users_i0
ON users ( full_name );

CREATE TABLE IF NOT EXISTS replies (
    chat_id  INTEGER,
    uid_from INTEGER,
    uid_to   INTEGER,
    count    INTEGER,
    PRIMARY KEY (chat_id, uid_from, uid_to)
);

CREATE TABLE IF NOT EXISTS chats (
    chat_id   INTEGER,
    title     TEXT NOT NULL,
    username  TEXT,
    random_id TEXT NOT NULL,
    PRIMARY KEY (chat_id)
);

CREATE UNIQUE INDEX IF NOT EXISTS chats_u0
ON chats ( random_id );

CREATE TABLE IF NOT EXISTS seek (
    name  TEXT,
    value INTEGER,
    PRIMARY KEY (name)
);
'
}

mx_init() {
sqlite3 1.db '
DELETE FROM chats WHERE chat_id = 1;
DELETE FROM users WHERE user_id < 0;
DELETE FROM messages WHERE chat_id = 1;

INSERT INTO chats VALUES
(1, "Софтач", "s:pyos.anoosdy.org", "oSh2oop6");
'
	local i
	for i in /n/Dev/Matrix-History-Grabber/data/*;do
		echo $i
		./target/debug/batch sync-mx 1.db $i
	done
}

# BIN=./target/debug/batch
# TG_LOG=/n/Dev2/HolyCrackers/n/identity/data/b2
# BIN_LOG=log.txt


# init_db
# mx_init
"$@"

# while :;do
#     date
#     $BIN sync-tg 1.db /n/Dev2/HolyCrackers/n/identity/data/b2 2> $BIN_LOG
# done
