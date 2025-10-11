import type { DetectedOpeningLine, ScheduledOpeningLine } from '../types/repertoire';

type Clock = () => Date;

type OpeningLineScheduler = (line: DetectedOpeningLine, offset: number) => ScheduledOpeningLine;

const scheduleDate = (baseDate: Date, offset: number): string => {
  const nextDate = new Date(baseDate);
  nextDate.setDate(nextDate.getDate() + 1 + offset);
  return nextDate.toISOString().slice(0, 10);
};

export const createOpeningLineScheduler = (
  clock: Clock = () => new Date(),
): OpeningLineScheduler => {
  return (line, offset) => {
    const baseDate = clock();
    const scheduledFor = scheduleDate(baseDate, offset);
    return {
      ...line,
      id: ['import', baseDate.getTime().toString(), offset.toString()].join('-'),
      scheduledFor,
    };
  };
};

export const linesMatch = (
  candidate: ScheduledOpeningLine,
  target: DetectedOpeningLine,
): boolean => {
  if (candidate.opening !== target.opening || candidate.color !== target.color) {
    return false;
  }

  if (candidate.moves.length !== target.moves.length) {
    return false;
  }

  return candidate.moves.every(
    (move, index) => move.toLowerCase() === target.moves[index]?.toLowerCase(),
  );
};
