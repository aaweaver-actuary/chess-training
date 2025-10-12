import { createRef } from 'react';

import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

import { FloatingCornerButton } from '../FloatingCornerButton';

const placements = ['top-right', 'top-left', 'bottom-right', 'bottom-left'] as const;

type Placement = (typeof placements)[number];

describe('FloatingCornerButton', () => {
  it('renders children inside the button and keeps it clickable', () => {
    const onClick = vi.fn();

    render(
      <FloatingCornerButton onClick={onClick}>
        <span data-testid="content">Action</span>
      </FloatingCornerButton>,
    );

    const button = screen.getByRole('button');
    expect(button).toHaveClass('floating-corner-button');

    const content = screen.getByTestId('content');
    expect(content).toHaveTextContent('Action');

    fireEvent.click(button);
    expect(onClick).toHaveBeenCalledTimes(1);
  });

  it('forwards refs to the underlying button element and merges class names', () => {
    const ref = createRef<HTMLButtonElement>();

    const { container } = render(
      <FloatingCornerButton ref={ref} className="test-class">
        Focus
      </FloatingCornerButton>,
    );

    const button = container.querySelector('button');
    expect(button).not.toBeNull();
    if (!button) {
      throw new Error('Expected a button element');
    }

    expect(ref.current).toBe(button);
    expect(button).toHaveAttribute('type', 'button');
    expect(button.className.split(' ')).toContain('test-class');
    expect(button.className.split(' ')).toContain('floating-corner-button--top-right');
    expect(button.className.split(' ')).toContain('floating-corner-button--strategy-absolute');
  });

  it.each(placements)(
    'applies placement specific class when placement is %s',
    (placement: Placement) => {
      render(
        <FloatingCornerButton placement={placement}>
          Place me
        </FloatingCornerButton>,
      );

      const button = screen.getByRole('button');
      expect(button.className.split(' ')).toContain(
        `floating-corner-button--${placement}`,
      );
    },
  );

  it('applies fixed strategy modifier when requested', () => {
    render(
      <FloatingCornerButton strategy="fixed">Pin me</FloatingCornerButton>,
    );

    const button = screen.getByRole('button');
    expect(button.className.split(' ')).toContain(
      'floating-corner-button--strategy-fixed',
    );
  });

  it('exposes a data-state flag when active', () => {
    render(
      <FloatingCornerButton isActive>Toggle</FloatingCornerButton>,
    );

    const button = screen.getByRole('button');
    expect(button).toHaveAttribute('data-state', 'active');
  });

  it('renders as an anchor without forcing a button type attribute', () => {
    render(
      <FloatingCornerButton as="a" href="#dest">
        Link to somewhere
      </FloatingCornerButton>,
    );

    const anchor = screen.getByRole('link');
    expect(anchor.tagName.toLowerCase()).toBe('a');
    expect(anchor).not.toHaveAttribute('type');
    expect(anchor).toHaveAttribute('href', '#dest');
  });
});
