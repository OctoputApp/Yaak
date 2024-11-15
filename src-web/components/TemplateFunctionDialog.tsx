import type { Folder, HttpRequest } from '@yaakapp-internal/models';
import type {
  TemplateFunction,
  TemplateFunctionArg,
  TemplateFunctionCheckboxArg,
  TemplateFunctionFileArg,
  TemplateFunctionHttpRequestArg,
  TemplateFunctionSelectArg,
  TemplateFunctionTextArg,
} from '@yaakapp-internal/plugin';
import type { FnArg, Tokens } from '@yaakapp-internal/template';
import classNames from 'classnames';
import { useCallback, useMemo, useState } from 'react';
import { useActiveRequest } from '../hooks/useActiveRequest';
import { useDebouncedValue } from '../hooks/useDebouncedValue';
import { useFolders } from '../hooks/useFolders';
import { useHttpRequests } from '../hooks/useHttpRequests';
import { useRenderTemplate } from '../hooks/useRenderTemplate';
import { useTemplateTokensToString } from '../hooks/useTemplateTokensToString';
import { resolvedModelName } from '../lib/resolvedModelName';
import { Button } from './core/Button';
import { Checkbox } from './core/Checkbox';
import { InlineCode } from './core/InlineCode';
import { PlainInput } from './core/PlainInput';
import { Select } from './core/Select';
import { VStack } from './core/Stacks';
import { SelectFile } from './SelectFile';

const NULL_ARG = '__NULL__';

interface Props {
  templateFunction: TemplateFunction;
  initialTokens: Tokens;
  hide: () => void;
  onChange: (insert: string) => void;
}

export function TemplateFunctionDialog({ templateFunction, hide, initialTokens, onChange }: Props) {
  const [argValues, setArgValues] = useState<Record<string, string | boolean>>(() => {
    const initial: Record<string, string> = {};
    const initialArgs =
      initialTokens.tokens[0]?.type === 'tag' && initialTokens.tokens[0]?.val.type === 'fn'
        ? initialTokens.tokens[0]?.val.args
        : [];
    for (const arg of templateFunction.args) {
      const initialArg = initialArgs.find((a) => a.name === arg.name);
      const initialArgValue =
        initialArg?.value.type === 'str'
          ? initialArg?.value.text
          : // TODO: Implement variable-based args
            '__NULL__';
      initial[arg.name] = initialArgValue ?? NULL_ARG;
    }

    return initial;
  });

  const setArgValue = useCallback((name: string, value: string | boolean | null) => {
    setArgValues((v) => ({ ...v, [name]: value == null ? '__NULL__' : value }));
  }, []);

  const tokens: Tokens = useMemo(() => {
    const argTokens: FnArg[] = Object.keys(argValues).map((name) => ({
      name,
      value:
        argValues[name] === NULL_ARG
          ? { type: 'null' }
          : typeof argValues[name] === 'boolean'
            ? { type: 'bool', value: argValues[name] === true }
            : { type: 'str', text: String(argValues[name] ?? '') },
    }));

    return {
      tokens: [
        {
          type: 'tag',
          val: {
            type: 'fn',
            name: templateFunction.name,
            args: argTokens,
          },
        },
      ],
    };
  }, [argValues, templateFunction.name]);

  const tagText = useTemplateTokensToString(tokens);

  const handleDone = () => {
    if (tagText.data) {
      onChange(tagText.data);
    }
    hide();
  };

  const debouncedTagText = useDebouncedValue(tagText.data ?? '', 200);
  const rendered = useRenderTemplate(debouncedTagText);
  const tooLarge = (rendered.data ?? '').length > 10000;

  return (
    <VStack className="pb-3" space={4}>
      <h1 className="font-mono !text-base">{templateFunction.name}(…)</h1>
      <VStack space={2}>
        {templateFunction.args.map((a: TemplateFunctionArg, i: number) => {
          switch (a.type) {
            case 'select':
              return (
                <SelectArg
                  key={i}
                  arg={a}
                  onChange={(v) => setArgValue(a.name, v)}
                  value={argValues[a.name] ? String(argValues[a.name]) : '__ERROR__'}
                />
              );
            case 'text':
              return (
                <TextArg
                  key={i}
                  arg={a}
                  onChange={(v) => setArgValue(a.name, v)}
                  value={argValues[a.name] ? String(argValues[a.name]) : '__ERROR__'}
                />
              );
            case 'checkbox':
              return (
                <CheckboxArg
                  key={i}
                  arg={a}
                  onChange={(v) => setArgValue(a.name, v)}
                  value={argValues[a.name] !== undefined ? argValues[a.name] === true : false}
                />
              );
            case 'http_request':
              return (
                <HttpRequestArg
                  key={i}
                  arg={a}
                  onChange={(v) => setArgValue(a.name, v)}
                  value={argValues[a.name] ? String(argValues[a.name]) : '__ERROR__'}
                />
              );
            case 'file':
              return (
                <FileArg
                  key={i}
                  arg={a}
                  onChange={(v) => setArgValue(a.name, v)}
                  filePath={argValues[a.name] ? String(argValues[a.name]) : '__ERROR__'}
                />
              );
          }
        })}
      </VStack>
      <VStack className="w-full">
        <div className="text-sm text-text-subtle">Preview</div>
        <InlineCode
          className={classNames(
            'whitespace-pre select-text cursor-text max-h-[10rem] overflow-y-auto hide-scrollbars',
            tooLarge && 'italic text-danger',
          )}
        >
          {tooLarge ? 'too large to preview' : rendered.data || <>&nbsp;</>}
        </InlineCode>
      </VStack>
      <Button color="primary" onClick={handleDone}>
        Done
      </Button>
    </VStack>
  );
}

