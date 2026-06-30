CREATE TABLE IF NOT EXISTS events (
   event_id UUID PRIMARY KEY,
   name TEXT NOT NULL,
   bets_placed BIGINT NOT NULL DEFAULT 0 CHECK (bets_placed >= 0)
 );