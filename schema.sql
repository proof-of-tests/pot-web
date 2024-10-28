DROP TABLE IF EXISTS pot;
CREATE TABLE pot (
    wasm TEXT NOT NULL,
    created_at INTEGER NOT NULL DEFAULT (datetime('now')),
    seed INTEGER NOT NULL UNIQUE,
    hash INTEGER NOT NULL UNIQUE,
    owner INTEGER
);
