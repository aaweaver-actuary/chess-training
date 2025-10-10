const FALLBACK_LABEL = 'Date to be announced';

type DateFormatterOptions = Intl.DateTimeFormatOptions;

const defaultOptions: DateFormatterOptions = {
  month: 'short',
  day: 'numeric',
  timeZone: 'UTC',
};

export const UNLOCK_DATE_FALLBACK_LABEL = FALLBACK_LABEL;

export function formatUnlockDate(
  input: string,
  options: DateFormatterOptions = defaultOptions,
): string {
  const normalizedInput = /^\d{4}-\d{2}-\d{2}$/.test(input) ? `${input}T00:00:00Z` : input;

  const parsed = new Date(normalizedInput);

  if (Number.isNaN(parsed.getTime())) {
    return FALLBACK_LABEL;
  }

  // Default to UTC so that date-only strings are not shifted by the local timezone.
  return new Intl.DateTimeFormat(undefined, {
    timeZone: 'UTC',
    ...options,
  }).format(parsed);
}
