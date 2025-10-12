import type {
  DetectedOpeningLine,
  ScheduledOpeningLine,
} from '../../types/repertoire';
import type { ImportPlan, ImportPlanner } from './ImportPlanner.js';

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

type OpeningPattern = {
  opening: string;
  color: DetectedOpeningLine['color'];
  moves: readonly string[];
};

const DEFAULT_PATTERNS: readonly OpeningPattern[] = [
  {
    opening: 'Danish Gambit',
    color: 'White',
    moves: ['e4', 'e5', 'd4', 'exd4', 'c3'],
  },
  {
    opening: "King's Knight Opening",
    color: 'White',
    moves: ['e4', 'e5', 'Nf3'],
  },
] as const;

const CLEAN_TAG_PATTERN = /\[[^\]]*\]/g;
const CLEAN_COMMENT_PATTERN = /\{[^}]*\}/g;
const CLEAN_VARIATION_PATTERN = /\([^)]*\)/g;

const sanitizeMoves = (input: string): string[] => {
  const cleaned = input
    .replace(CLEAN_TAG_PATTERN, ' ')
    .replace(CLEAN_COMMENT_PATTERN, ' ')
    .replace(CLEAN_VARIATION_PATTERN, ' ');

  return cleaned
    .split(/\s+/)
    .map((token) => token.trim())
    .filter((token) => token.length > 0)
    .map((token) => token.replace(/^[0-9]+\.\.\./, ''))
    .map((token) => token.replace(/^[0-9]+\./, ''))
    .map((token) => token.replace(/[?!+#]/g, ''))
    .filter((token) => token.length > 0);
};

const formatMoveSequence = (moves: string[]): string => {
  const segments: string[] = [];
  for (let index = 0; index < moves.length; index += 2) {
    const moveNumber = Math.floor(index / 2) + 1;
    const whiteMove = moves[index];
    const blackMove = moves[index + 1];

    if (whiteMove) {
      segments.push(`${String(moveNumber)}.${whiteMove}`);
    }

    if (blackMove) {
      segments.push(blackMove);
    }
  }
  return segments.join(' ');
};

const detectOpening = (
  moves: string[],
  patterns: readonly OpeningPattern[],
): DetectedOpeningLine | undefined => {
  if (moves.length === 0) {
    return undefined;
  }

  const normalized = moves.map((move) => move.toLowerCase());

  const matchedPattern = patterns.find((pattern) => {
    if (normalized.length < pattern.moves.length) {
      return false;
    }

    return pattern.moves.every((expectedMove, index) => {
      return normalized[index] === expectedMove.toLowerCase();
    });
  });

  if (!matchedPattern) {
    return undefined;
  }

  return {
    opening: matchedPattern.opening,
    color: matchedPattern.color,
    moves,
    display: formatMoveSequence(moves),
  } satisfies DetectedOpeningLine;
};

const defaultReadText = async (blob: Blob | File): Promise<string> => {
  if (typeof (blob as File).text === 'function') {
    return (blob as File).text();
  }

  throw new Error('Cannot read PGN text');
};

const deriveMessageTone = (headline: string): 'success' | 'info' => {
  return /already/i.test(headline) ? 'info' : 'success';
};

const toFeedbackMessage = (
  plan: ImportPlan,
  idFactory: () => string,
  dispatchAt: Date,
  acknowledgedHeadlines: Set<string>,
): PgnImportFeedbackMessage | undefined => {
  if (!plan.messages.length) {
    return undefined;
  }

  const [headline, ...rest] = plan.messages;

  if (!headline || acknowledgedHeadlines.has(headline)) {
    return undefined;
  }

  return {
    id: idFactory(),
    tone: deriveMessageTone(headline),
    headline,
    body: rest.length > 0 ? rest.join(' ') : undefined,
    dispatchAt,
  } satisfies PgnImportFeedbackMessage;
};

const createEmptyPreview = (): PgnImportPreview => ({
  normalizedPgn: '',
  detectedLines: [],
  scheduledLines: [],
});

type ServiceDependencies = {
  importPlanner: ImportPlanner;
  idFactory?: () => string;
  clock?: () => Date;
  readText?: (blob: Blob | File) => Promise<string>;
  patterns?: readonly OpeningPattern[];
};

export const createPgnImportService = ({
  importPlanner,
  idFactory = () => crypto.randomUUID(),
  clock = () => new Date(),
  readText = defaultReadText,
  patterns = DEFAULT_PATTERNS,
}: ServiceDependencies): PgnImportService => {
  const acknowledgedHeadlines = new Set<string>();

  const buildOutcome = (
    normalizedPgn: string,
    detectedLines: DetectedOpeningLine[],
    plans: ImportPlan[],
    errors: string[],
  ): PgnImportOutcome => {
    const scheduledLines = plans.map((plan) => plan.line);
    const dispatchAt = clock();
    const messages = plans
      .map((plan) => toFeedbackMessage(plan, idFactory, dispatchAt, acknowledgedHeadlines))
      .filter((message): message is PgnImportFeedbackMessage => Boolean(message));

    return {
      preview: {
        normalizedPgn,
        detectedLines,
        scheduledLines,
      },
      messages,
      errors,
    } satisfies PgnImportOutcome;
  };

  const handleDetection = async (text: string): Promise<PgnImportOutcome> => {
    const moves = sanitizeMoves(text);
    const normalizedPgn = moves.length > 0 ? formatMoveSequence(moves) : '';
    const detected = detectOpening(moves, patterns);

    if (!detected) {
      return buildOutcome(normalizedPgn, [], [], [
        "We could not recognize that PGN yet. Try a standard Danish Gambit or King's Knight Opening line.",
      ]);
    }

    try {
      const plans = importPlanner.planBulk([detected], clock());
      return buildOutcome(normalizedPgn, [detected], plans, []);
    } catch (error) {
      return buildOutcome(normalizedPgn, [detected], [], [
        error instanceof Error ? error.message : 'Unable to schedule the detected opening.',
      ]);
    }
  };

  return {
    async detect(source) {
      if (source.kind === 'text') {
        return handleDetection(String(source.value ?? ''));
      }

      try {
        const text = await readText(source.value as File | Blob);
        return handleDetection(text);
      } catch (error) {
        return {
          preview: createEmptyPreview(),
          messages: [],
          errors: ['We could not read that PGN file. Please try again.'],
        } satisfies PgnImportOutcome;
      }
    },
    acknowledge(outcome) {
      outcome.messages.forEach((message) => {
        acknowledgedHeadlines.add(message.headline);
      });
    },
    clear() {
      acknowledgedHeadlines.clear();
    },
  } satisfies PgnImportService;
};
