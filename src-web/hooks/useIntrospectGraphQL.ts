import type { IntrospectionQuery } from 'graphql';
import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { useLocalStorage } from 'react-use';
import { buildClientSchema, getIntrospectionQuery } from '../components/core/Editor';
import { minPromiseMillis } from '../lib/minPromiseMillis';
import type { HttpRequest } from '../lib/models';
import { getResponseBodyText } from '../lib/responseBody';
import { sendEphemeralRequest } from '../lib/sendEphemeralRequest';
import { useActiveEnvironmentId } from './useActiveEnvironmentId';
import { useDebouncedValue } from './useDebouncedValue';

const introspectionRequestBody = JSON.stringify({
  query: getIntrospectionQuery(),
  operationName: 'IntrospectionQuery',
});

export function useIntrospectGraphQL(baseRequest: HttpRequest) {
  // Debounce the request because it can change rapidly and we don't
  // want to send so too many requests.
  const request = useDebouncedValue(baseRequest);
  const activeEnvironmentId = useActiveEnvironmentId();
  const [refetchKey, setRefetchKey] = useState<number>(0);
  const [isLoading, setIsLoading] = useState<boolean>(false);
  const [error, setError] = useState<string>();
  const [introspection, setIntrospection] = useLocalStorage<IntrospectionQuery | null>(
    `introspection:${baseRequest.id}`,
  );

  const introspectionInterval = useRef<NodeJS.Timeout>();

  useEffect(() => {
    const fetchIntrospection = async () => {
      setIsLoading(true);
      setError(undefined);
      const args = {
        ...baseRequest,
        bodyType: 'application/json',
        body: { text: introspectionRequestBody },
      };
      const response = await minPromiseMillis(sendEphemeralRequest(args, activeEnvironmentId), 700);

      if (response.error) {
        return Promise.reject(new Error(response.error));
      }

      const bodyText = await getResponseBodyText(response);
      if (response.status < 200 || response.status >= 300) {
        return Promise.reject(
          new Error(`Request failed with status ${response.status}.\n\n${bodyText}`),
        );
      }

      if (bodyText === null) {
        return Promise.reject(new Error('Empty body returned in response'));
      }

      const { data } = JSON.parse(bodyText);
      console.log('Introspection response', data);
      setIntrospection(data);
    };

    const runIntrospection = () => {
      fetchIntrospection()
        .catch((e) => {
          setIntrospection(null);
          setError(e.message);
        })
        .finally(() => setIsLoading(false));
    };

    // Do it again on an interval
    clearInterval(introspectionInterval.current);
    introspectionInterval.current = setInterval(runIntrospection, 1000 * 60);
    runIntrospection(); // Run immediately

    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [request.id, request.url, request.method, refetchKey, activeEnvironmentId]);

  const refetch = useCallback(() => {
    setRefetchKey((k) => k + 1);
  }, []);

  const schema = useMemo(() => {
    try {
      return introspection ? buildClientSchema(introspection) : undefined;
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
    } catch (e: any) {
      setError('message' in e ? e.message : String(e));
    }
  }, [introspection]);

  return { schema, isLoading, error, refetch };
}
