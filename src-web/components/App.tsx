import { QueryCache, QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { ReactQueryDevtools } from '@tanstack/react-query-devtools';
import { MotionConfig } from 'framer-motion';
import React, { Suspense } from 'react';
import { DndProvider } from 'react-dnd';
import { HTML5Backend } from 'react-dnd-html5-backend';
import { HelmetProvider } from 'react-helmet-async';
import { AppRouter } from './AppRouter';

const queryClient = new QueryClient({
  queryCache: new QueryCache({
    onError: (err, query) => {
      console.log('Query client error', { err, query });
    },
  }),
  defaultOptions: {
    queries: {
      retry: false,
      networkMode: 'always',
      refetchOnWindowFocus: true,
      refetchOnReconnect: false,
      refetchOnMount: false, // Don't refetch when a hook mounts
    },
  },
});

const ENABLE_REACT_QUERY_DEVTOOLS = true;

export function App() {
  return (
    <QueryClientProvider client={queryClient}>
      {ENABLE_REACT_QUERY_DEVTOOLS && <ReactQueryDevtools buttonPosition="bottom-right" />}
      <MotionConfig transition={{ duration: 0.1 }}>
        <HelmetProvider>
          <DndProvider backend={HTML5Backend}>
            <Suspense>
              <AppRouter />
            </Suspense>
          </DndProvider>
        </HelmetProvider>
      </MotionConfig>
    </QueryClientProvider>
  );
}
