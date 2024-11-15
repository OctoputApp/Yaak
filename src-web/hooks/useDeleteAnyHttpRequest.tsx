import { useMutation } from '@tanstack/react-query';
import type { HttpRequest } from '@yaakapp-internal/models';
import { useSetAtom } from 'jotai';
import { InlineCode } from '../components/core/InlineCode';
import { trackEvent } from '../lib/analytics';
import { resolvedModelName } from '../lib/resolvedModelName';
import { getHttpRequest } from '../lib/store';
import { invokeCmd } from '../lib/tauri';
import { useConfirm } from './useConfirm';
import { httpRequestsAtom } from './useHttpRequests';
import { removeModelById } from './useSyncModelStores';

export function useDeleteAnyHttpRequest() {
  const confirm = useConfirm();
  const setHttpRequests = useSetAtom(httpRequestsAtom);

  return useMutation<HttpRequest | null, string, string>({
    mutationKey: ['delete_any_http_request'],
    mutationFn: async (id) => {
      const request = await getHttpRequest(id);
      if (request == null) return null;

      const confirmed = await confirm({
        id: 'delete-request',
        title: 'Delete Request',
        variant: 'delete',
        description: (
          <>
            Permanently delete <InlineCode>{resolvedModelName(request)}</InlineCode>?
          </>
        ),
      });
      if (!confirmed) return null;
      return invokeCmd<HttpRequest>('cmd_delete_http_request', { requestId: id });
    },
    onSuccess: (request) => {
      if (request == null) return;

      // Optimistic update
      setHttpRequests(removeModelById(request));
    },
    onSettled: () => trackEvent('http_request', 'delete'),
  });
}
