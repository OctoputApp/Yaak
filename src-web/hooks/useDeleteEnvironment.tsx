import { useMutation } from '@tanstack/react-query';
import type { Environment } from '@yaakapp-internal/models';
import { InlineCode } from '../components/core/InlineCode';
import { trackEvent } from '../lib/analytics';
import { invokeCmd } from '../lib/tauri';
import { useConfirm } from './useConfirm';

export function useDeleteEnvironment(environment: Environment | null) {
  const confirm = useConfirm();

  return useMutation<Environment | null, string>({
    mutationKey: ['delete_environment', environment?.id],
    mutationFn: async () => {
      const confirmed = await confirm({
        id: 'delete-environment',
        title: 'Delete Environment',
        variant: 'delete',
        description: (
          <>
            Permanently delete <InlineCode>{environment?.name}</InlineCode>?
          </>
        ),
      });
      if (!confirmed) return null;
      return invokeCmd('cmd_delete_environment', { environmentId: environment?.id });
    },
    onSettled: () => trackEvent('environment', 'delete'),
  });
}
