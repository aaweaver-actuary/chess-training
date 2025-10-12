import type {
  DetectedOpeningLine,
  ScheduledOpeningLine,
} from '../../types/repertoire';

export type PgnImportSource = {
  kind: 'text' | 'file';
  value: string | File | Blob;
};

export type PgnImportPreview = {
  normalizedPgn: string;
  detectedLines: DetectedOpeningLine[];
  scheduledLines: ScheduledOpeningLine[];
};

export type PgnImportFeedbackMessage = {
  id: string;
  tone: 'success' | 'info' | 'warning' | 'danger';
  headline: string;
  body?: string;
  dispatchAt: Date;
};

export type PgnImportOutcome = {
  preview: PgnImportPreview;
  messages: PgnImportFeedbackMessage[];
  errors: string[];
};

export interface PgnImportService {
  detect(source: PgnImportSource): Promise<PgnImportOutcome>;
  acknowledge(outcome: PgnImportOutcome): void;
  clear(): void;
}
