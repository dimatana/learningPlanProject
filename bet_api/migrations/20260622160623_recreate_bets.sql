-- Add migration script here
DROP TABLE IF EXISTS bets;

CREATE TABLE bets (
                      id         UUID            PRIMARY KEY DEFAULT gen_random_uuid(),
                      stake      FLOAT8          NOT NULL,
                      odds       FLOAT8          NOT NULL,
                      bookmaker  VARCHAR(100)    NOT NULL,
                      status     VARCHAR(20)     NOT NULL DEFAULT 'pending',
                      created_at TIMESTAMPTZ     NOT NULL DEFAULT NOW()
);