import type { DetailedHTMLProps, HTMLAttributes } from 'react';
import type * as React from 'react';

type ChessBoardElementProps = DetailedHTMLProps<HTMLAttributes<HTMLElement>, HTMLElement> & {
  position?: string;
};

declare module 'react/jsx-runtime' {
  namespace JSX {
    type IntrinsicElements = React.JSX.IntrinsicElements & {
      'chess-board': ChessBoardElementProps;
    };
  }
}
