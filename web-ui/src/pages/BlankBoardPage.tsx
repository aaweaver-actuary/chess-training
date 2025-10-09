import { useEffect, useMemo, useRef } from 'react';
import type { FC } from 'react';
import { Chess } from 'chess.js';
import type { Move, Square } from 'chess.js';

import 'chessboard-element';

type DropDetail = {
  source: string;
  target: string;
  promotion?: string;
  setAction?: (action: 'snapback' | 'trash') => void;
};

type DropEvent = CustomEvent<DropDetail | undefined>;

type ChessBoardElement = HTMLElement & {
  setPosition?: (position: string, useAnimation?: boolean) => void;
  start?: (useAnimation?: boolean) => void;
};

const START_POSITION = 'start';
const BOARD_SIZE = 'min(100vw, 100vh)';

export const BlankBoardPage: FC = () => {
  const boardRef = useRef<ChessBoardElement | null>(null);
  const gameRef = useRef(new Chess());
  const selectedSquareRef = useRef<string | null>(null);

  const shellStyle = useMemo(
    () => ({
      position: 'fixed' as const,
      inset: '0px',
      display: 'flex',
      alignItems: 'center',
      justifyContent: 'center',
      margin: '0px',
      padding: '0px',
      background: 'var(--color-app-shell-background)',
    }),
    [],
  );

  const boardStyle = useMemo(
    () => ({
      width: BOARD_SIZE,
      height: BOARD_SIZE,
      maxWidth: BOARD_SIZE,
      maxHeight: BOARD_SIZE,
    }),
    [],
  );

  useEffect(() => {
    const board = boardRef.current;
    if (!board) {
      return;
    }


    const getBoardPosition = () =>
      gameRef.current.history().length === 0 ? START_POSITION : gameRef.current.fen();

    const syncBoard = () => {
      const position = getBoardPosition();
      board.setAttribute('position', position);
      if (board.setPosition) {
        board.setPosition(position, false);
      } else if (position === START_POSITION) {
        board.start?.(false);
      }
    };

    const clearSelection = () => {
      selectedSquareRef.current = null;
      board.removeAttribute('data-active-square');
    };

    const restoreSelectionIndicator = () => {
      const activeSquare = selectedSquareRef.current;
      if (activeSquare) {
        board.setAttribute('data-active-square', activeSquare);
      } else {
        board.removeAttribute('data-active-square');
      }
    };

    const selectSquare = (square: string) => {
      selectedSquareRef.current = square;
      board.setAttribute('data-active-square', square);
    };

    const attemptMove = (
      from: string,
      to: string,
      options: { promotion?: string; setAction?: DropDetail['setAction'] } = {},
    ): boolean => {
      let move: Move | null = null;
      try {
        move = gameRef.current.move({
          from,
          to,
          promotion: options.promotion ?? 'q',
        });
      } catch (error) {
        move = null;
      }

      if (!move) {
        options.setAction?.('snapback');
        syncBoard();
        restoreSelectionIndicator();
        return false;
      }

      syncBoard();
      clearSelection();
      return true;
    };

    const handleDrop = (event: Event) => {
      const { detail } = event as DropEvent;
      if (!detail) {
        return;
      }

      attemptMove(detail.source, detail.target, {
        promotion: detail.promotion,
        setAction: detail.setAction,
      });
    };

    const extractSquareFromEvent = (event: Event): string | null => {
      if (typeof (event as { composedPath?: () => EventTarget[] }).composedPath !== 'function') {
        return null;
      }

      const eventWithComposedPath = event as Event & { composedPath: () => EventTarget[] };
      const path = eventWithComposedPath.composedPath();
      for (const element of path) {
        if (element instanceof HTMLElement) {
          const square = element.getAttribute('data-square');
          if (square) {
            return square;
          }
        }
      }

      return null;
    };

    const handleClick = (event: Event) => {
      const square = extractSquareFromEvent(event);
      if (!square) {
        clearSelection();
        return;
      }

      const currentSelection = selectedSquareRef.current;
      const piece = gameRef.current.get(square as Square);

      if (!currentSelection) {
        if (!piece || piece.color !== gameRef.current.turn()) {
          clearSelection();
          return;
        }

        selectSquare(square);
        return;
      }

      if (currentSelection === square) {
        clearSelection();
        return;
      }

      const moved = attemptMove(currentSelection, square);
      if (moved) {
        return;
      }

      if (piece && piece.color === gameRef.current.turn()) {
        selectSquare(square);
      } else if (!gameRef.current.get(currentSelection as Square)) {
        clearSelection();
      } else {
        restoreSelectionIndicator();
      }
    };

    board.setAttribute('data-initial-position', START_POSITION);
    board.setAttribute('draggable-pieces', 'true');
    board.start?.(false);
    syncBoard();

    board.addEventListener('drop', handleDrop);
    board.addEventListener('click', handleClick);
    return () => {
      board.removeEventListener('drop', handleDrop);
      board.removeEventListener('click', handleClick);
    };
  }, []);

  return (
    <main className="blank-board-page" aria-label="Sandbox board" style={shellStyle}>
      <chess-board ref={boardRef} data-testid="sandbox-board" position={START_POSITION} style={boardStyle} />
    </main>
  );
};
