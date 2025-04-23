-- Your SQL goes here
CREATE TYPE permission_type AS ENUM (
    'VIEW_PAYMENTS',
    'EDIT_PAYMENTS',
    'DELETE_PAYMENTS',
    'MANAGE_USERS',
    'VIEW_REPORTS'
    -- Add more as needed
);

CREATE TABLE
    user_permissions (
        user_id INTEGER NOT NULL,
        permission permission_type NOT NULL,
        PRIMARY KEY (user_id, permission),
        CONSTRAINT fk_user_permissions_user FOREIGN KEY (user_id) REFERENCES "user" (user_id)
    );

CREATE INDEX idx__user_permissions__user_id ON user_permissions (user_id);
