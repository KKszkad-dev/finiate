-- Add migration script here

CREATE TABLE IF NOT EXISTS agenda
(
    id              TEXT PRIMARY KEY,
    title           VARCHAR(250)        NOT NULL,
    agenda_status   TEXT                NOT NULL,
    initiate_at     INTEGER             NOT NULL,
    terminate_at    INTEGER             NOT NULL
);

CREATE TABLE IF NOT EXISTS log
(
    id              TEXT PRIMARY KEY,
    create_at       INTEGER             NOT NULL,
    content         TEXT                NOT NULL,
    log_type        TEXT                NOT NULL,
    agenda_id       TEXT,
    FOREIGN KEY (agenda_id) REFERENCES agenda(id)
    ON DELETE SET NULL
    ON UPDATE CASCADE
);