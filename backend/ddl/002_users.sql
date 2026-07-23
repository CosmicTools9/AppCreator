-- AppCreator standalone users table
-- 用于无密码登录：任意用户名只要 namespace 不冲突即可登录
-- Schema 独立（app_creator），不与 isahl_auth.auth_users 耦合

CREATE SEQUENCE IF NOT EXISTS app_creator.users_id_seq;

CREATE TABLE IF NOT EXISTS app_creator.users (
    id             bigint  PRIMARY KEY DEFAULT nextval('app_creator.users_id_seq'),
    username       text    NOT NULL,
    username_norm  text    NOT NULL,  -- lower(trim(username)) 大小写不敏感登录匹配
    namespace      text    NOT NULL,  -- 编码 NS-<PascalUsername>，满足 ^[A-Z][a-zA-Z0-9-]*$
    created_at     timestamptz NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX uq_app_creator_users_username_norm
    ON app_creator.users (username_norm);
CREATE UNIQUE INDEX uq_app_creator_users_namespace
    ON app_creator.users (namespace);

-- username_norm 自动维护
CREATE OR REPLACE FUNCTION app_creator.sync_username_norm()
RETURNS trigger AS $$
BEGIN
    NEW.username_norm = lower(trim(NEW.username));
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS trg_app_creator_sync_username_norm ON app_creator.users;
CREATE TRIGGER trg_app_creator_sync_username_norm
    BEFORE INSERT OR UPDATE ON app_creator.users
    FOR EACH ROW EXECUTE FUNCTION app_creator.sync_username_norm();
