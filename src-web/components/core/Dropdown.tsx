import classNames from 'classnames';
import { motion } from 'framer-motion';
import type {
  CSSProperties,
  FocusEvent as ReactFocusEvent,
  HTMLAttributes,
  MouseEvent,
  ReactElement,
  ReactNode,
  SetStateAction,
} from 'react';
import React, {
  Children,
  cloneElement,
  forwardRef,
  useCallback,
  useEffect,
  useImperativeHandle,
  useMemo,
  useRef,
  useState,
} from 'react';
import { useKey, useWindowSize } from 'react-use';
import type { HotkeyAction } from '../../hooks/useHotKey';
import { useHotKey } from '../../hooks/useHotKey';
import { useStateWithDeps } from '../../hooks/useStateWithDeps';
import { getNodeText } from '../../lib/getNodeText';
import { Overlay } from '../Overlay';
import { Button } from './Button';
import { HotKey } from './HotKey';
import { Icon } from './Icon';
import { Separator } from './Separator';
import { HStack, VStack } from './Stacks';

export type DropdownItemSeparator = {
  type: 'separator';
  label?: string;
};

export type DropdownItemDefault = {
  key: string;
  type?: 'default';
  label: ReactNode;
  keepOpen?: boolean;
  hotKeyAction?: HotkeyAction;
  hotKeyLabelOnly?: boolean;
  variant?: 'default' | 'danger' | 'notify';
  disabled?: boolean;
  hidden?: boolean;
  leftSlot?: ReactNode;
  rightSlot?: ReactNode;
  onSelect?: () => void;
};

export type DropdownItem = DropdownItemDefault | DropdownItemSeparator;

export interface DropdownProps {
  children: ReactElement<HTMLAttributes<HTMLButtonElement>>;
  items: DropdownItem[];
  onOpen?: () => void;
  onClose?: () => void;
  hotKeyAction?: HotkeyAction;
}

export interface DropdownRef {
  isOpen: boolean;
  open: () => void;
  toggle: () => void;
  close?: () => void;
  next?: () => void;
  prev?: () => void;
  select?: () => void;
}

export const Dropdown = forwardRef<DropdownRef, DropdownProps>(function Dropdown(
  { children, items, onOpen, onClose, hotKeyAction }: DropdownProps,
  ref,
) {
  const [isOpen, _setIsOpen] = useState<boolean>(false);
  const [defaultSelectedIndex, setDefaultSelectedIndex] = useState<number>();
  const buttonRef = useRef<HTMLButtonElement>(null);
  const menuRef = useRef<Omit<DropdownRef, 'open'>>(null);

  const setIsOpen = useCallback(
    (o: SetStateAction<boolean>) => {
      _setIsOpen(o);
      if (o) onOpen?.();
      else onClose?.();
    },
    [onClose, onOpen],
  );

  const handleClose = useCallback(() => {
    setIsOpen(false);
    buttonRef.current?.focus();
    // Reset so it triggers a render if opening sets to 0, for example
    setDefaultSelectedIndex(undefined);
  }, [setIsOpen]);

  useImperativeHandle(ref, () => ({
    ...menuRef.current,
    isOpen: isOpen,
    toggle() {
      if (!isOpen) this.open();
      else this.close();
    },
    open() {
      setIsOpen(true);
    },
    close() {
      handleClose();
    },
  }));

  useHotKey(hotKeyAction ?? null, () => {
    setDefaultSelectedIndex(0);
    setIsOpen(true);
  });

  const child = useMemo(() => {
    const existingChild = Children.only(children);
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const props: any = {
      ...existingChild.props,
      ref: buttonRef,
      'aria-haspopup': 'true',
      onClick:
        existingChild.props?.onClick ??
        ((e: MouseEvent<HTMLButtonElement>) => {
          e.preventDefault();
          e.stopPropagation();
          setDefaultSelectedIndex(undefined);
          setIsOpen((o) => !o);
        }),
    };
    return cloneElement(existingChild, props);
  }, [children, setIsOpen]);

  useEffect(() => {
    buttonRef.current?.setAttribute('aria-expanded', isOpen.toString());
  }, [isOpen]);

  const windowSize = useWindowSize();
  const triggerRect = useMemo(() => {
    if (!windowSize) return null; // No-op to TS happy with this dep
    if (!isOpen) return null;
    return buttonRef.current?.getBoundingClientRect();
  }, [isOpen, windowSize]);

  return (
    <>
      {child}
      <Menu
        ref={menuRef}
        showTriangle
        defaultSelectedIndex={defaultSelectedIndex}
        items={items}
        triggerShape={triggerRect ?? null}
        onClose={handleClose}
        isOpen={isOpen}
      />
    </>
  );
});

