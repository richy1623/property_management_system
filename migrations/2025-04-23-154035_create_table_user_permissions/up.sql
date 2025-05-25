-- Your SQL goes here
CREATE TYPE permission_type AS ENUM (
    'EMPLOYEE',
    'VIEW_PAYMENTS',
    'MODIFY_PAYMENTS',
    'VIEW_USERS',
    'MODIFY_USERS',
    'VIEW_PROPERTIES',
    'MODIFY_PROPERTIES',
    'VIEW_OWNERSHIPS',
    'MODIFY_OWNERSHIPS',
    'VIEW_REPORTS'
);

CREATE TABLE
    user_permission (
        user_id INTEGER NOT NULL,
        permission permission_type NOT NULL,
        PRIMARY KEY (user_id, permission),
        CONSTRAINT fk_user_permission_user FOREIGN KEY (user_id) REFERENCES "user" (user_id)
    );

CREATE INDEX idx__user_permission__user_id ON user_permission (user_id);
