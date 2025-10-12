import type {
  DetectedOpeningLine,
  ScheduledOpeningLine,
} from '../../types/repertoire';

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
type IdGenerator = () => string;
type PersistHandler = (line: ScheduledOpeningLine) => Promise<void> | void;

export type ImportPlannerOptions = {
  clock?: Clock;
  idGenerator?: IdGenerator;
  persistLine?: PersistHandler;
  initialLines?: ScheduledOpeningLine[];
};

const formatDate = (date: Date): string => date.toISOString().slice(0, 10);

const calculateScheduledDate = (baseDate: Date, offset: number): string => {
  const next = new Date(baseDate.getTime());
  next.setUTCDate(next.getUTCDate() + 1 + offset);
  return formatDate(next);
};

const createLineKey = (line: DetectedOpeningLine | ScheduledOpeningLine): string => {
  const moves = line.moves.map((move) => move.trim().toLowerCase()).join('#');
  return `${line.color.toLowerCase()}::${line.opening.toLowerCase()}::${moves}`;
};

export class DefaultImportPlanner implements ImportPlanner {
  private readonly clock: Clock;

  private readonly idGenerator: IdGenerator;

  private readonly persistLine?: PersistHandler;

  private readonly knownLines: Map<string, ScheduledOpeningLine>;

  public constructor(options: ImportPlannerOptions = {}) {
    this.clock = options.clock ?? (() => new Date());
    this.idGenerator = options.idGenerator ?? (() => crypto.randomUUID());
    this.persistLine = options.persistLine;
    this.knownLines = new Map(
      (options.initialLines ?? []).map((line) => [createLineKey(line), line] as const),
    );
  }

  public planLine(line: DetectedOpeningLine, referenceDate?: Date): ImportPlan {
    const baseDate = referenceDate ? new Date(referenceDate.getTime()) : this.clock();
    const createdAt = new Date(baseDate.getTime());
    const key = createLineKey(line);
    const existing = this.knownLines.get(key);

    if (existing) {
      return {
        line: existing,
        createdAt,
        messages: [`Already scheduled for ${existing.scheduledFor}`],
      } satisfies ImportPlan;
    }

    const scheduledFor = calculateScheduledDate(baseDate, this.knownLines.size);
    const scheduledLine: ScheduledOpeningLine = {
      ...line,
      id: this.idGenerator(),
      scheduledFor,
    };

    this.knownLines.set(key, scheduledLine);

    return {
      line: scheduledLine,
      createdAt,
      messages: [`Scheduled for ${scheduledFor}`],
    } satisfies ImportPlan;
  }

  public planBulk(lines: DetectedOpeningLine[], referenceDate?: Date): ImportPlan[] {
    return lines.map((line) => this.planLine(line, referenceDate));
  }

  public async persist(plan: ImportPlan): Promise<void> {
    if (!this.persistLine) {
      return;
    }

    await this.persistLine(plan.line);
  }
}
