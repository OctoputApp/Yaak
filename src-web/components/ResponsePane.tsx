import type { HttpRequest, HttpResponse } from '@yaakapp-internal/models';
import classNames from 'classnames';
import type { CSSProperties, ReactNode } from 'react';
import React, { memo, useCallback, useMemo } from 'react';
import { useLocalStorage } from 'react-use';
import { useContentTypeFromHeaders } from '../hooks/useContentTypeFromHeaders';
import { usePinnedHttpResponse } from '../hooks/usePinnedHttpResponse';
import { useResponseViewMode } from '../hooks/useResponseViewMode';
import { isResponseLoading } from '../lib/model_util';
import { Banner } from './core/Banner';
import { CountBadge } from './core/CountBadge';
import { DurationTag } from './core/DurationTag';
import { HotKeyList } from './core/HotKeyList';
import { Icon } from './core/Icon';
import { SizeTag } from './core/SizeTag';
import { HStack } from './core/Stacks';
import { StatusTag } from './core/StatusTag';
import type { TabItem } from './core/Tabs/Tabs';
import { TabContent, Tabs } from './core/Tabs/Tabs';
import { EmptyStateText } from './EmptyStateText';
import { RecentResponsesDropdown } from './RecentResponsesDropdown';
import { ResponseHeaders } from './ResponseHeaders';
import { ResponseInfo } from './ResponseInfo';
import { AudioViewer } from './responseViewers/AudioViewer';
import { CsvViewer } from './responseViewers/CsvViewer';
import { EventStreamViewer } from './responseViewers/EventStreamViewer';
import { HTMLOrTextViewer } from './responseViewers/HTMLOrTextViewer';
import { ImageViewer } from './responseViewers/ImageViewer';
import { PdfViewer } from './responseViewers/PdfViewer';
import { VideoViewer } from './responseViewers/VideoViewer';

interface Props {
  style?: CSSProperties;
  className?: string;
  activeRequest: HttpRequest;
}

const TAB_BODY = 'body';
const TAB_HEADERS = 'headers';
const TAB_INFO = 'info';
const DEFAULT_TAB = TAB_BODY;

