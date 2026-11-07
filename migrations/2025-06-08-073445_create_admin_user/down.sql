-- This file should undo anything in `up.sql`
DELETE FROM "user"
WHERE
    user_name = 'admin';
