-- Your SQL goes here
-- 
INSERT INTO
    "user" (
        user_name,
        name,
        cellphone_number,
        email_address,
        password_hash,
        is_employee
    )
VALUES
    (
        'admin',
        'Administrator',
        '+27721234197',
        'richy1623@gmail.com',
        '$argon2id$v=19$m=19456,t=4,p=1$wE+hodHLGizc24bsaGmp0w$AdrY6NjgZmI4DM7MwhfO43T+MqSKOgPZedo7kIKI+kE',
        true
    );

INSERT INTO user_permission (user_id, permission)
SELECT u.user_id, e.enumlabel::permission_type
FROM pg_enum e
JOIN pg_type t ON t.oid = e.enumtypid
JOIN "user" u ON u.user_name = 'admin'
WHERE t.typname = 'permission_type';
