import type {
  DetectedOpeningLine,
  ScheduledOpeningLine,
} from '../../types/repertoire';
import { linesMatch } from '../../utils/importedLines.js';

export type ImportPlan = {
  line: ScheduledOpeningLine;
  createdAt: Date;
  messages: string[];
};

export interface ImportPlanner {
  planLine(line: DetectedOpeningLine, referenceDate?: Date): ImportPlan;
  planBulk(lines: DetectedOpeningLine[], referenceDate?: Date): ImportPlan[];
  persist(plan: ImportPlan): Promise<void>;
}

type Clock = () => Date;
type IdFactory = () => string;
type PersistPlan = (plan: ImportPlan) => Promise<void> | void;

export type ImportPlannerDependencies = {
  clock?: Clock;
  idFactory?: IdFactory;
  persistPlan?: PersistPlan;
  initialLines?: ScheduledOpeningLine[];
};

const cloneDate = (date: Date): Date => new Date(date.getTime());

const scheduleDate = (baseDate: Date, offset: number): string => {
  const scheduled = cloneDate(baseDate);
  scheduled.setDate(scheduled.getDate() + 1 + offset);
  return scheduled.toISOString().slice(0, 10);
};

const defaultIdFactory: IdFactory = () => {
  if (typeof globalThis.crypto?.randomUUID === 'function') {
    return globalThis.crypto.randomUUID();
  }

  return Math.random().toString(36).slice(2);
};

const defaultPersist: PersistPlan = async () => {
  // Intentionally left blank; consumers can supply a persistence adapter.
};

export const createImportPlanner = (
  dependencies: ImportPlannerDependencies = {},
): ImportPlanner => {
  const {
    clock = () => new Date(),
    idFactory = defaultIdFactory,
    persistPlan = defaultPersist,
    initialLines = [],
  } = dependencies;

  const plannedLines: ScheduledOpeningLine[] = [...initialLines];

  const resolveBaseDate = (referenceDate?: Date): Date => {
    if (referenceDate) {
      return cloneDate(referenceDate);
    }

    return cloneDate(clock());
  };

  const planLine = (
    line: DetectedOpeningLine,
    referenceDate?: Date,
  ): ImportPlan => {
    const baseDate = resolveBaseDate(referenceDate);
    const existing = plannedLines.find((candidate) => linesMatch(candidate, line));

    if (existing) {
      return {
        line: existing,
        createdAt: cloneDate(baseDate),
        messages: [`Line already scheduled for ${existing.scheduledFor}`],
      };
    }

    const scheduledFor = scheduleDate(baseDate, plannedLines.length);
    const scheduledLine: ScheduledOpeningLine = {
      ...line,
      id: idFactory(),
      scheduledFor,
    };

    plannedLines.push(scheduledLine);

    return {
      line: scheduledLine,
      createdAt: cloneDate(baseDate),
      messages: [`Line scheduled for ${scheduledFor}`],
    };
  };

  const planBulk = (
    lines: DetectedOpeningLine[],
    referenceDate?: Date,
  ): ImportPlan[] => {
    if (lines.length === 0) {
      return [];
    }

    const baseDate = resolveBaseDate(referenceDate);
    return lines.map((line) => planLine(line, baseDate));
  };

  const persist = async (plan: ImportPlan): Promise<void> => {
    await persistPlan(plan);
  };

  return {
    planLine,
    planBulk,
    persist,
  };
};
