const FALLBACK_LABEL = 'Date to be announced';

type DateFormatterOptions = Pick<Intl.DateTimeFormatOptions, 'month' | 'day'>;

const defaultOptions: DateFormatterOptions = {
  month: 'short',
  day: 'numeric',
};

export const UNLOCK_DATE_FALLBACK_LABEL = FALLBACK_LABEL;

export function formatUnlockDate(
  input: string,
  options: DateFormatterOptions = defaultOptions,
): string {
  const parsed = new Date(input);

  if (Number.isNaN(parsed.getTime())) {
    return FALLBACK_LABEL;
  }

  return new Intl.DateTimeFormat(undefined, options).format(parsed);
}
