//! 通用本体坐标展示组件
//!
//! 纯数据展示组件，不依赖外部 UI 库，供各业务模块复用。

import * as React from 'react';
import { useT } from '@alioth/i18n';
import { useEntityView, useEntityCoordinate, useEntityTimeline, useEntityInheritance, useStateTransition } from '../hooks';

function SectionCard({ title, children }: {
  title: string;
  children: React.ReactNode;
}) {
  return (
    <div style={{ background: '#fff', borderRadius: 12, border: '1px solid #e2e8f0', padding: 20, marginBottom: 16 }}>
      <h3 style={{ fontSize: 14, fontWeight: 600, color: '#0f172a', marginBottom: 12, marginTop: 0 }}>{title}</h3>
      {children}
    </div>
  );
}

function StatusBadge({ status, t }: { status?: { notice: string; color?: string | null } | null; t: ReturnType<typeof useT> }) {
  if (!status) return <span style={{ color: '#64748b' }}>{t('components.ontology.status.none')}</span>;
  const bg = status.color ? `${status.color}20` : '#f1f5f9';
  return (
    <span style={{
      display: 'inline-flex', alignItems: 'center', padding: '4px 10px', borderRadius: 9999,
      fontSize: 12, fontWeight: 600, background: bg, color: status.color || '#64748b'
    }}>
      {status.notice}
    </span>
  );
}

function DimensionBadge({ item }: { item?: { notice: string; color?: string | null } | null }) {
  if (!item) return <span style={{ color: '#64748b', fontSize: 14 }}>—</span>;
  return (
    <span style={{
      display: 'inline-flex', alignItems: 'center', padding: '2px 8px', borderRadius: 4,
      fontSize: 12, fontWeight: 500, background: '#f1f5f9', color: '#64748b'
    }}>
      {item.notice}
    </span>
  );
}

export interface EntityOntologyViewProps {
  table: string;
  entityId?: number;
  title?: string;
}

