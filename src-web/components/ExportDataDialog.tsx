import { save } from '@tauri-apps/plugin-dialog';
import { useCallback, useMemo, useState } from 'react';
import slugify from 'slugify';
import type { Workspace } from '@yaakapp-internal/models';
import { count } from '../lib/pluralize';
import { invokeCmd } from '../lib/tauri';
import { Button } from './core/Button';
import { Checkbox } from './core/Checkbox';
import { HStack, VStack } from './core/Stacks';

interface Props {
  onHide: () => void;
  onSuccess: (path: string) => void;
  activeWorkspace: Workspace;
  workspaces: Workspace[];
}

export function ExportDataDialog({
  onHide,
  onSuccess,
  activeWorkspace,
  workspaces: allWorkspaces,
}: Props) {
  const [selectedWorkspaces, setSelectedWorkspaces] = useState<Record<string, boolean>>({
    [activeWorkspace.id]: true,
  });

  // Put active workspace first
  const workspaces = useMemo(
    () => [activeWorkspace, ...allWorkspaces.filter((w) => w.id !== activeWorkspace.id)],
    [activeWorkspace, allWorkspaces],
  );

  const handleToggleAll = () => {
    setSelectedWorkspaces(
      allSelected ? {} : workspaces.reduce((acc, w) => ({ ...acc, [w.id]: true }), {}),
    );
  };

  const handleExport = useCallback(async () => {
    const ids = Object.keys(selectedWorkspaces).filter((k) => selectedWorkspaces[k]);
    const workspace = ids.length === 1 ? workspaces.find((w) => w.id === ids[0]) : undefined;
    const slug = workspace ? slugify(workspace.name, { lower: true }) : 'workspaces';
    const exportPath = await save({
      title: 'Export Data',
      defaultPath: `yaak.${slug}.json`,
    });
    if (exportPath == null) {
      return;
    }

    await invokeCmd('cmd_export_data', { workspaceIds: ids, exportPath });
    onHide();
    onSuccess(exportPath);
  }, [onHide, onSuccess, selectedWorkspaces, workspaces]);

  const allSelected = workspaces.every((w) => selectedWorkspaces[w.id]);
  const numSelected = Object.values(selectedWorkspaces).filter(Boolean).length;
  const noneSelected = numSelected === 0;
  return (
    <VStack space={3} className="w-full mb-3 px-4">
      <table className="w-full mb-auto min-w-full max-w-full divide-y divide-surface-highlight">
        <thead>
          <tr>
            <th className="w-6 min-w-0 py-2 text-left pl-1">
              <Checkbox
                checked={!allSelected && !noneSelected ? 'indeterminate' : allSelected}
                hideLabel
                title="All workspaces"
                onChange={handleToggleAll}
              />
            </th>
            <th className="py-2 text-left pl-4" onClick={handleToggleAll}>
              Workspace
            </th>
          </tr>
        </thead>
        <tbody className="divide-y divide-surface-highlight">
          {workspaces.map((w) => (
            <tr key={w.id}>
              <td className="min-w-0 py-1 pl-1">
                <Checkbox
                  checked={selectedWorkspaces[w.id] ?? false}
                  title={w.name}
                  hideLabel
                  onChange={() =>
                    setSelectedWorkspaces((prev) => ({ ...prev, [w.id]: !prev[w.id] }))
                  }
                />
              </td>
              {/* eslint-disable-next-line jsx-a11y/no-noninteractive-element-interactions,jsx-a11y/click-events-have-key-events */}
              <td
                className="py-1 pl-4 text whitespace-nowrap overflow-x-auto hide-scrollbars"
                onClick={() => setSelectedWorkspaces((prev) => ({ ...prev, [w.id]: !prev[w.id] }))}
              >
                {w.name} {w.id === activeWorkspace.id ? '(current workspace)' : ''}
              </td>
            </tr>
          ))}
        </tbody>
      </table>
      <HStack space={2} justifyContent="end">
        <Button className="focus" variant="border" onClick={onHide}>
          Cancel
        </Button>
        <Button
          type="submit"
          className="focus"
          color="primary"
          disabled={noneSelected}
          onClick={() => handleExport()}
        >
          Export {count('Workspace', numSelected, { omitSingle: true, noneWord: 'Nothing' })}
        </Button>
      </HStack>
    </VStack>
  );
}
