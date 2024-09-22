import type { HttpResponse } from '@yaakapp-internal/models';
import classNames from 'classnames';
import type { ReactNode } from 'react';
import { useCallback, useMemo } from 'react';
import { createGlobalState } from 'react-use';
import { useContentTypeFromHeaders } from '../../hooks/useContentTypeFromHeaders';
import { useDebouncedValue } from '../../hooks/useDebouncedValue';
import { useFilterResponse } from '../../hooks/useFilterResponse';
import { useResponseBodyText } from '../../hooks/useResponseBodyText';
import { useSaveResponse } from '../../hooks/useSaveResponse';
import { useToggle } from '../../hooks/useToggle';
import { isJSON, languageFromContentType } from '../../lib/contentType';
import { tryFormatJson, tryFormatXml } from '../../lib/formatters';
import { CopyButton } from '../CopyButton';
import { Banner } from '../core/Banner';
import { Button } from '../core/Button';
import { Editor } from '../core/Editor';
import { hyperlink } from '../core/Editor/hyperlink/extension';
import { IconButton } from '../core/IconButton';
import { InlineCode } from '../core/InlineCode';
import { Input } from '../core/Input';
import { SizeTag } from '../core/SizeTag';
import { HStack } from '../core/Stacks';
import { BinaryViewer } from './BinaryViewer';

const extraExtensions = [hyperlink];
const LARGE_RESPONSE_BYTES = 2 * 1000 * 1000;

interface Props {
  response: HttpResponse;
  pretty: boolean;
  className?: string;
}

const useFilterText = createGlobalState<Record<string, string | null>>({});

export function TextViewer({ response, pretty, className }: Props) {
  const [filterTextMap, setFilterTextMap] = useFilterText();
  const [showLargeResponse, toggleShowLargeResponse] = useToggle();
  const filterText = filterTextMap[response.id] ?? null;
  const debouncedFilterText = useDebouncedValue(filterText, 200);
  const setFilterText = useCallback(
    (v: string | null) => {
      setFilterTextMap((m) => ({ ...m, [response.id]: v }));
    },
    [setFilterTextMap, response],
  );

  const rawBody = useResponseBodyText(response);
  const saveResponse = useSaveResponse(response);
  let language = languageFromContentType(useContentTypeFromHeaders(response.headers));

  // A lot of APIs return JSON with `text/html` content type, so interpret as JSON if so
  if (language === 'html' && isJSON(rawBody.data ?? '')) {
    language = 'json';
  }

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

  const canFilter = language === 'json' || language === 'xml' || language === 'html';

  const actions = useMemo<ReactNode[]>(() => {
    const nodes: ReactNode[] = [];

    if (!canFilter) return nodes;

    if (isSearching) {
      nodes.push(
        <div key="input" className="w-full !opacity-100">
          <Input
            key={response.id}
            validate={!filteredResponse.error}
            hideLabel
            autoFocus
            containerClassName="bg-surface"
            size="sm"
            placeholder={language === 'json' ? 'JSONPath expression' : 'XPath expression'}
            label="Filter expression"
            name="filter"
            defaultValue={filterText}
            onKeyDown={(e) => e.key === 'Escape' && toggleSearch()}
            onChange={setFilterText}
          />
        </div>,
      );
    }

    nodes.push(
      <IconButton
        key="icon"
        size="sm"
        icon={isSearching ? 'x' : 'filter'}
        title={isSearching ? 'Close filter' : 'Filter response'}
        onClick={toggleSearch}
        className={classNames('border !border-border-subtle', isSearching && '!opacity-100')}
      />,
    );

    return nodes;
  }, [
    canFilter,
    filterText,
    filteredResponse.error,
    isSearching,
    language,
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

  if (!showLargeResponse && (response.contentLength ?? 0) > LARGE_RESPONSE_BYTES) {
    return (
      <Banner color="primary" className="h-full flex flex-col gap-3">
        <p>
          Showing responses over{' '}
          <InlineCode>
            <SizeTag contentLength={LARGE_RESPONSE_BYTES} />
          </InlineCode>{' '}
          may impact performance
        </p>
        <HStack wrap space={2}>
          <Button color="primary" size="xs" onClick={toggleShowLargeResponse}>
            Reveal Response
          </Button>
          <Button variant="border" size="xs" onClick={() => saveResponse.mutate()}>
            Save to File
          </Button>
          <CopyButton
            variant="border"
            size="xs"
            onClick={() => saveResponse.mutate()}
            text={rawBody.data}
          />
        </HStack>
      </Banner>
    );
  }

  const formattedBody =
    pretty && language === 'json'
      ? tryFormatJson(rawBody.data)
      : pretty && (language === 'xml' || language === 'html')
        ? tryFormatXml(rawBody.data)
        : rawBody.data;

  let body;
  if (isSearching && filterText?.length > 0) {
    if (filteredResponse.error) {
      body = '';
    } else {
      body = filteredResponse.data != null ? filteredResponse.data : '';
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
      language={language}
      actions={actions}
      extraExtensions={extraExtensions}
    />
  );
}