export function EntityOntologyView({ table, entityId, title }: EntityOntologyViewProps) {
  const t = useT();
  const { data: expression, isLoading: exprLoading } = useEntityView(table, entityId);
  const { data: coordinate } = useEntityCoordinate(table, entityId);
  const { data: timeline } = useEntityTimeline(table, entityId);
  const { data: inheritance } = useEntityInheritance(table, entityId);

  const transition = useStateTransition();

  if (exprLoading) {
    return (
      <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', height: 256 }}>
        <div style={{ width: 32, height: 32, border: '3px solid #e2e8f0', borderTopColor: '#3b82f6', borderRadius: '50%', animation: 'spin 1s linear infinite' }} />
      </div>
    );
  }

  if (!expression) {
    return (
      <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', height: 256, color: '#64748b' }}>
        {t('components.ontology.empty.notFound')}
      </div>
    );
  }

  return (
    <div style={{ maxWidth: 1024, margin: '0 auto', padding: 24 }}>
      <style>{`
        @keyframes spin { to { transform: rotate(360deg); } }
      `}</style>

      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start', marginBottom: 24 }}>
        <div>
          <h1 style={{ fontSize: 24, fontWeight: 700, color: '#0f172a', margin: 0 }}>{title || expression.name}</h1>
          <p style={{ fontSize: 14, color: '#64748b', fontFamily: 'monospace', margin: '4px 0 0' }}>{expression.code}</p>
          <div style={{ display: 'flex', gap: 8, marginTop: 8 }}>
            <StatusBadge status={expression.status} t={t} />
            <span style={{ fontSize: 12, color: '#64748b', padding: '4px 8px', borderRadius: 4, background: '#f1f5f9' }}>
              {expression.phase}
            </span>
          </div>
        </div>
        <div style={{ textAlign: 'right' }}>
          <p style={{ fontSize: 12, color: '#64748b', margin: 0 }}>{t('components.ontology.space.title')}</p>
          <p style={{ fontSize: 14, fontWeight: 500, margin: '4px 0 0' }}>{expression.space_expression || '—'}</p>
        </div>
      </div>

      <SectionCard title={`${t('components.ontology.space.title')} (Scene × Factor × Function)`}>
        <div style={{ display: 'grid', gridTemplateColumns: 'repeat(3, 1fr)', gap: 16 }}>
          <div>
            <p style={{ fontSize: 12, color: '#64748b', margin: '0 0 4px' }}>{t('components.ontology.space.scene')}</p>
            <DimensionBadge item={coordinate?.space.scene} />
          </div>
          <div>
            <p style={{ fontSize: 12, color: '#64748b', margin: '0 0 4px' }}>{t('components.ontology.space.factor')}</p>
            <DimensionBadge item={coordinate?.space.factor} />
          </div>
          <div>
            <p style={{ fontSize: 12, color: '#64748b', margin: '0 0 4px' }}>{t('components.ontology.space.function')}</p>
            <DimensionBadge item={coordinate?.space.function} />
          </div>
        </div>
      </SectionCard>

      <SectionCard title={t('components.ontology.timeline.title')}>
        <div>
          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start' }}>
            <div>
              <p style={{ fontSize: 12, color: '#64748b', margin: '0 0 4px' }}>{t('components.ontology.timeline.currentStatus')}</p>
              <StatusBadge status={expression.status} t={t} />
            </div>
            <div style={{ textAlign: 'right' }}>
              <p style={{ fontSize: 12, color: '#64748b', margin: '0 0 4px' }}>{t('components.ontology.timeline.availableTransitions')}</p>
              <div style={{ display: 'flex', gap: 4, flexWrap: 'wrap' }}>
                {timeline?.available_transitions.slice(0, 5).map((s) => (
                  <button
                    key={s.id}
                    style={{ padding: '4px 8px', borderRadius: 4, border: '1px solid #e2e8f0', fontSize: 12, background: '#fff', cursor: 'pointer' }}
                    onClick={() => {
                      if (entityId) {
                        transition.mutate({ table, id: entityId, target_status_id: s.id });
                      }
                    }}
                  >
                    {s.notice}
                  </button>
                ))}
              </div>
            </div>
          </div>

          {timeline && timeline.history.length > 0 && (
            <div style={{ marginTop: 12, paddingTop: 12, borderTop: '1px solid #e2e8f0' }}>
              <p style={{ fontSize: 12, color: '#64748b', margin: '0 0 8px' }}>{t('components.ontology.timeline.history')}</p>
              <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
                {timeline.history.map((h, i) => (
                  <div key={i} style={{ display: 'flex', alignItems: 'center', gap: 8, fontSize: 14 }}>
                    <span style={{ color: '#64748b' }}>{h.from?.notice || t('components.ontology.timeline.initial')}</span>
                    <span style={{ color: '#94a3b8' }}>→</span>
                    <span style={{ fontWeight: 500 }}>{h.to.notice}</span>
                    <span style={{ fontSize: 12, color: '#94a3b8', marginLeft: 'auto' }}>
                      {new Date(h.triggered_at).toLocaleString()}
                    </span>
                  </div>
                ))}
              </div>
            </div>
          )}
        </div>
      </SectionCard>

      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(2, 1fr)', gap: 16 }}>
        <SectionCard title={t('components.ontology.category.title')}>
          <div style={{ display: 'flex', flexWrap: 'wrap', gap: 8 }}>
            {expression.categories.length > 0 ? (
              expression.categories.map((c) => (
                <span key={c.id} style={{ padding: '4px 10px', borderRadius: 9999, fontSize: 12, fontWeight: 500, background: '#eff6ff', color: '#2563eb' }}>
                  {c.notice}
                </span>
              ))
            ) : (
              <span style={{ fontSize: 14, color: '#64748b' }}>{t('components.ontology.category.empty')}</span>
            )}
          </div>
        </SectionCard>

        <SectionCard title={t('components.ontology.tag.title')}>
          <div style={{ display: 'flex', flexWrap: 'wrap', gap: 8 }}>
            {expression.tags.length > 0 ? (
              expression.tags.map((t) => (
                <span key={t.id} style={{ padding: '4px 10px', borderRadius: 9999, fontSize: 12, fontWeight: 500, background: '#fffbeb', color: '#d97706' }}>
                  {t.notice}
                </span>
              ))
            ) : (
              <span style={{ fontSize: 14, color: '#64748b' }}>{t('components.ontology.tag.empty')}</span>
            )}
          </div>
        </SectionCard>
      </div>

      <SectionCard title={t('components.ontology.inheritance.title')}>
        <div>
          <div style={{ display: 'flex', alignItems: 'center', gap: 8, fontSize: 14, marginBottom: 8 }}>
            <span style={{ color: '#64748b' }}>{t('components.ontology.inheritance.currentTable')}</span>
            <code style={{ padding: '2px 8px', borderRadius: 4, background: '#f1f5f9', fontSize: 12, fontFamily: 'monospace' }}>{inheritance?.current_table}</code>
            <span style={{
              fontSize: 12, padding: '2px 8px', borderRadius: 9999,
              background: inheritance?.is_leaf ? '#ecfdf5' : '#f3f4f6',
              color: inheritance?.is_leaf ? '#059669' : '#6b7280'
            }}>
              {inheritance?.is_leaf ? t('components.ontology.inheritance.leaf') : t('components.ontology.inheritance.abstract')}
            </span>
          </div>
          <div style={{ display: 'flex', flexWrap: 'wrap', alignItems: 'center', gap: 4 }}>
            {inheritance?.path.map((node, i) => (
              <React.Fragment key={node.table_name}>
                <span style={{
                  padding: '4px 8px', borderRadius: 4, fontSize: 12, fontFamily: 'monospace',
                  background: node.is_abstract ? '#f1f5f9' : '#eff6ff',
                  color: node.is_abstract ? '#64748b' : '#2563eb'
                }}>
                  {node.table_name}
                </span>
                {i < (inheritance?.path.length ?? 0) - 1 && (
                  <span style={{ color: '#94a3b8', fontSize: 12 }}>→</span>
                )}
              </React.Fragment>
            ))}
          </div>
        </div>
      </SectionCard>

      <SectionCard title={t('components.ontology.form.title')}>
        <div style={{ display: 'flex', gap: 16 }}>
          <div>
            <p style={{ fontSize: 12, color: '#64748b', margin: '0 0 4px' }}>Phase</p>
            <p style={{ fontSize: 18, fontWeight: 600, margin: 0 }}>{expression.phase}</p>
          </div>
          <div style={{ width: 1, background: '#e2e8f0' }} />
          <div>
            <p style={{ fontSize: 12, color: '#64748b', margin: '0 0 4px' }}>{t('components.ontology.form.abstractLevel')}</p>
            <p style={{ fontSize: 18, fontWeight: 600, margin: 0 }}>{inheritance?.abstract_level ?? '—'}</p>
          </div>
        </div>
      </SectionCard>
    </div>
  );
}
