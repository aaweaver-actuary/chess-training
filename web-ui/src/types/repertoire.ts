export type LineColor = 'White' | 'Black';

export type DetectedOpeningLine = {
  opening: string;
  color: LineColor;
  moves: string[];
  display: string;
};

export type ScheduledOpeningLine = DetectedOpeningLine & {
  id: string;
  scheduledFor: string;
};

export type ImportResult = {
  added: boolean;
  line: ScheduledOpeningLine;
};
