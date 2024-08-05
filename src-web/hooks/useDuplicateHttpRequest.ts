import { useMutation } from '@tanstack/react-query';
import { trackEvent } from '../lib/analytics';
import type { HttpRequest } from '@yaakapp/api';
import { invokeCmd } from '../lib/tauri';
import { useActiveEnvironmentId } from './useActiveEnvironmentId';
import { useActiveWorkspaceId } from './useActiveWorkspaceId';
import { useAppRoutes } from './useAppRoutes';

export function useDuplicateHttpRequest({
  id,
  navigateAfter,
}: {
  id: string | null;
  navigateAfter: boolean;
}) {
  const activeWorkspaceId = useActiveWorkspaceId();
  const activeEnvironmentId = useActiveEnvironmentId();
  const routes = useAppRoutes();
  return useMutation<HttpRequest, string>({
    mutationKey: ['duplicate_http_request', id],
    mutationFn: async () => {
      if (id === null) throw new Error("Can't duplicate a null request");
      return invokeCmd('cmd_duplicate_http_request', { id });
    },
    onSettled: () => trackEvent('http_request', 'duplicate'),
    onSuccess: async (request) => {
      if (navigateAfter && activeWorkspaceId !== null) {
        routes.navigate('request', {
          workspaceId: activeWorkspaceId,
          requestId: request.id,
          environmentId: activeEnvironmentId ?? undefined,
        });
      }
    },
  });
}
