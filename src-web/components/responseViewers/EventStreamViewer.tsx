import { useVirtualizer } from '@tanstack/react-virtual';
import type { HttpResponse } from '@yaakapp-internal/models';
import type { ServerSentEvent } from '@yaakapp-internal/sse';
import classNames from 'classnames';
import type { ReactNode } from 'react';
import React, { useEffect, useMemo, useRef, useState } from 'react';
import { useResponseBodyEventSource } from '../../hooks/useResponseBodyEventSource';
import { isJSON } from '../../lib/contentType';
import { tryFormatJson } from '../../lib/formatters';
import { Button } from '../core/Button';
import { Editor } from '../core/Editor';
import { Icon } from '../core/Icon';
import { InlineCode } from '../core/InlineCode';
import { Separator } from '../core/Separator';
import { SplitLayout } from '../core/SplitLayout';
import { HStack, VStack } from '../core/Stacks';
import { motion } from 'framer-motion';

interface Props {
  response: HttpResponse;
}

export function EventStreamViewer({ response }: Props) {
  return (
    <Lazy // The initial parsing can be heavy, so lazily load the component
      key={response.id}
      render={() => <_EventStreamViewer response={response} />}
    />
  );
}

function _EventStreamViewer({ response }: Props) {
  const [showLarge, setShowLarge] = useState<boolean>(false);
  const [showingLarge, setShowingLarge] = useState<boolean>(false);
  const [activeEventIndex, setActiveEventIndex] = useState<number | null>(null);
  const events = useResponseBodyEventSource(response);
  const activeEvent = useMemo(
    () => (activeEventIndex == null ? null : events.data?.[activeEventIndex]),
    [activeEventIndex, events],
  );

  const language = useMemo<'text' | 'json'>(() => {
    if (!activeEvent?.data) return 'text';
    return isJSON(activeEvent?.data) ? 'json' : 'text';
  }, [activeEvent?.data]);

  return (
    <SplitLayout
      layout="vertical"
      name="grpc_events"
      defaultRatio={0.4}
      minHeightPx={20}
      firstSlot={() => (
        <EventStreamEventsVirtual
          events={events.data ?? []}
          activeEventIndex={activeEventIndex}
          setActiveEventIndex={setActiveEventIndex}
        />
      )}
      secondSlot={
        activeEvent
          ? () => (
              <div className="grid grid-rows-[auto_minmax(0,1fr)]">
                <div className="pb-3 px-2">
                  <Separator />
                </div>
                <div className="pl-2 overflow-y-auto">
                  <div className="mb-2 select-text cursor-text font-semibold">Message Received</div>
                  {!showLarge && activeEvent.data.length > 1000 * 1000 ? (
                    <VStack space={2} className="italic text-text-subtlest">
                      Message previews larger than 1MB are hidden
                      <div>
                        <Button
                          onClick={() => {
                            setShowingLarge(true);
                            setTimeout(() => {
                              setShowLarge(true);
                              setShowingLarge(false);
                            }, 500);
                          }}
                          isLoading={showingLarge}
                          color="secondary"
                          variant="border"
                          size="xs"
                        >
                          Try Showing
                        </Button>
                      </div>
                    </VStack>
                  ) : (
                    <Editor
                      readOnly
                      forceUpdateKey={activeEvent.id ?? activeEvent.data}
                      defaultValue={tryFormatJson(activeEvent.data)}
                      language={language}
                    />
                  )}
                </div>
              </div>
            )
          : null
      }
    />
  );
}

function EventStreamEventsVirtual({
  events,
  activeEventIndex,
  setActiveEventIndex,
}: {
  events: ServerSentEvent[];
  activeEventIndex: number | null;
  setActiveEventIndex: (eventId: number | null) => void;
}) {
  // The scrollable element for your list
  const parentRef = useRef<HTMLDivElement>(null);

  // The virtualizer
  const rowVirtualizer = useVirtualizer({
    count: events.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => 28, // react-virtual requires a height, so we'll give it one
  });

  return (
    <div ref={parentRef} className="pb-3 overflow-y-auto">
      <div
        style={{
          height: `${rowVirtualizer.getTotalSize()}px`,
          width: '100%',
          position: 'relative',
        }}
      >
        {rowVirtualizer.getVirtualItems().map((virtualItem) => {
          const event = events[virtualItem.index]!;
          return (
            <div
              key={virtualItem.key}
              style={{
                position: 'absolute',
                top: 0,
                left: 0,
                width: '100%',
                height: `${virtualItem.size}px`,
                transform: `translateY(${virtualItem.start}px)`,
              }}
            >
              <EventStreamEvent
                event={event}
                isActive={virtualItem.index === activeEventIndex}
                onClick={() => {
                  if (virtualItem.index === activeEventIndex) setActiveEventIndex(null);
                  else setActiveEventIndex(virtualItem.index);
                }}
              />
            </div>
          );
        })}
      </div>
    </div>
  );
}

function EventStreamEvent({
  onClick,
  isActive,
  event,
  className,
}: {
  onClick: () => void;
  isActive: boolean;
  event: ServerSentEvent;
  className?: string;
}) {
  return (
    <motion.button
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
      onClick={onClick}
      className={classNames(
        className,
        'w-full grid grid-cols-[auto_auto_minmax(0,3fr)] gap-2 items-center text-left',
        'px-1.5 py-1 font-mono cursor-default group focus:outline-none rounded',
        isActive && '!bg-surface-highlight !text-text',
        'text-text-subtle hover:text',
      )}
    >
      <Icon className={classNames('text-info')} title="Server Message" icon="arrow_big_down_dash" />
      <HStack space={1.5} className="text-sm">
        {event.eventType && (
          <InlineCode className={classNames('py-0', isActive && 'bg-text-subtlest text-text')}>
            {event.eventType}
          </InlineCode>
        )}
        {event.id && (
          <InlineCode className={classNames('py-0', isActive && 'bg-text-subtlest text-text')}>
            {event.id}
          </InlineCode>
        )}
      </HStack>
      <div className={classNames('w-full truncate text-xs')}>{event.data.slice(0, 1000)}</div>
    </motion.button>
  );
}

function Lazy({ render }: { render: () => ReactNode }) {
  const [visible, setVisible] = useState(false);
  useEffect(() => {
    setVisible(true);
  }, []);

  if (!visible) return false;

  return <>{render()}</>;
}
