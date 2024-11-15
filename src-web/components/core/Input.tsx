import classNames from 'classnames';
import type { EditorView } from 'codemirror';
import type { HTMLAttributes, ReactNode } from 'react';
import { forwardRef, useCallback, useMemo, useRef, useState } from 'react';
import { useStateWithDeps } from '../../hooks/useStateWithDeps';
import type { EditorProps } from './Editor';
import { Editor } from './Editor';
import { IconButton } from './IconButton';
import { HStack } from './Stacks';

export type InputProps = Omit<
  HTMLAttributes<HTMLInputElement>,
  'onChange' | 'onFocus' | 'onKeyDown' | 'onPaste'
> &
  Pick<
    EditorProps,
    | 'language'
    | 'useTemplating'
    | 'autocomplete'
    | 'forceUpdateKey'
    | 'autoFocus'
    | 'autoSelect'
    | 'autocompleteVariables'
    | 'onKeyDown'
    | 'readOnly'
  > & {
    name?: string;
    type?: 'text' | 'password';
    label: ReactNode;
    hideLabel?: boolean;
    labelPosition?: 'top' | 'left';
    labelClassName?: string;
    containerClassName?: string;
    onChange?: (value: string) => void;
    onFocus?: () => void;
    onBlur?: () => void;
    onPaste?: (value: string) => void;
    onPasteOverwrite?: (value: string) => void;
    defaultValue?: string;
    leftSlot?: ReactNode;
    rightSlot?: ReactNode;
    size?: 'xs' | 'sm' | 'md' | 'auto';
    className?: string;
    placeholder?: string;
    validate?: boolean | ((v: string) => boolean);
    require?: boolean;
    wrapLines?: boolean;
  };

export const Input = forwardRef<EditorView | undefined, InputProps>(function Input(
  {
    className,
    containerClassName,
    defaultValue,
    forceUpdateKey,
    hideLabel,
    label,
    labelClassName,
    labelPosition = 'top',
    leftSlot,
    onBlur,
    onChange,
    onFocus,
    onPaste,
    onPasteOverwrite,
    placeholder,
    require,
    rightSlot,
    wrapLines,
    size = 'md',
    type = 'text',
    validate,
    readOnly,
    ...props
  }: InputProps,
  ref,
) {
  const [obscured, setObscured] = useStateWithDeps(type === 'password', [type]);
  const [currentValue, setCurrentValue] = useState(defaultValue ?? '');
  const [focused, setFocused] = useState(false);

  const handleFocus = useCallback(() => {
    if (readOnly) return;
    setFocused(true);
    onFocus?.();
  }, [onFocus, readOnly]);

  const handleBlur = useCallback(() => {
    setFocused(false);
    onBlur?.();
  }, [onBlur]);

  const id = `input-${label}`;
  const editorClassName = classNames(
    className,
    '!bg-transparent min-w-0 h-auto w-full focus:outline-none placeholder:text-placeholder',
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

  // Submit nearest form on Enter key press
  const handleKeyDown = useCallback(
    (e: KeyboardEvent) => {
      if (e.key !== 'Enter') return;

      const form = wrapperRef.current?.closest('form');
      if (!isValid || form == null) return;

      form?.dispatchEvent(new Event('submit', { cancelable: true, bubbles: true }));
    },
    [isValid],
  );

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
          'text-text-subtle whitespace-nowrap',
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
          focused ? 'border-border-focus' : 'border-border',
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
          <Editor
            ref={ref}
            id={id}
            singleLine
            wrapLines={wrapLines}
            onKeyDown={handleKeyDown}
            type={type === 'password' && !obscured ? 'text' : type}
            defaultValue={defaultValue}
            forceUpdateKey={forceUpdateKey}
            placeholder={placeholder}
            onChange={handleChange}
            onPaste={onPaste}
            onPasteOverwrite={onPasteOverwrite}
            className={editorClassName}
            onFocus={handleFocus}
            onBlur={handleBlur}
            readOnly={readOnly}
            {...props}
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
});

function validateRequire(v: string) {
  return v.length > 0;
}
