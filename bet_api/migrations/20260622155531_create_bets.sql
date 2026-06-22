-- Add migration script here
CREATE TABLE bets (
                      id         BIGSERIAL        PRIMARY KEY,
                      user_id    BIGINT           NOT NULL,
                      event_id   VARCHAR(100)     NOT NULL,
                      market     VARCHAR(50)      NOT NULL,
                      selection  VARCHAR(100)     NOT NULL,
                      stake      NUMERIC(10, 2)   NOT NULL,
                      odds       NUMERIC(6, 3)    NOT NULL,
                      status     VARCHAR(20)      NOT NULL DEFAULT 'pending',
                      created_at TIMESTAMPTZ      NOT NULL DEFAULT NOW()
);