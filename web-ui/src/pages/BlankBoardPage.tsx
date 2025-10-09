import { useEffect, useMemo, useRef } from 'react';
import type { FC } from 'react';
import { Chess } from 'chess.js';
import type { Move } from 'chess.js';

import 'chessboard-element';

type DropDetail = {
  source: string;
  target: string;
  promotion?: string;
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

    gameRef.current = new Chess();

    const getBoardPosition = () =>
      gameRef.current.history({ verbose: true }).length === 0 ? START_POSITION : gameRef.current.fen();

    const syncBoard = () => {
      const position = getBoardPosition();
      board.setAttribute('position', position);
      if (board.setPosition) {
        board.setPosition(position, false);
      } else if (position === START_POSITION) {
        board.start?.(false);
      }
    };

    const handleDrop = (event: Event) => {
      const { detail } = event as DropEvent;
      if (!detail) {
        return;
      }

      let move: Move | null = null;
      try {
        move = gameRef.current.move({
          from: detail.source,
          to: detail.target,
          promotion: detail.promotion ?? 'q',
        });
      } catch (error) {
        move = null;
      }

      if (!move) {
        syncBoard();
        return;
      }

      syncBoard();
    };

    board.setAttribute('data-initial-position', START_POSITION);
    board.setAttribute('draggable-pieces', 'true');
    board.start?.(false);
    syncBoard();

    board.addEventListener('drop', handleDrop);
    return () => {
      board.removeEventListener('drop', handleDrop);
    };
  }, []);

  return (
    <main className="blank-board-page" aria-label="Sandbox board" style={shellStyle}>
      <chess-board ref={boardRef} data-testid="sandbox-board" position={START_POSITION} style={boardStyle} />
    </main>
  );
};