interface ContextMenuProps {
  show: { x: number; y: number } | null;
  className?: string;
  items: DropdownProps['items'];
  onClose: () => void;
}

export const ContextMenu = forwardRef<DropdownRef, ContextMenuProps>(function ContextMenu(
  { show, className, items, onClose },
  ref,
) {
  const triggerShape = useMemo(
    () => ({
      top: show?.y ?? 0,
      bottom: show?.y ?? 0,
      left: show?.x ?? 0,
      right: show?.x ?? 0,
    }),
    [show],
  );

  return (
    <Menu
      className={className}
      ref={ref}
      items={items}
      isOpen={show != null}
      onClose={onClose}
      triggerShape={triggerShape}
    />
  );
});

interface MenuProps {
  className?: string;
  defaultSelectedIndex?: number;
  items: DropdownProps['items'];
  triggerShape: Pick<DOMRect, 'top' | 'bottom' | 'left' | 'right'> | null;
  onClose: () => void;
  showTriangle?: boolean;
  isOpen: boolean;
}

const Menu = forwardRef<Omit<DropdownRef, 'open' | 'isOpen' | 'toggle'>, MenuProps>(function Menu(
  {
    className,
    isOpen,
    items,
    onClose,
    triggerShape,
    defaultSelectedIndex,
    showTriangle,
  }: MenuProps,
  ref,
) {
  const [selectedIndex, setSelectedIndex] = useStateWithDeps<number | null>(
    defaultSelectedIndex ?? null,
    [defaultSelectedIndex],
  );
  const [menuStyles, setMenuStyles] = useState<CSSProperties>({});
  const [filter, setFilter] = useState<string>('');
  const [containerWidth, setContainerWidth] = useState<number | null>(null);

  // Calculate the max height so we can scroll
  const initMenu = useCallback((el: HTMLDivElement | null) => {
    if (el === null) return {};
    const windowBox = document.documentElement.getBoundingClientRect();
    const menuBox = el.getBoundingClientRect();
    setMenuStyles({ maxHeight: windowBox.height - menuBox.top - 5 });
  }, []);

  const handleClose = useCallback(() => {
    onClose();
    setSelectedIndex(null);
    setFilter('');
  }, [onClose, setSelectedIndex]);

  // Close menu on space bar
  const handleMenuKeyDown = (e: React.KeyboardEvent<HTMLDivElement>) => {
    const isCharacter = e.key.length === 1;
    const isSpecial = e.ctrlKey || e.metaKey || e.altKey;
    if (isCharacter && !isSpecial) {
      e.preventDefault();
      setFilter((f) => (f + e.key).trim());
      setSelectedIndex(0);
    } else if (e.key === 'Backspace' && !isSpecial) {
      e.preventDefault();
      setFilter((f) => f.slice(0, -1));
    }
  };

  useKey(
    'Escape',
    () => {
      if (!isOpen) return;
      if (filter !== '') setFilter('');
      else handleClose();
    },
    {},
    [isOpen, filter, setFilter, handleClose],
  );

  const handlePrev = useCallback(() => {
    setSelectedIndex((currIndex) => {
      let nextIndex = (currIndex ?? 0) - 1;
      const maxTries = items.length;
      for (let i = 0; i < maxTries; i++) {
        if (items[nextIndex]?.type === 'separator') {
          nextIndex--;
        } else if (nextIndex < 0) {
          nextIndex = items.length - 1;
        } else {
          break;
        }
      }
      return nextIndex;
    });
  }, [items, setSelectedIndex]);

  const handleNext = useCallback(() => {
    setSelectedIndex((currIndex) => {
      let nextIndex = (currIndex ?? -1) + 1;
      const maxTries = items.length;
      for (let i = 0; i < maxTries; i++) {
        if (items[nextIndex]?.type === 'separator') {
          nextIndex++;
        } else if (nextIndex >= items.length) {
          nextIndex = 0;
        } else {
          break;
        }
      }
      return nextIndex;
    });
  }, [items, setSelectedIndex]);

  useKey(
    'ArrowUp',
    (e) => {
      if (!isOpen) return;
      e.preventDefault();
      handlePrev();
    },
    {},
    [isOpen],
  );

  useKey(
    'ArrowDown',
    (e) => {
      if (!isOpen) return;
      e.preventDefault();
      handleNext();
    },
    {},
    [isOpen],
  );

  const handleSelect = useCallback(
    (i: DropdownItem) => {
      if (i.type !== 'separator' && !i.keepOpen) {
        handleClose();
      }
      setSelectedIndex(null);
      if (i.type !== 'separator' && typeof i.onSelect === 'function') {
        i.onSelect();
      }
    },
    [handleClose, setSelectedIndex],
  );

  useImperativeHandle(
    ref,
    () => ({
      close: handleClose,
      prev: handlePrev,
      next: handleNext,
      select: () => {
        const item = items[selectedIndex ?? -1] ?? null;
        if (!item) return;
        handleSelect(item);
      },
    }),
    [handleClose, handleNext, handlePrev, handleSelect, items, selectedIndex],
  );

  const initContainerRef = (n: HTMLDivElement | null) => {
    if (n == null) return null;
    setContainerWidth(n.offsetWidth);
  };

  const { containerStyles, triangleStyles } = useMemo<{
    containerStyles: CSSProperties;
    triangleStyles: CSSProperties | null;
  }>(() => {
    if (triggerShape == null) return { containerStyles: {}, triangleStyles: null };

    const docRect = document.documentElement.getBoundingClientRect();
    const width = triggerShape.right - triggerShape.left;
    const heightAbove = triggerShape.top;
    const heightBelow = docRect.height - triggerShape.bottom;
    const hSpaceRemaining = docRect.width - triggerShape.left;
    const top = triggerShape?.bottom + 5;
    const onRight = hSpaceRemaining < 200;
    const upsideDown = heightAbove > heightBelow && heightBelow < 200;
    const containerStyles = {
      top: !upsideDown ? top : undefined,
      bottom: upsideDown ? docRect.height - top : undefined,
      right: onRight ? docRect.width - triggerShape?.right : undefined,
      left: !onRight ? triggerShape?.left : undefined,
      width: containerWidth ?? 'auto',
    };
    const size = { top: '-0.2rem', width: '0.4rem', height: '0.4rem' };
    const triangleStyles = onRight
      ? { right: width / 2, marginRight: '-0.2rem', ...size }
      : { left: width / 2, marginLeft: '-0.2rem', ...size };
    return { containerStyles, triangleStyles };
  }, [triggerShape, containerWidth]);

  const filteredItems = useMemo(
    () => items.filter((i) => getNodeText(i.label).toLowerCase().includes(filter.toLowerCase())),
    [items, filter],
  );

  const handleFocus = useCallback(
    (i: DropdownItem) => {
      const index = filteredItems.findIndex((item) => item === i) ?? null;
      setSelectedIndex(index);
    },
    [filteredItems, setSelectedIndex],
  );

  if (items.length === 0) return null;

  return (
    <>
      {filteredItems.map(
        (item) =>
          item.type !== 'separator' &&
          !item.hotKeyLabelOnly && (
            <MenuItemHotKey
              key={item.key}
              onSelect={handleSelect}
              item={item}
              action={item.hotKeyAction}
            />
          ),
      )}
      {isOpen && (
        <Overlay open variant="transparent" portalName="dropdown" zIndex={50}>
          <div className="x-theme-dialog">
            <div tabIndex={-1} aria-hidden className="fixed inset-0 z-30" onClick={handleClose} />
            <motion.div
              tabIndex={0}
              onKeyDown={handleMenuKeyDown}
              initial={{ opacity: 0, y: -5, scale: 0.98 }}
              animate={{ opacity: 1, y: 0, scale: 1 }}
              role="menu"
              aria-orientation="vertical"
              dir="ltr"
              ref={initContainerRef}
              style={containerStyles}
              className={classNames(className, 'outline-none my-1 pointer-events-auto fixed z-50')}
            >
              {triangleStyles && showTriangle && (
                <span
                  aria-hidden
                  style={triangleStyles}
                  className="bg-background absolute rotate-45 border-background-highlight border-t border-l"
                />
              )}
              {containerStyles && (
                <VStack
                  ref={initMenu}
                  style={menuStyles}
                  className={classNames(
                    className,
                    'h-auto bg-background rounded-md shadow-lg py-1.5 border',
                    'border-background-highlight overflow-auto mb-1 mx-0.5',
                  )}
                >
                  {filter && (
                    <HStack
                      space={2}
                      alignItems="center"
                      className="pb-0.5 px-1.5 mb-2 text-xs border border-background-highlight-secondary mx-2 rounded font-mono h-2xs"
                    >
                      <Icon icon="search" size="xs" className="text-fg-subtle" />
                      <div className="text-fg">{filter}</div>
                    </HStack>
                  )}
                  {filteredItems.length === 0 && (
                    <span className="text-fg-subtler text-sm text-center px-2 py-1">
                      No matches
                    </span>
                  )}
                  {filteredItems.map((item, i) => {
                    if (item.type === 'separator') {
                      return (
                        <Separator key={i} className={classNames('my-1.5', item.label && 'ml-2')}>
                          {item.label}
                        </Separator>
                      );
                    }
                    if (item.hidden) {
                      return null;
                    }
                    return (
                      <MenuItem
                        focused={i === selectedIndex}
                        onFocus={handleFocus}
                        onSelect={handleSelect}
                        key={item.key}
                        item={item}
                      />
                    );
                  })}
                </VStack>
              )}
            </motion.div>
          </div>
        </Overlay>
      )}
    </>
  );
});

