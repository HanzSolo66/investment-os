INSERT INTO users (
    username,
    password_hash
)
VALUES (
    'fixture_user',
    'test-password-hash'
);

INSERT INTO assets (
    user_id,
    name,
    unit_value,
    quantity
)
SELECT
    id,
    'Bitcoin',
    10.0,
    0.0
FROM users
WHERE username = 'fixture_user';
