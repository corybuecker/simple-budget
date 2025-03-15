ALTER TABLE goals ADD COLUMN accumulated_amount NUMERIC;
UPDATE goals SET accumulated_amount = 0.0;
ALTER TABLE goals ALTER COLUMN accumulated_amount SET NOT NULL;
