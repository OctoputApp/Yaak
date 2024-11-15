import classNames from 'classnames';
import type { HTMLAttributes } from 'react';
import { useCallback, useMemo, useRef, useState } from 'react';
import { useStateWithDeps } from '../../hooks/useStateWithDeps';
import { IconButton } from './IconButton';
import type { InputProps } from './Input';
import { HStack } from './Stacks';

export type PlainInputProps = Omit<InputProps, 'wrapLines' | 'onKeyDown' | 'type'> &
  Pick<HTMLAttributes<HTMLInputElement>, 'onKeyDownCapture'> & {
    type?: 'text' | 'password' | 'number';
    step?: number;
  };

export function PlainInput({
  className,
  containerClassName,
  defaultValue,
  forceUpdateKey,
  hideLabel,
  label,
  labelClassName,
  labelPosition = 'top',
  leftSlot,
  name,
  onBlur,
  onChange,
  onFocus,
  onPaste,
  placeholder,
  require,
  rightSlot,
  size = 'md',
  type = 'text',
  validate,
  autoSelect,
  step,
  autoFocus,
  readOnly,
}: PlainInputProps) {
  const [obscured, setObscured] = useStateWithDeps(type === 'password', [type]);
  const [currentValue, setCurrentValue] = useState(defaultValue ?? '');
  const [focused, setFocused] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  const handleFocus = useCallback(() => {
    setFocused(true);
    if (autoSelect) {
      inputRef.current?.select();
      textareaRef.current?.select();
    }
    onFocus?.();
  }, [autoSelect, onFocus]);

  const handleBlur = useCallback(() => {
    setFocused(false);
    onBlur?.();
  }, [onBlur]);

  const id = `input-${name}`;
  const commonClassName = classNames(
    className,
    '!bg-transparent min-w-0 w-full focus:outline-none placeholder:text-placeholder',
    'px-2 text-xs font-mono cursor-text',
  );

  const isValid = useMemo(() => {
    if (require && !validateRequire(currentValue)) return false;
    if (typeof validate === 'boolean') return validate;
    if (typeof validate === 'function' && !validate(currentValue)) return false;
    return true;
  }, [require, currentValue, validate]);

  const handleChange = useCallback(
    (value: string) => {
      setCurrentValue(value);
      onChange?.(value);
    },
    [onChange],
  );

  const wrapperRef = useRef<HTMLDivElement>(null);

  return (
    <div
      ref={wrapperRef}
      className={classNames(
        'w-full',
        'pointer-events-auto', // Just in case we're placing in disabled parent
        labelPosition === 'left' && 'flex items-center gap-2',
        labelPosition === 'top' && 'flex-row gap-0.5',
      )}
    >
      <label
        htmlFor={id}
        className={classNames(
          labelClassName,
          'text-text-subtle whitespace-nowrap flex-shrink-0',
          hideLabel && 'sr-only',
        )}
      >
        {label}
      </label>
      <HStack
        alignItems="stretch"
        className={classNames(
          containerClassName,
          'x-theme-input',
          'relative w-full rounded-md text',
          'border',
          focused ? 'border-border-focus' : 'border-border-subtle',
          !isValid && '!border-danger',
          size === 'md' && 'min-h-md',
          size === 'sm' && 'min-h-sm',
          size === 'xs' && 'min-h-xs',
        )}
      >
        {leftSlot}
        <HStack
          className={classNames(
            'w-full min-w-0',
            leftSlot && 'pl-0.5 -ml-2',
            rightSlot && 'pr-0.5 -mr-2',
          )}
        >
          <input
            ref={inputRef}
            key={forceUpdateKey}
            id={id}
            type={type === 'password' && !obscured ? 'text' : type}
            defaultValue={defaultValue}
            placeholder={placeholder}
            autoComplete="off"
            autoCapitalize="off"
            autoCorrect="off"
            onChange={(e) => handleChange(e.target.value)}
            onPaste={(e) => onPaste?.(e.clipboardData.getData('Text'))}
            className={classNames(commonClassName, 'h-auto')}
            onFocus={handleFocus}
            onBlur={handleBlur}
            autoFocus={autoFocus}
            step={step}
            readOnly={readOnly}
          />
        </HStack>
        {type === 'password' && (
          <IconButton
            title={obscured ? `Show ${label}` : `Obscure ${label}`}
            size="xs"
            className="mr-0.5 group/obscure !h-auto my-0.5"
            iconClassName="text-text-subtle group-hover/obscure:text"
            iconSize="sm"
            icon={obscured ? 'eye' : 'eye_closed'}
            onClick={() => setObscured((o) => !o)}
          />
        )}
        {rightSlot}
      </HStack>
    </div>
  );
}

function validateRequire(v: string) {
  return v.length > 0;
}
