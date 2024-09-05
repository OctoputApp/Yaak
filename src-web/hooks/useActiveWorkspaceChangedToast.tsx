import { useEffect, useState } from 'react';
import { InlineCode } from '../components/core/InlineCode';
import { useToast } from '../components/ToastContext';
import { useActiveWorkspace } from './useActiveWorkspace';

export function useActiveWorkspaceChangedToast() {
  const toast = useToast();
  const activeWorkspace = useActiveWorkspace();
  const [id, setId] = useState<string | null>(activeWorkspace?.id ?? null);

  useEffect(() => {
    // Early return if same or invalid active workspace
    if (id === activeWorkspace?.id || activeWorkspace == null) return;

    setId(activeWorkspace?.id ?? null);

    // Don't notify on the first load
    if (id === null) return;

    console.log('HELLO?', activeWorkspace?.id, id, window.location);

    toast.show({
      id: 'workspace-changed',
      timeout: 3000,
      message: (
        <>
          Switched workspace to <InlineCode>{activeWorkspace.name}</InlineCode>
        </>
      ),
    });
  }, [activeWorkspace, id, toast]);
}
