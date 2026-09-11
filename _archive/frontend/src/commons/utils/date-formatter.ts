export function formatDate(
  date: string | Date,
  locale?: string,
  variant: "date-only" | "full" = "full",
): string {
  const d = typeof date === "string" ? new Date(date) : date;
  const lng = locale ?? "en";

  const options: Intl.DateTimeFormatOptions =
    variant === "date-only"
      ? { year: "numeric", month: "2-digit", day: "2-digit" }
      : {
          year: "numeric",
          month: "2-digit",
          day: "2-digit",
          hour: "2-digit",
          minute: "2-digit",
          second: "2-digit",
        };

  return new Intl.DateTimeFormat(lng, options).format(d);
}

export function formatDuration(ms: number): string {
  if (ms < 1) {
    return "< 1ms";
  }
  if (ms < 1000) {
    return `${Math.round(ms)}ms`;
  }
  const seconds = ms / 1000;
  if (seconds < 60) {
    return `${seconds.toFixed(1)}s`;
  }
  const minutes = Math.floor(seconds / 60);
  const remainingSeconds = Math.round(seconds % 60);
  if (minutes < 60) {
    return `${minutes}m ${remainingSeconds}s`;
  }
  const hours = Math.floor(minutes / 60);
  const remainingMinutes = minutes % 60;
  return `${hours}h ${remainingMinutes}m`;
}

export function formatRelativeTime(date: string | Date, locale?: string): string {
  const d = typeof date === "string" ? new Date(date) : date;
  const now = new Date();
  const diffMs = now.getTime() - d.getTime();
  const diffSeconds = Math.floor(diffMs / 1000);
  const diffMinutes = Math.floor(diffSeconds / 60);
  const diffHours = Math.floor(diffMinutes / 60);
  const diffDays = Math.floor(diffHours / 24);

  const lng = locale ?? "en";
  const rtf = new Intl.RelativeTimeFormat(lng, { numeric: "auto" });

  if (diffSeconds < 60) {
    return rtf.format(-diffSeconds, "second");
  }
  if (diffMinutes < 60) {
    return rtf.format(-diffMinutes, "minute");
  }
  if (diffHours < 24) {
    return rtf.format(-diffHours, "hour");
  }
  return rtf.format(-diffDays, "day");
}
