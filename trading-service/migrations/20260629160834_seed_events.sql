INSERT INTO events (event_id, name, bets_placed)
VALUES
    ('11111111-1111-1111-1111-111111111111', 'world cup final', 12),
    ('22222222-2222-2222-2222-222222222222', 'NBA final', 3)
    ON CONFLICT (event_id) DO NOTHING;
