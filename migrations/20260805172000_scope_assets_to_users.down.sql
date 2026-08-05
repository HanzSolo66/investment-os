DROP INDEX IF EXISTS assets_user_id_index;
DROP INDEX IF EXISTS assets_user_name_unique;

ALTER TABLE assets
DROP CONSTRAINT IF EXISTS assets_user_id_fkey;

ALTER TABLE assets
DROP COLUMN user_id;

ALTER TABLE assets
ADD CONSTRAINT assets_name_key UNIQUE (name);
