import type {
  PgnImportFeedbackMessage,
  PgnImportOutcome,
  PgnImportService,
  PgnImportSource,
} from './PgnImportService.js';
import type { DetectedOpeningLine } from '../../types/repertoire.js';

const DEFAULT_PATTERNS: ReadonlyArray<{
  opening: string;
  color: DetectedOpeningLine['color'];
  moves: readonly string[];
}> = [
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
];

type Dependencies = {
  clock?: () => Date;
  generateId?: () => string;
  patterns?: ReadonlyArray<{
    opening: string;
    color: DetectedOpeningLine['color'];
    moves: readonly string[];
  }>;
};

const sanitizeMoves = (input: string): string[] =>
  input
    .replace(/\s+/g, ' ')
    .split(' ')
    .map((token) => token.trim())
    .map((token) => token.replace(/^[0-9]+\.\.\./, ''))
    .map((token) => token.replace(/^[0-9]+\./, ''))
    .map((token) => token.replace(/[?!+#]/g, ''))
    .filter((token) => token.length > 0);

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
  patterns: ReadonlyArray<{
    opening: string;
    color: DetectedOpeningLine['color'];
    moves: readonly string[];
  }>,
): DetectedOpeningLine | undefined => {
  if (moves.length === 0) {
    return undefined;
  }

  const normalized = moves.map((move) => move.toLowerCase());
  const matched = patterns.find((pattern) => {
    if (normalized.length < pattern.moves.length) {
      return false;
    }

    return pattern.moves.every((expectedMove, index) => {
      return normalized[index] === expectedMove.toLowerCase();
    });
  });

  if (!matched) {
    return undefined;
  }

  return {
    opening: matched.opening,
    color: matched.color,
    moves: [...moves],
    display: formatMoveSequence(moves),
  } satisfies DetectedOpeningLine;
};

const readSource = async (source: PgnImportSource): Promise<string> => {
  if (source.kind === 'text') {
    return String(source.value ?? '');
  }

  const blobLike = source.value as Blob;
  if (blobLike && typeof blobLike.text === 'function') {
    return blobLike.text();
  }

  if (blobLike && typeof blobLike.arrayBuffer === 'function') {
    const buffer = await blobLike.arrayBuffer();
    return new TextDecoder().decode(buffer);
  }

  if (typeof Response !== 'undefined') {
    try {
      const response = new Response(blobLike as BodyInit);
      return await response.text();
    } catch (error) {
      console.warn('PgnImportService: failed to decode blob source', error);
    }
  }

  return String(blobLike ?? '');
};

const buildSuccessMessage = (
  line: DetectedOpeningLine,
  clock: () => Date,
  generateId: () => string,
): PgnImportFeedbackMessage => ({
  id: generateId(),
  tone: 'info',
  headline: `Detected ${line.opening} (${line.color})`,
  body: `Line preview: ${line.display}`,
  dispatchAt: clock(),
});

const emptyOutcome: PgnImportOutcome = {
  preview: {
    normalizedPgn: '',
    detectedLines: [],
    scheduledLines: [],
  },
  messages: [],
  errors: [],
};

export const createPgnImportService = (
  dependencies: Dependencies = {},
): PgnImportService => {
  const {
    clock = () => new Date(),
    generateId = () => (typeof crypto !== 'undefined' ? crypto.randomUUID() : Math.random().toString(36).slice(2)),
    patterns = DEFAULT_PATTERNS,
  } = dependencies;

  const pendingMessages = new Map<string, PgnImportFeedbackMessage>();

  const buildOutcome = (outcome: PgnImportOutcome): PgnImportOutcome => {
    pendingMessages.clear();
    outcome.messages.forEach((message) => {
      pendingMessages.set(message.id, message);
    });
    return outcome;
  };

  return {
    async detect(source) {
      const raw = (await readSource(source)).trim();
      if (raw.length === 0) {
        return buildOutcome({
          ...emptyOutcome,
          errors: ['PGN source did not include any moves.'],
        });
      }

      const moves = sanitizeMoves(raw);
      if (moves.length === 0) {
        return buildOutcome({
          ...emptyOutcome,
          errors: ['PGN source did not include any moves.'],
        });
      }

      const normalizedPgn = formatMoveSequence(moves);
      const detected = detectOpening(moves, patterns);

      if (!detected) {
        return buildOutcome({
          preview: {
            normalizedPgn,
            detectedLines: [],
            scheduledLines: [],
          },
          messages: [],
          errors: ['No matching opening found for the provided PGN.'],
        });
      }

      const message = buildSuccessMessage(detected, clock, generateId);
      return buildOutcome({
        preview: {
          normalizedPgn,
          detectedLines: [detected],
          scheduledLines: [],
        },
        messages: [message],
        errors: [],
      });
    },
    acknowledge(outcome) {
      outcome.messages.forEach((message) => {
        pendingMessages.delete(message.id);
      });
    },
    clear() {
      pendingMessages.clear();
    },
  };
};
