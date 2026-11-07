-- This file should undo anything in `up.sql`
-- Drop the trigger
DROP TRIGGER IF EXISTS trg__log_payment_state_change ON payment;

-- Drop the function
DROP FUNCTION IF EXISTS log_payment_state_change ();
