CREATE TABLE IF NOT EXISTS user_app(
    id              INTEGER PRIMARY KEY,
    email           TEXT NOT NULL,
    phone_reminder  TEXT NULL DEFAULT(NULL),
    account_role    TEXT NOT NULL DEFAULT('user'),
    is_subscribed   BOOLEAN NOT NULL DEFAULT(0),
    is_enabled      BOOLEAN NOT NULL DEFAULT(1),
    created_at      TEXT NOT NULL DEFAULT (datetime('now','utc')),
    updated_at      TEXT NOT NULL DEFAULT (datetime('now','utc'))
);
CREATE INDEX IF NOT EXISTS idx_email_user_app
ON user_app (email);


CREATE TABLE IF NOT EXISTS user_sub_payment(
    id                      INTEGER PRIMARY KEY,
    user_id                 INTEGER REFERENCES user_app(id) ON DELETE CASCADE,
    mp_paym_id              TEXT NOT NULL,
    payment_idempotency_h   TEXT NOT NULL,
    transaction_amount      TEXT NOT NULL,
    installments            NUMERIC NOT NULL DEFAULT(1),
    payment_method_id       TEXT NOT NULL,
    issuer_id               TEXT NOT NULL,
    status                  TEXT NOT NULL,
    created_at              TEXT NOT NULL DEFAULT (datetime('now','utc')),
    updated_at              TEXT NOT NULL DEFAULT (datetime('now','utc')),
    UNIQUE(payment_idempotency_h),
    UNIQUE(mp_paym_id)
);


CREATE TABLE IF NOT EXISTS add_pet_balance(
  id                      INTEGER PRIMARY KEY,
  user_id                 INTEGER REFERENCES user_app(id) ON DELETE CASCADE,
  balance                 INTEGER NOT NULL DEFAULT(0),
  UNIQUE(user_id)
);


CREATE TABLE IF NOT EXISTS pet(
    id                      INTEGER PRIMARY KEY,
    user_app_id             INTEGER NOT NULL REFERENCES user_app(id) ON DELETE CASCADE,
    pet_name                TEXT NOT NULL,
    birthday                TEXT NOT NULL,
    breed                   TEXT NOT NULL,
    about                   TEXT NOT NULL,
    is_female               BOOLEAN NOT NULL,
    is_lost                 BOOLEAN NOT NULL,
    is_spaying_neutering    BOOLEAN NOT NULL,
    pic                     TEXT DEFAULT NULL,
    created_at              TEXT NOT NULL DEFAULT (datetime('now','utc')),
    updated_at              TEXT NOT NULL DEFAULT (datetime('now','utc'))
);


CREATE TABLE IF NOT EXISTS pet_linked(
  pet_id                  INTEGER REFERENCES pet(id) ON DELETE CASCADE,
  id_pet_external_id      INTEGER REFERENCES pet_external_id(id) ON DELETE CASCADE,
  UNIQUE(pet_id, id_pet_external_id)
);


CREATE TABLE IF NOT EXISTS pet_external_id(
  id                      INTEGER PRIMARY KEY,
  external_id             TEXT NOT NULL,
  created_at              TEXT NOT NULL DEFAULT (datetime('now','utc'))
);
CREATE INDEX IF NOT EXISTS idx_external_id_pet
ON pet_external_id (external_id);


CREATE TABLE IF NOT EXISTS pet_weight(
    id            INTEGER PRIMARY KEY,
    pet_id        INTEGER NOT NULL REFERENCES pet(id) ON DELETE CASCADE,
    weight        REAL NOT NULL,
    created_at    TEXT NOT NULL DEFAULT (datetime('now','utc'))
);


CREATE TABLE IF NOT EXISTS pet_health(
    id              INTEGER PRIMARY KEY,
    pet_id          INTEGER REFERENCES pet(id) ON DELETE CASCADE,
    health_record   TEXT NOT NULL,
    description     TEXT NOT NULL,
    created_at      TEXT NOT NULL DEFAULT (datetime('now','utc'))
);
CREATE INDEX IF NOT EXISTS idx_pet_health_record_type
ON pet_health (health_record);


CREATE TABLE IF NOT EXISTS owner_contact(
  id            INTEGER PRIMARY KEY,
  user_app_id   INTEGER NOT NULL REFERENCES user_app(id) ON DELETE CASCADE,
  full_name     TEXT NOT NULL,
  contact_value TEXT NOT NULL,
  created_at    TEXT NOT NULL DEFAULT (datetime('now','utc')),
  UNIQUE(contact_value)
);


CREATE TABLE IF NOT EXISTS pet_note(
  id              INTEGER PRIMARY KEY,
  pet_id          INTEGER REFERENCES pet(id) ON DELETE CASCADE,
  title           TEXT NOT NULL,
  content         TEXT NOT NULL,
  created_at      TEXT NOT NULL DEFAULT (datetime('now','utc')),
  updated_at      TEXT NOT NULL DEFAULT (datetime('now','utc'))
);
CREATE INDEX IF NOT EXISTS idx_pet_note_title
ON pet_note (title);


CREATE TABLE IF NOT EXISTS reminder(
  id                    INTEGER PRIMARY KEY,
  user_app_id           INTEGER NOT NULL REFERENCES user_app(id) ON DELETE CASCADE,
  body                  TEXT NOT NULL,
  execution_id          TEXT NOT NULL,
  notification_type     TEXT NOT NULL,
  send_at               TEXT NOT NULL DEFAULT (datetime('now','utc')),
  user_timezone         TEXT NOT NULL,
  created_at            TEXT NOT NULL DEFAULT (datetime('now','utc')),
  UNIQUE(execution_id)
);
CREATE INDEX IF NOT EXISTS idx_reminder_execution
ON reminder (execution_id);
