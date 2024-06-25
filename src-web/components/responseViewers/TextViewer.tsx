import classNames from 'classnames';
import type { ReactNode } from 'react';
import { useCallback, useMemo } from 'react';
import { createGlobalState } from 'react-use';
import { useContentTypeFromHeaders } from '../../hooks/useContentTypeFromHeaders';
import { useDebouncedValue } from '../../hooks/useDebouncedValue';
import { useFilterResponse } from '../../hooks/useFilterResponse';
import { useResponseBodyText } from '../../hooks/useResponseBodyText';
import { tryFormatJson, tryFormatXml } from '../../lib/formatters';
import type { HttpResponse } from '../../lib/models';
import { Editor } from '../core/Editor';
import { hyperlink } from '../core/Editor/hyperlink/extension';
import { IconButton } from '../core/IconButton';
import { Input } from '../core/Input';
import { EmptyStateText } from '../EmptyStateText';
import { BinaryViewer } from './BinaryViewer';

const extraExtensions = [hyperlink];

interface Props {
  response: HttpResponse;
  pretty: boolean;
  className?: string;
}

const useFilterText = createGlobalState<Record<string, string | null>>({});

export function TextViewer({ response, pretty, className }: Props) {
  const [filterTextMap, setFilterTextMap] = useFilterText();
  const filterText = filterTextMap[response.id] ?? null;
  const debouncedFilterText = useDebouncedValue(filterText, 200);
  const setFilterText = useCallback(
    (v: string | null) => {
      setFilterTextMap((m) => ({ ...m, [response.id]: v }));
    },
    [setFilterTextMap, response],
  );

  const contentType = useContentTypeFromHeaders(response.headers);
  const rawBody = useResponseBodyText(response);
  const isSearching = filterText != null;

  const filteredResponse = useFilterResponse({
    filter: debouncedFilterText ?? '',
    responseId: response.id,
  });

  const toggleSearch = useCallback(() => {
    if (isSearching) {
      setFilterText(null);
    } else {
      setFilterText('');
    }
  }, [isSearching, setFilterText]);

  const isJson = contentType?.includes('json');
  const isXml = contentType?.includes('xml') || contentType?.includes('html');
  const canFilter = isJson || isXml;

  const actions = useMemo<ReactNode[]>(() => {
    const result: ReactNode[] = [];

    if (!canFilter) return result;

    if (isSearching) {
      result.push(
        <div key="input" className="w-full !opacity-100">
          <Input
            key={response.id}
            validate={!filteredResponse.error}
            hideLabel
            autoFocus
            containerClassName="bg-background"
            size="sm"
            placeholder={isJson ? 'JSONPath expression' : 'XPath expression'}
            label="Filter expression"
            name="filter"
            defaultValue={filterText}
            onKeyDown={(e) => e.key === 'Escape' && toggleSearch()}
            onChange={setFilterText}
          />
        </div>,
      );
    }

    result.push(
      <IconButton
        key="icon"
        size="sm"
        icon={isSearching ? 'x' : 'filter'}
        title={isSearching ? 'Close filter' : 'Filter response'}
        onClick={toggleSearch}
        className={classNames(
          'bg-background border !border-background-highlight',
          isSearching && '!opacity-100',
        )}
      />,
    );

    return result;
  }, [
    canFilter,
    filterText,
    filteredResponse.error,
    isJson,
    isSearching,
    response.id,
    setFilterText,
    toggleSearch,
  ]);

  if (rawBody.isLoading) {
    return null;
  }

  if (rawBody.data == null) {
    return <BinaryViewer response={response} />;
  }

  if ((response.contentLength ?? 0) > 2 * 1000 * 1000) {
    return <EmptyStateText>Cannot preview text responses larger than 2MB</EmptyStateText>;
  }

  const formattedBody =
    pretty && contentType?.includes('json')
      ? tryFormatJson(rawBody.data)
      : pretty && contentType?.includes('xml')
      ? tryFormatXml(rawBody.data)
      : rawBody.data;

  let body;
  if (isSearching && filterText?.length > 0) {
    if (filteredResponse.error) {
      body = '';
    } else {
      body = filteredResponse.data ?? '';
    }
  } else {
    body = formattedBody;
  }

  return (
    <Editor
      readOnly
      className={className}
      forceUpdateKey={body}
      defaultValue={body}
      contentType={contentType}
      actions={actions}
      extraExtensions={extraExtensions}
    />
  );
}