function TextArg({
  arg,
  onChange,
  value,
}: {
  arg: TemplateFunctionTextArg;
  value: string;
  onChange: (v: string) => void;
}) {
  const handleChange = useCallback(
    (value: string) => {
      onChange(value === '' ? NULL_ARG : value);
    },
    [onChange],
  );

  return (
    <PlainInput
      name={arg.name}
      onChange={handleChange}
      defaultValue={value === NULL_ARG ? '' : value}
      require={!arg.optional}
      label={
        <>
          {arg.label ?? arg.name}
          {arg.optional && <span> (optional)</span>}
        </>
      }
      hideLabel={arg.label == null}
      placeholder={arg.placeholder ?? arg.defaultValue ?? ''}
    />
  );
}

function SelectArg({
  arg,
  value,
  onChange,
}: {
  arg: TemplateFunctionSelectArg;
  value: string;
  onChange: (v: string) => void;
}) {
  return (
    <Select
      label={arg.label ?? arg.name}
      name={arg.name}
      onChange={onChange}
      value={value}
      options={[
        ...arg.options.map((a) => ({
          label: a.label + (arg.defaultValue === a.value ? ' (default)' : ''),
          value: a.value === arg.defaultValue ? NULL_ARG : a.value,
        })),
      ]}
    />
  );
}

function FileArg({
  arg,
  filePath,
  onChange,
}: {
  arg: TemplateFunctionFileArg;
  filePath: string;
  onChange: (v: string | null) => void;
}) {
  return (
    <SelectFile
      onChange={({ filePath }) => onChange(filePath)}
      filePath={filePath === '__NULL__' ? null : filePath}
      directory={!!arg.directory}
    />
  );
}

function HttpRequestArg({
  arg,
  value,
  onChange,
}: {
  arg: TemplateFunctionHttpRequestArg;
  value: string;
  onChange: (v: string) => void;
}) {
  const folders = useFolders();
  const httpRequests = useHttpRequests();
  const activeRequest = useActiveRequest();
  return (
    <Select
      label={arg.label ?? arg.name}
      name={arg.name}
      onChange={onChange}
      value={value}
      options={[
        ...httpRequests.map((r) => {
          return {
            label:
              buildRequestBreadcrumbs(r, folders).join(' / ') +
              (r.id == activeRequest?.id ? ' (current)' : ''),
            value: r.id,
          };
        }),
      ]}
    />
  );
}

function buildRequestBreadcrumbs(request: HttpRequest, folders: Folder[]): string[] {
  const ancestors: (HttpRequest | Folder)[] = [request];

  const next = () => {
    const latest = ancestors[0];
    if (latest == null) return [];

    const parent = folders.find((f) => f.id === latest.folderId);
    if (parent == null) return;

    ancestors.unshift(parent);
    next();
  };
  next();

  return ancestors.map(resolvedModelName);
}

function CheckboxArg({
  arg,
  onChange,
  value,
}: {
  arg: TemplateFunctionCheckboxArg;
  value: boolean;
  onChange: (v: boolean) => void;
}) {
  return (
    <Checkbox
      onChange={onChange}
      checked={value}
      title={arg.label ?? arg.name}
      hideLabel={arg.label == null}
    />
  );
}