export const ResponsePane = memo(function ResponsePane({ style, className, activeRequest }: Props) {
  const { activeResponse, setPinnedResponseId, responses } = usePinnedHttpResponse(activeRequest);
  const [viewMode, setViewMode] = useResponseViewMode(activeResponse?.requestId);
  const [activeTabs, setActiveTabs] = useLocalStorage<Record<string, string>>(
    'responsePaneActiveTabs',
    {},
  );
  const contentType = useContentTypeFromHeaders(activeResponse?.headers ?? null);
  const activeTab = activeTabs?.[activeRequest.id] ?? DEFAULT_TAB;
  const setActiveTab = useCallback(
    (tab: string) => {
      setActiveTabs((r) => ({ ...r, [activeRequest.id]: tab }));
    },
    [activeRequest.id, setActiveTabs],
  );

  const tabs = useMemo<TabItem[]>(
    () => [
      {
        value: TAB_BODY,
        label: 'Preview Mode',
        options: {
          value: viewMode,
          onChange: setViewMode,
          items: [
            { label: 'Pretty', value: 'pretty' },
            ...(contentType?.startsWith('image') ? [] : [{ label: 'Raw', value: 'raw' }]),
          ],
        },
      },
      {
        value: TAB_HEADERS,
        label: (
          <div className="flex items-center">
            Headers
            <CountBadge
              count={activeResponse?.headers.filter((h) => h.name && h.value).length ?? 0}
            />
          </div>
        ),
      },
      {
        value: TAB_INFO,
        label: 'Info',
      },
    ],
    [activeResponse?.headers, contentType, setViewMode, viewMode],
  );

  const isLoading = isResponseLoading(activeResponse);

  return (
    <div
      style={style}
      className={classNames(
        className,
        'x-theme-responsePane',
        'max-h-full h-full',
        'bg-surface rounded-md border border-border-subtle',
        'relative',
      )}
    >
      {activeResponse == null ? (
        <HotKeyList
          hotkeys={['http_request.send', 'http_request.create', 'sidebar.focus', 'urlBar.focus']}
        />
      ) : (
        <div className="h-full w-full grid grid-rows-[auto_minmax(0,1fr)] grid-cols-1">
          <HStack
            className={classNames(
              'text-text-subtle w-full flex-shrink-0',
              // Remove a bit of space because the tabs have lots too
              '-mb-1.5',
            )}
          >
            {activeResponse && (
              <HStack
                space={2}
                alignItems="center"
                className={classNames(
                  'select-none cursor-default',
                  'whitespace-nowrap w-full pl-3 overflow-x-auto font-mono text-sm',
                )}
              >
                {isLoading && <Icon size="sm" icon="refresh" spin />}
                <StatusTag showReason response={activeResponse} />
                <span>&bull;</span>
                <DurationTag
                  headers={activeResponse.elapsedHeaders}
                  total={activeResponse.elapsed}
                />
                <span>&bull;</span>
                <SizeTag contentLength={activeResponse.contentLength ?? 0} />

                <div className="ml-auto">
                  <RecentResponsesDropdown
                    responses={responses}
                    activeResponse={activeResponse}
                    onPinnedResponseId={setPinnedResponseId}
                  />
                </div>
              </HStack>
            )}
          </HStack>

          {activeResponse?.error ? (
            <Banner color="danger" className="m-2">
              {activeResponse.error}
            </Banner>
          ) : (
            <Tabs
              key={activeRequest.id} // Freshen tabs on request change
              value={activeTab}
              onChangeValue={setActiveTab}
              tabs={tabs}
              label="Response"
              className="ml-3 mr-3 mb-3"
              tabListClassName="mt-1.5"
            >
              <TabContent value={TAB_BODY}>
                {!activeResponse.contentLength ? (
                  <div className="pb-2 h-full">
                    <EmptyStateText>Empty Body</EmptyStateText>
                  </div>
                ) : contentType?.match(/^text\/event-stream$/i) && viewMode === 'pretty' ? (
                  <EventStreamViewer response={activeResponse} />
                ) : contentType?.match(/^image/i) ? (
                  <EnsureCompleteResponse response={activeResponse} render={ImageViewer} />
                ) : contentType?.match(/^audio/i) ? (
                  <EnsureCompleteResponse response={activeResponse} render={AudioViewer} />
                ) : contentType?.match(/^video/i) ? (
                  <EnsureCompleteResponse response={activeResponse} render={VideoViewer} />
                ) : contentType?.match(/pdf/i) ? (
                  <EnsureCompleteResponse response={activeResponse} render={PdfViewer} />
                ) : contentType?.match(/csv|tab-separated/i) ? (
                  <CsvViewer className="pb-2" response={activeResponse} />
                ) : (
                  // ) : viewMode === 'pretty' && contentType?.includes('json') ? (
                  //   <JsonAttributeTree attrValue={activeResponse} />
                  <HTMLOrTextViewer
                    textViewerClassName="-mr-2 bg-surface" // Pull to the right
                    response={activeResponse}
                    pretty={viewMode === 'pretty'}
                  />
                )}
              </TabContent>
              <TabContent value={TAB_HEADERS}>
                <ResponseHeaders response={activeResponse} />
              </TabContent>
              <TabContent value={TAB_INFO}>
                <ResponseInfo response={activeResponse} />
              </TabContent>
            </Tabs>
          )}
        </div>
      )}
    </div>
  );
});

function EnsureCompleteResponse({
  response,
  render,
}: {
  response: HttpResponse;
  render: (v: { bodyPath: string }) => ReactNode;
}) {
  if (response.bodyPath === null) {
    return <div>Empty response body</div>;
  }

  // Wait until the response has been fully-downloaded
  if (response.state !== 'closed') {
    return (
      <EmptyStateText>
        <Icon icon="refresh" spin />
      </EmptyStateText>
    );
  }

  return render({ bodyPath: response.bodyPath });
}
