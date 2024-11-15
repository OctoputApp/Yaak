import classNames from 'classnames';
import type { CSSProperties, MouseEvent as ReactMouseEvent } from 'react';
import React from 'react';

interface ResizeBarProps {
  style?: CSSProperties;
  className?: string;
  isResizing: boolean;
  onResizeStart: (e: ReactMouseEvent<HTMLDivElement>) => void;
  onReset?: () => void;
  side: 'left' | 'right' | 'top';
  justify: 'center' | 'end' | 'start';
}

export function ResizeHandle({
  style,
  justify,
  className,
  onResizeStart,
  onReset,
  isResizing,
  side,
}: ResizeBarProps) {
  const vertical = side === 'top';
  return (
    <div
      aria-hidden
      draggable
      style={style}
      onDragStart={onResizeStart}
      onDoubleClick={onReset}
      className={classNames(
        className,
        'group z-10 flex select-none',
        // 'bg-info', // For debugging
        vertical ? 'w-full h-3 cursor-row-resize' : 'h-full w-3 cursor-col-resize',
        justify === 'center' && 'justify-center',
        justify === 'end' && 'justify-end',
        justify === 'start' && 'justify-start',
        side === 'right' && 'right-0',
        side === 'left' && 'left-0',
        side === 'top' && 'top-0',
      )}
    >
      {/* Show global overlay with cursor style to ensure cursor remains the same when moving quickly */}
      {isResizing && (
        <div
          className={classNames(
            'fixed -left-20 -right-20 -top-20 -bottom-20',
            vertical && 'cursor-row-resize',
            !vertical && 'cursor-col-resize',
          )}
        />
      )}
    </div>
  );
}