interface MenuItemProps {
  className?: string;
  item: DropdownItemDefault;
  onSelect: (item: DropdownItemDefault) => void;
  onFocus: (item: DropdownItemDefault) => void;
  focused: boolean;
}

function MenuItem({ className, focused, onFocus, item, onSelect, ...props }: MenuItemProps) {
  const handleClick = useCallback(() => onSelect?.(item), [item, onSelect]);
  const handleFocus = useCallback(
    (e: ReactFocusEvent<HTMLButtonElement>) => {
      e.stopPropagation(); // Don't trigger focus on any parents
      return onFocus?.(item);
    },
    [item, onFocus],
  );

  const initRef = useCallback(
    (el: HTMLButtonElement | null) => {
      if (el === null) return;
      if (focused) {
        setTimeout(() => el.focus(), 0);
      }
    },
    [focused],
  );

  const rightSlot = item.rightSlot ?? <HotKey action={item.hotKeyAction ?? null} />;

  return (
    <Button
      ref={initRef}
      size="sm"
      tabIndex={-1}
      onMouseEnter={(e) => e.currentTarget.focus()}
      onMouseLeave={(e) => e.currentTarget.blur()}
      disabled={item.disabled}
      onFocus={handleFocus}
      onClick={handleClick}
      justify="start"
      leftSlot={item.leftSlot && <div className="pr-2 flex justify-start">{item.leftSlot}</div>}
      rightSlot={rightSlot && <div className="ml-auto pl-3">{rightSlot}</div>}
      innerClassName="!text-left"
      color="custom"
      className={classNames(
        className,
        'h-xs', // More compact
        'min-w-[8rem] outline-none px-2 mx-1.5 flex text-sm whitespace-nowrap',
        'focus:bg-background-highlight focus:text-fg rounded',
        item.variant === 'default' && 'text-fg-subtle',
        item.variant === 'danger' && 'text-fg-danger',
        item.variant === 'notify' && 'text-fg-primary',
      )}
      {...props}
    >
      <div
        className={classNames(
          // Add padding on right when no right slot, for some visual balance
          !item.rightSlot && 'pr-4',
        )}
      >
        {item.label}
      </div>
    </Button>
  );
}

interface MenuItemHotKeyProps {
  action: HotkeyAction | undefined;
  onSelect: MenuItemProps['onSelect'];
  item: MenuItemProps['item'];
}

function MenuItemHotKey({ action, onSelect, item }: MenuItemHotKeyProps) {
  useHotKey(action ?? null, () => onSelect(item));
  return null;
}
