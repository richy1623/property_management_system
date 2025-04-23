-- Your SQL goes here
CREATE OR REPLACE FUNCTION log_payment_state_change()
RETURNS TRIGGER AS $$
BEGIN
    -- Log only if the state actually changes
    IF NEW.state IS DISTINCT FROM OLD.state THEN
        INSERT INTO payment_state_audit (
            payment_id,
            previous_state,
            new_state,
            changed_by_user,
            comment
        )
        VALUES (
            NEW.payment_id,
            OLD.state,
            NEW.state,
            NEW.last_changed_by,
            NEW.comment
        );
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg__log_payment_state_change
AFTER UPDATE ON payment
FOR EACH ROW
WHEN (OLD.state IS DISTINCT FROM NEW.state)
EXECUTE FUNCTION log_payment_state_change();

