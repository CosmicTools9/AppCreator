-- Drop AppCreator projects/builds/deployments tables
-- Builds/Deployments/Docker 链路已剥离（opensource standalone 轻量化）
-- 删除顺序：deployments → builds → projects（无 FOREIGN KEY 约束，顺序仅为整洁）
-- 前需 backup-ddl.sh（§6.8 验证-恢复协议）

DROP INDEX IF EXISTS app_creator.idx_app_creator_deployments_project;
DROP INDEX IF EXISTS app_creator.idx_app_creator_builds_project;
DROP INDEX IF EXISTS app_creator.idx_app_creator_projects_namespace;
DROP INDEX IF EXISTS app_creator.idx_app_creator_projects_status;

DROP TABLE IF EXISTS app_creator.deployments;
DROP TABLE IF EXISTS app_creator.builds;
DROP TABLE IF EXISTS app_creator.projects;

DROP SEQUENCE IF EXISTS app_creator.deployments_id_seq;
DROP SEQUENCE IF EXISTS app_creator.builds_id_seq;
DROP SEQUENCE IF EXISTS app_creator.projects_id_seq;

DROP FUNCTION IF EXISTS app_creator.projects_set_updated_at;
