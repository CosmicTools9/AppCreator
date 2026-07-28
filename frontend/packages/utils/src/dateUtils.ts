/**
 * Format date to locale string
 */
export function formatDate(date: Date | string | number): string {
  const d = new Date(date);
  return d.toLocaleDateString('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
  });
}

/**
 * Format datetime to locale string
 */
export function formatDateTime(date: Date | string | number): string {
  const d = new Date(date);
  return d.toLocaleString('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
  });
}

/**
 * Format relative time (e.g., "2 hours ago")
 */
export function formatRelativeTime(
  date: Date | string | number,
  t?: (key: string, params?: Record<string, string>) => string
): string {
  const now = new Date();
  const then = new Date(date);
  const diff = now.getTime() - then.getTime();

  const seconds = Math.floor(diff / 1000);
  const minutes = Math.floor(seconds / 60);
  const hours = Math.floor(minutes / 60);
  const days = Math.floor(hours / 24);

  if (days > 0) {
    return t ? t('components.date.daysAgo', { days: String(days) }) : 'components.date.daysAgo';
  }
  if (hours > 0) {
    return t ? t('components.date.hoursAgo', { hours: String(hours) }) : 'components.date.hoursAgo';
  }
  if (minutes > 0) {
    return t ? t('components.date.minutesAgo', { minutes: String(minutes) }) : 'components.date.minutesAgo';
  }
  return t ? t('components.date.justNow') : 'components.date.justNow';
}
