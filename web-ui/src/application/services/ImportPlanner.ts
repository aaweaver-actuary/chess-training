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
