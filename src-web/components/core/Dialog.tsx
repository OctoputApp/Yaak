import classNames from 'classnames';
import { motion } from 'framer-motion';
import type { ReactNode } from 'react';
import { useMemo } from 'react';
import { useKey } from 'react-use';
import { Overlay } from '../Overlay';
import { Heading } from './Heading';
import { IconButton } from './IconButton';

export interface DialogProps {
  children: ReactNode;
  open: boolean;
  onClose: () => void;
  title?: ReactNode;
  description?: ReactNode;
  className?: string;
  size?: 'sm' | 'md' | 'lg' | 'full' | 'dynamic';
  hideX?: boolean;
  noPadding?: boolean;
  noScroll?: boolean;
}

export function Dialog({
  children,
  className,
  size = 'full',
  open,
  onClose,
  title,
  description,
  hideX,
  noPadding,
  noScroll,
}: DialogProps) {
  const titleId = useMemo(() => Math.random().toString(36).slice(2), []);
  const descriptionId = useMemo(
    () => (description ? Math.random().toString(36).slice(2) : undefined),
    [description],
  );

  useKey(
    'Escape',
    () => {
      if (!open) return;
      onClose();
    },
    {},
    [open],
  );

  return (
    <Overlay open={open} onClose={onClose} portalName="dialog">
      <div className="absolute inset-0 flex items-center justify-center pointer-events-none">
        <div
          role="dialog"
          aria-labelledby={titleId}
          aria-describedby={descriptionId}
          className="pointer-events-auto"
        >
          <motion.div
            initial={{ top: 5, scale: 0.97 }}
            animate={{ top: 0, scale: 1 }}
            className={classNames(
              className,
              'grid grid-rows-[auto_auto_minmax(0,1fr)]',
              'relative bg-gray-50 pointer-events-auto',
              'rounded-lg',
              'dark:border border-highlight shadow shadow-black/10',
              'max-w-[calc(100vw-5rem)] max-h-[calc(100vh-6rem)]',
              size === 'sm' && 'w-[25rem] max-h-[80vh]',
              size === 'md' && 'w-[45rem] max-h-[80vh]',
              size === 'lg' && 'w-[65rem] max-h-[80vh]',
              size === 'full' && 'w-[100vw] h-[100vh]',
              size === 'dynamic' && 'min-w-[30vw] max-w-[80vw]',
            )}
          >
            {title ? (
              <Heading className="px-6 mt-4 mb-2" size={1} id={titleId}>
                {title}
              </Heading>
            ) : (
              <span />
            )}

            {description ? (
              <p className="px-6 text-fg-subtle" id={descriptionId}>
                {description}
              </p>
            ) : (
              <span />
            )}

            <div
              className={classNames(
                'h-full w-full grid grid-cols-[minmax(0,1fr)]',
                !noPadding && 'px-6 py-2',
                !noScroll && 'overflow-y-auto',
              )}
            >
              {children}
            </div>

            {/*Put close at the end so that it's the last thing to be tabbed to*/}
            {!hideX && (
              <div className="ml-auto absolute right-1 top-1">
                <IconButton
                  className="opacity-70 hover:opacity-100"
                  onClick={onClose}
                  title="Close dialog (Esc)"
                  aria-label="Close"
                  size="sm"
                  icon="x"
                />
              </div>
            )}
          </motion.div>
        </div>
      </div>
    </Overlay>
  );
}
