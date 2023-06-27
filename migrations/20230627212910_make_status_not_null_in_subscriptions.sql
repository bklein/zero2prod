BEGIN;
  -- backfill status
  UPDATE subscriptions
    SET status = 'confirmed'
    WHERE status IS NULL;
  -- enforce not null
  ALTER TABLE subscriptions ALTER COLUMN status SET NOT NULL;
COMMIT;
