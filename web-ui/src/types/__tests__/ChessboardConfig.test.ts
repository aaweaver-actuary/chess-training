import { describe, expect, it, vi } from 'vitest';

import { ChessboardConfig } from '../chessboardConfig';

describe('ChessboardConfig', () => {
  it('provides chessboard-element defaults out of the box', () => {
    const config = new ChessboardConfig();

    expect(config.orientation).toBe('white');
    expect(config.showCoordinates).toBe(true);
    expect(config.hideNotation).toBe(false);
    expect(config.draggablePieces).toBe(false);
    expect(config.dropOffBoard).toBe('snapback');
    expect(config.animation).toEqual({
      moveSpeed: 200,
      snapbackSpeed: 60,
      snapSpeed: 30,
      trashSpeed: 100,
      appearSpeed: 200,
    });
    expect(config.style).toEqual({
      lightSquareColor: '#f0d9b5',
      darkSquareColor: '#b58863',
      highlightColor: 'yellow',
    });
    expect(config.toCssVariables()).toEqual({
      '--light-color': '#f0d9b5',
      '--dark-color': '#b58863',
      '--highlight-color': 'yellow',
    });
    expect(Object.isFrozen(config.animation)).toBe(true);
    expect(Object.isFrozen(config.style)).toBe(true);
    expect(Object.isFrozen(config.eventHandlers)).toBe(true);
  });

  it('merges overrides without mutating the base config', () => {
    const dropHandler = vi.fn();
    const base = new ChessboardConfig({
      orientation: 'black',
      showCoordinates: false,
      draggablePieces: true,
      animation: { moveSpeed: 'fast' },
      style: { highlightColor: '#ff00ff' },
      eventHandlers: { onDrop: dropHandler },
    });

    const overrideChangeHandler = vi.fn();
    const updated = base.withOverrides({
      showCoordinates: true,
      animation: { snapSpeed: 150 },
      style: { darkSquareColor: '#222222' },
      eventHandlers: { onChange: overrideChangeHandler },
    });

    expect(updated).not.toBe(base);
    expect(updated.orientation).toBe('black');
    expect(updated.showCoordinates).toBe(true);
    expect(updated.draggablePieces).toBe(true);
    expect(updated.animation).toEqual({
      moveSpeed: 'fast',
      snapbackSpeed: 60,
      snapSpeed: 150,
      trashSpeed: 100,
      appearSpeed: 200,
    });
    expect(updated.style).toEqual({
      lightSquareColor: '#f0d9b5',
      darkSquareColor: '#222222',
      highlightColor: '#ff00ff',
    });
    expect(updated.eventHandlers.onDrop).toBe(dropHandler);
    expect(updated.eventHandlers.onChange).toBe(overrideChangeHandler);

    expect(base.showCoordinates).toBe(false);
    expect(base.animation.snapSpeed).toBe(30);
    expect(base.style.darkSquareColor).toBe('#b58863');
  });
});
