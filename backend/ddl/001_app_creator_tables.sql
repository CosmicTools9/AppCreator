-- AppCreator 持久化 DDL
-- 将 AppCreator 的 Projects / Builds / Deployments 从内存 HashMap 迁移到 Postgres 持久化。
-- 约定（对齐 Meta/backend/migrations 自有管理表）：
--   * 独立 schema `app_creator`，避免污染 `isahl_meta`。
--   * ID 使用独立 SEQUENCE + nextval（非 isahl_meta 生命周期表的 gen_next_zuid，遵守 NEVER 规则）。
--   * config 以 jsonb 存储（应用层自由结构）。
--   * 迁移由人工执行（Meta 迁移在启动时禁用，见 Meta/backend/src/main.rs:122）。

CREATE SCHEMA IF NOT EXISTS app_creator;

-- ── Projects ──
CREATE SEQUENCE IF NOT EXISTS app_creator.projects_id_seq;

CREATE TABLE IF NOT EXISTS app_creator.projects (
    id           bigint       PRIMARY KEY DEFAULT nextval('app_creator.projects_id_seq'),
    name         text         NOT NULL,
    namespace    text         NOT NULL DEFAULT '',
    description  text         NOT NULL DEFAULT '',
    status       text         NOT NULL DEFAULT 'draft',   -- draft | building | deployed | archived
    config       jsonb        NOT NULL DEFAULT '{}'::jsonb,
    template_id  bigint,
    created_by   bigint       NOT NULL DEFAULT 0,
    created_at   timestamptz  NOT NULL DEFAULT now(),
    updated_at   timestamptz  NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_app_creator_projects_namespace
    ON app_creator.projects (namespace);
CREATE INDEX IF NOT EXISTS idx_app_creator_projects_status
    ON app_creator.projects (status);

-- updated_at 自动维护
CREATE OR REPLACE FUNCTION app_creator.projects_set_updated_at()
RETURNS trigger AS $$
BEGIN
    NEW.updated_at = now();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS trg_app_creator_projects_updated_at ON app_creator.projects;
CREATE TRIGGER trg_app_creator_projects_updated_at
    BEFORE UPDATE ON app_creator.projects
    FOR EACH ROW EXECUTE FUNCTION app_creator.projects_set_updated_at();

-- ── Builds ──
CREATE SEQUENCE IF NOT EXISTS app_creator.builds_id_seq;

CREATE TABLE IF NOT EXISTS app_creator.builds (
    id          bigint       PRIMARY KEY DEFAULT nextval('app_creator.builds_id_seq'),
    project_id  bigint       NOT NULL,
    status      text         NOT NULL DEFAULT 'pending',  -- pending | running | success | failed
    log         text         NOT NULL DEFAULT '',
    created_at  timestamptz  NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_app_creator_builds_project
    ON app_creator.builds (project_id);

-- ── Deployments ──
CREATE SEQUENCE IF NOT EXISTS app_creator.deployments_id_seq;

CREATE TABLE IF NOT EXISTS app_creator.deployments (
    id          bigint       PRIMARY KEY DEFAULT nextval('app_creator.deployments_id_seq'),
    project_id  bigint       NOT NULL,
    build_id    bigint       NOT NULL,
    status      text         NOT NULL DEFAULT 'pending',  -- pending | running | success | failed
    target      text         NOT NULL DEFAULT '',
    created_at  timestamptz  NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_app_creator_deployments_project
    ON app_creator.deployments (project_id);
