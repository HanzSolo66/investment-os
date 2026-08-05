ALTER TABLE assets
ADD COLUMN user_id BIGINT;

DO $$
DECLARE
    legacy_owner_id BIGINT;
BEGIN
    SELECT id
    INTO legacy_owner_id
    FROM users
    ORDER BY id
    LIMIT 1;

    IF EXISTS (SELECT 1 FROM assets)
       AND legacy_owner_id IS NULL THEN
        RAISE EXCEPTION
            'Cannot assign existing assets: no user exists';
    END IF;

    UPDATE assets
    SET user_id = legacy_owner_id
    WHERE user_id IS NULL;
END
$$;

ALTER TABLE assets
ALTER COLUMN user_id SET NOT NULL;

ALTER TABLE assets
ADD CONSTRAINT assets_user_id_fkey
FOREIGN KEY (user_id)
REFERENCES users (id)
ON DELETE CASCADE;

ALTER TABLE assets
DROP CONSTRAINT IF EXISTS assets_name_key;

DO $$
BEGIN
    IF EXISTS (
        SELECT 1
        FROM assets
        GROUP BY user_id, LOWER(name)
        HAVING COUNT(*) > 1
    ) THEN
        RAISE EXCEPTION
            'Cannot create per-user asset uniqueness: duplicate names exist';
    END IF;
END
$$;

CREATE UNIQUE INDEX assets_user_name_unique
ON assets (user_id, LOWER(name));

CREATE INDEX assets_user_id_index
ON assets (user_id);
