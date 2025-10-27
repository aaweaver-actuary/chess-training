import type {
  DetectedOpeningLine,
  ScheduledOpeningLine,
} from '../../types/repertoire';
import type { ImportPlan, ImportPlanner } from './ImportPlanner';

type Clock = () => Date;
type GenerateId = () => string;
type PersistSink = (plan: ImportPlan) => Promise<void> | void;

type PlannerOptions = {
  clock?: Clock;
  generateId?: GenerateId;
  initialLines?: ScheduledOpeningLine[];
  persist?: PersistSink;
};

const defaultClock: Clock = () => new Date();
const defaultGenerateId: GenerateId = () => {
  if (typeof crypto !== 'undefined' && typeof crypto.randomUUID === 'function') {
    return crypto.randomUUID();
  }

  return Math.random().toString(36).slice(2);
};

const signatureFor = (line: Pick<DetectedOpeningLine, 'opening' | 'color' | 'moves'>): string => {
  const normalizedMoves = line.moves.map((move) => move.trim().toLowerCase()).join(' ');
  return `${line.opening.toLowerCase()}|${line.color.toLowerCase()}|${normalizedMoves}`;
};

const scheduleDate = (base: Date, offset: number): string => {
  const scheduled = new Date(base);
  scheduled.setHours(0, 0, 0, 0);
  scheduled.setDate(scheduled.getDate() + 1 + offset);
  return scheduled.toISOString().slice(0, 10);
};

export class DefaultImportPlanner implements ImportPlanner {
  private readonly clock: Clock;
  private readonly generateId: GenerateId;
  private readonly persistSink?: PersistSink;
  private readonly persisted: Map<string, ScheduledOpeningLine> = new Map();
  private readonly persistedOrder: string[] = [];
  private readonly staged: Map<string, ScheduledOpeningLine> = new Map();
  private stagedOrder: string[] = [];

  constructor(options: PlannerOptions = {}) {
    const { clock = defaultClock, generateId = defaultGenerateId, initialLines = [], persist } = options;
    this.clock = clock;
    this.generateId = generateId;
    this.persistSink = persist;

    for (const line of initialLines) {
      this.storePersisted(line);
    }
  }

  planLine(line: DetectedOpeningLine, referenceDate?: Date): ImportPlan {
    const signature = signatureFor(line);
    const existing = this.lookup(signature);
    const createdAt = this.clock();

    if (existing) {
      return {
        line: existing,
        createdAt,
        messages: [`Line already scheduled for ${existing.scheduledFor}`],
      };
    }

    const baseDate = referenceDate ? new Date(referenceDate) : new Date(createdAt);
    const offset = this.persistedOrder.length + this.stagedOrder.length;
    const scheduledLine: ScheduledOpeningLine = {
      ...line,
      id: this.generateId(),
      scheduledFor: scheduleDate(baseDate, offset),
    };

    this.staged.set(signature, scheduledLine);
    this.stagedOrder = [...this.stagedOrder, signature];

    return {
      line: scheduledLine,
      createdAt,
      messages: [`Scheduled ${line.opening} (${line.color}) for ${scheduledLine.scheduledFor}`],
    };
  }

  planBulk(lines: DetectedOpeningLine[], referenceDate?: Date): ImportPlan[] {
    const baseDate = referenceDate ? new Date(referenceDate) : this.clock();
    return lines.map((line) => this.planLine(line, baseDate));
  }

  async persist(plan: ImportPlan): Promise<void> {
    if (this.persistSink) {
      await this.persistSink(plan);
    }

    const signature = signatureFor(plan.line);
    this.staged.delete(signature);
    this.stagedOrder = this.stagedOrder.filter((value) => value !== signature);
    this.storePersisted(plan.line);
  }

  private lookup(signature: string): ScheduledOpeningLine | undefined {
    return this.staged.get(signature) ?? this.persisted.get(signature);
  }

  private storePersisted(line: ScheduledOpeningLine): void {
    const signature = signatureFor(line);
    if (this.persisted.has(signature)) {
      this.persisted.set(signature, line);
      return;
    }

    this.persisted.set(signature, line);
    this.persistedOrder.push(signature);
  }
}
