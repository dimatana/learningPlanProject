DROP TABLE IF EXISTS bets;

CREATE TABLE bets (
                      id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                      event_id UUID NOT NULL,
                      stake FLOAT8 NOT NULL,
                      odds FLOAT8 NOT NULL,
                      created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);