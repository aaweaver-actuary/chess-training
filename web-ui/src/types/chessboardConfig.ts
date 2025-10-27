import type {
  AnimationSpeed,
  OffBoardAction,
  PositionObject,
  RenderPieceFunction,
  SquareColor,
} from 'chessboard-element/lib/chessboard-element';
import type { Position } from 'chessboard-element/lib/chess-utils';

export interface ChessboardSquareEventDetail {
  square: string;
  piece: string | false;
  position: PositionObject;
  orientation: SquareColor;
}

export interface ChessboardSnapbackEventDetail {
  piece: string;
  square: string;
  position: PositionObject;
  orientation: SquareColor;
}

export interface ChessboardSnapEventDetail {
  source: string;
  square: string;
  piece: string;
}

export interface ChessboardDragStartEventDetail {
  source: string;
  piece: string;
  position: PositionObject;
  orientation: SquareColor;
}

export interface ChessboardDragMoveEventDetail {
  newLocation: string;
  oldLocation: string;
  source: string;
  piece: string;
  position: PositionObject;
  orientation: SquareColor;
}

export interface ChessboardDropEventDetail {
  source: string;
  target: string;
  piece: string;
  newPosition: PositionObject;
  oldPosition: PositionObject;
  orientation: SquareColor;
  setAction: (action: OffBoardAction) => void;
}

export interface ChessboardMoveEndEventDetail {
  oldPosition: PositionObject;
  newPosition: PositionObject;
}

export interface ChessboardChangeEventDetail {
  value: PositionObject;
  oldValue: PositionObject;
}

export interface ChessboardEventHandlers {
  onMouseoverSquare?: (event: CustomEvent<ChessboardSquareEventDetail>) => void;
  onMouseoutSquare?: (event: CustomEvent<ChessboardSquareEventDetail>) => void;
  onSnapbackEnd?: (event: CustomEvent<ChessboardSnapbackEventDetail>) => void;
  onSnapEnd?: (event: CustomEvent<ChessboardSnapEventDetail>) => void;
  onDragStart?: (event: CustomEvent<ChessboardDragStartEventDetail>) => void;
  onDragMove?: (event: CustomEvent<ChessboardDragMoveEventDetail>) => void;
  onDrop?: (event: CustomEvent<ChessboardDropEventDetail>) => void;
  onMoveEnd?: (event: CustomEvent<ChessboardMoveEndEventDetail>) => void;
  onChange?: (event: CustomEvent<ChessboardChangeEventDetail>) => void;
  onError?: (event: Event) => void;
}

export interface ChessboardAnimationOptions {
  moveSpeed?: AnimationSpeed;
  snapbackSpeed?: AnimationSpeed;
  snapSpeed?: AnimationSpeed;
  trashSpeed?: AnimationSpeed;
  appearSpeed?: AnimationSpeed;
}

export interface ChessboardStyleOptions {
  lightSquareColor?: string;
  darkSquareColor?: string;
  highlightColor?: string;
}

export interface ChessboardConfigOptions {
  position?: Position;
  orientation?: SquareColor;
  showCoordinates?: boolean;
  draggablePieces?: boolean;
  dropOffBoard?: OffBoardAction;
  pieceTheme?: string | ((piece: string) => string);
  renderPiece?: RenderPieceFunction;
  animation?: ChessboardAnimationOptions;
  sparePieces?: boolean;
  style?: ChessboardStyleOptions;
  eventHandlers?: ChessboardEventHandlers;
}

export type NormalizedChessboardAnimationOptions = Readonly<{
  moveSpeed: AnimationSpeed;
  snapbackSpeed: AnimationSpeed;
  snapSpeed: AnimationSpeed;
  trashSpeed: AnimationSpeed;
  appearSpeed: AnimationSpeed;
}>;

export type NormalizedChessboardStyleOptions = Readonly<{
  lightSquareColor: string;
  darkSquareColor: string;
  highlightColor: string;
}>;

export class ChessboardConfig {
  readonly position?: Position;

  readonly orientation: SquareColor;

  readonly showCoordinates: boolean;

  readonly draggablePieces: boolean;

  readonly dropOffBoard: OffBoardAction;

  readonly pieceTheme?: string | ((piece: string) => string);

  readonly renderPiece?: RenderPieceFunction;

  readonly animation: NormalizedChessboardAnimationOptions;

  readonly sparePieces: boolean;

  readonly style: NormalizedChessboardStyleOptions;

  readonly eventHandlers: Readonly<ChessboardEventHandlers>;

  constructor(options: ChessboardConfigOptions = {}) {
    const {
      position,
      orientation,
      showCoordinates,
      draggablePieces,
      dropOffBoard,
      pieceTheme,
      renderPiece,
      animation,
      sparePieces,
      style,
      eventHandlers,
    } = options;

    this.position = position;
    this.orientation = orientation ?? 'white';
    this.showCoordinates = showCoordinates ?? true;
    this.draggablePieces = draggablePieces ?? false;
    this.dropOffBoard = dropOffBoard ?? 'snapback';
    this.pieceTheme = pieceTheme;
    this.renderPiece = renderPiece;
    this.animation = Object.freeze({
      moveSpeed: animation?.moveSpeed ?? 200,
      snapbackSpeed: animation?.snapbackSpeed ?? 60,
      snapSpeed: animation?.snapSpeed ?? 30,
      trashSpeed: animation?.trashSpeed ?? 100,
      appearSpeed: animation?.appearSpeed ?? 200,
    });
    this.sparePieces = sparePieces ?? false;
    this.style = Object.freeze({
      lightSquareColor: style?.lightSquareColor ?? '#f0d9b5',
      darkSquareColor: style?.darkSquareColor ?? '#b58863',
      highlightColor: style?.highlightColor ?? 'yellow',
    });
    this.eventHandlers = Object.freeze({ ...(eventHandlers ?? {}) });
  }

  get showNotation(): boolean {
    return this.showCoordinates;
  }

  get hideNotation(): boolean {
    return !this.showCoordinates;
  }

  toCssVariables(): Record<string, string> {
    return {
      '--light-color': this.style.lightSquareColor,
      '--dark-color': this.style.darkSquareColor,
      '--highlight-color': this.style.highlightColor,
    };
  }

  toObject(): ChessboardConfigOptions {
    return {
      position: this.position,
      orientation: this.orientation,
      showCoordinates: this.showCoordinates,
      draggablePieces: this.draggablePieces,
      dropOffBoard: this.dropOffBoard,
      pieceTheme: this.pieceTheme,
      renderPiece: this.renderPiece,
      animation: { ...this.animation },
      sparePieces: this.sparePieces,
      style: { ...this.style },
      eventHandlers: { ...this.eventHandlers },
    };
  }

  withOverrides(overrides: ChessboardConfigOptions): ChessboardConfig {
    const base = this.toObject();
    const mergedAnimation = {
      ...(base.animation ?? {}),
      ...(overrides.animation ?? {}),
    } satisfies ChessboardAnimationOptions;
    const mergedStyle = {
      ...(base.style ?? {}),
      ...(overrides.style ?? {}),
    } satisfies ChessboardStyleOptions;
    const mergedHandlers = {
      ...(base.eventHandlers ?? {}),
      ...(overrides.eventHandlers ?? {}),
    } satisfies ChessboardEventHandlers;

    return new ChessboardConfig({
      ...base,
      ...overrides,
      animation: mergedAnimation,
      style: mergedStyle,
      eventHandlers: mergedHandlers,
    });
  }
}
