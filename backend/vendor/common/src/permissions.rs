use crate::AliothError;
use sqlx::{AssertSqlSafe, PgPool};

pub async fn require_resource_access(
    pool: &PgPool,
    user_id: i64,
    resource_type: &str,
    resource_id: i64,
    action: &str,
) -> Result<(), AliothError> {
    // Skip all NGAC checks if the isahl_auth schema doesn't exist
    if !sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT FROM information_schema.schemata WHERE schema_name='isahl_auth')",
    )
    .fetch_one(pool)
    .await
    .unwrap_or(false)
    {
        return Ok(());
    }
    // From here on, isahl_auth schema is guaranteed to exist
    let owner_cte = r#"
        UNION
        SELECT op.owner_attr_id as ua_id, 0 as depth
        FROM isahl_auth.ngac_ownership_policy op, isahl.zc_id_lifecycle r
        WHERE op.resource_type = $2 AND r.id = $3
          AND r.created_by_id = $1 AND op.enabled = TRUE
        UNION
        SELECT op.benefit_attr_id as ua_id, 0 as depth
        FROM isahl_auth.ngac_ownership_policy op, isahl.zc_id_lifecycle r
        WHERE op.resource_type = $2 AND r.id = $3
          AND $1 = ANY(r.ak_benefit_user) AND op.enabled = TRUE
        UNION
        SELECT op.permit_attr_id as ua_id, 0 as depth
        FROM isahl_auth.ngac_ownership_policy op, isahl.zc_id_lifecycle r
        WHERE op.resource_type = $2 AND r.id = $3
          AND $1 = ANY(r.ak_permit_user) AND op.enabled = TRUE
        UNION
        SELECT op.access_attr_id as ua_id, 0 as depth
        FROM isahl_auth.ngac_ownership_policy op, isahl.zc_id_lifecycle r
        WHERE op.resource_type = $2 AND r.id = $3
          AND $1 = ANY(r.ak_access_user) AND op.enabled = TRUE
    "#
    .to_string();

    let sql = format!(
        "WITH RECURSIVE user_attrs AS (
            SELECT fk_user_attribute as ua_id, 0 as depth
            FROM isahl_auth.ngac_user_rr_attribute
            WHERE fk_user = $1 AND deleted_at IS NULL AND (expires_at IS NULL OR expires_at > NOW())
            UNION
            SELECT unnest(ua.ancestor_ids)::BIGINT as ua_id, depth + 1
            FROM isahl_auth.ngac_user_attribute ua
            JOIN user_attrs ua2 ON ua.id = ua2.ua_id
            WHERE ua2.depth < 10 AND ua.deleted_at IS NULL
            {0}
        ),
        resource_attrs AS (
            SELECT id as oa_id, 0 as depth
            FROM isahl_auth.ngac_object_attribute
            WHERE resource_type = $2 AND fk_resource = $3 AND deleted_at IS NULL
            UNION
            SELECT unnest(oa.ancestor_ids)::BIGINT as oa_id, depth + 1
            FROM isahl_auth.ngac_object_attribute oa
            JOIN resource_attrs ra ON oa.id = ra.oa_id
            WHERE ra.depth < 10 AND oa.deleted_at IS NULL
        )
        SELECT EXISTS(
            SELECT 1 FROM isahl_auth.ngac_association a
            JOIN user_attrs ua ON a.fk_user_attribute = ua.ua_id
            JOIN resource_attrs ra ON a.fk_object_attribute = ra.oa_id
            WHERE a.deleted_at IS NULL
              AND EXISTS(SELECT 1 FROM isahl_auth.ngac_access_right ar
                         WHERE ar.id = ANY(a.ak_access_rights) AND ar.o_name = $4)
        ) as permitted",
        owner_cte,
    );

    let permitted: bool = sqlx::query_scalar(AssertSqlSafe(sql.as_str()))
        .bind(user_id)
        .bind(resource_type)
        .bind(resource_id)
        .bind(action)
        .fetch_one(pool)
        .await
        .map_err(|e| AliothError::Internal(format!("Permission check: {}", e)))?;

    if !permitted {
        let bootstrap: (bool,) = sqlx::query_as(
            "SELECT COUNT(*)=0 FROM isahl_auth.ngac_association WHERE deleted_at IS NULL",
        )
        .fetch_one(pool)
        .await
        .map_err(|e| AliothError::Internal(format!("Bootstrap: {}", e)))?;

        if !bootstrap.0 {
            return Err(AliothError::Forbidden(format!(
                "Access denied: user {} lacks '{}' on {}:{}",
                user_id, action, resource_type, resource_id
            )));
        }
    }
    Ok(())
}
