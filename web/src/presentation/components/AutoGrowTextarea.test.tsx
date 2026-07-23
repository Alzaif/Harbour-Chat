import { cleanup, fireEvent, render, screen } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { AutoGrowTextarea } from './AutoGrowTextarea';

afterEach(() => cleanup());

describe('AutoGrowTextarea', () => {
  it('renders a textarea (so content wraps and grows vertically)', () => {
    render(<AutoGrowTextarea value="hello" onChange={vi.fn()} ariaLabel="Message" />);
    const el = screen.getByLabelText('Message');
    expect(el.tagName).toBe('TEXTAREA');
  });

  it('reports edits through onChange', () => {
    const onChange = vi.fn();
    render(<AutoGrowTextarea value="" onChange={onChange} ariaLabel="Message" />);
    fireEvent.change(screen.getByLabelText('Message'), { target: { value: 'hi there' } });
    expect(onChange).toHaveBeenCalledWith('hi there');
  });

  it('forwards key events for Enter-to-send handling', () => {
    const onKeyDown = vi.fn();
    render(<AutoGrowTextarea value="hi" onChange={vi.fn()} ariaLabel="Message" onKeyDown={onKeyDown} />);
    fireEvent.keyDown(screen.getByLabelText('Message'), { key: 'Enter' });
    expect(onKeyDown).toHaveBeenCalled();
  });
});
