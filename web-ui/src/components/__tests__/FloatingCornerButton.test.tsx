import { createRef } from 'react';

import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

import { FloatingCornerButton } from '../FloatingCornerButton';

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
  });
});
