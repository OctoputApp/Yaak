import type { Plugin } from '@yaakapp/api';
import React from 'react';
import { useCreatePlugin } from '../../hooks/useCreatePlugin';
import { usePluginInfo } from '../../hooks/usePluginInfo';
import { usePlugins, useRefreshPlugins } from '../../hooks/usePlugins';
import { Button } from '../core/Button';
import { Checkbox } from '../core/Checkbox';
import { IconButton } from '../core/IconButton';
import { InlineCode } from '../core/InlineCode';
import { HStack } from '../core/Stacks';
import { SelectFile } from '../SelectFile';

export function SettingsPlugins() {
  const [directory, setDirectory] = React.useState<string | null>(null);
  const plugins = usePlugins();
  const createPlugin = useCreatePlugin();
  const refreshPlugins = useRefreshPlugins();
  return (
    <div className="grid grid-rows-[minmax(0,1fr)_auto] h-full">
      <table className="w-full text-sm mb-auto min-w-full max-w-full divide-y divide-surface-highlight">
        <thead>
          <tr>
            <th></th>
            <th className="py-2 text-left">Plugin</th>
            <th className="py-2 text-right">Version</th>
            <th></th>
          </tr>
        </thead>
        <tbody className="divide-y divide-surface-highlight">
          {plugins.map((p) => (
            <PluginInfo key={p.id} plugin={p} />
          ))}
        </tbody>
      </table>
      <form
        onSubmit={(e) => {
          e.preventDefault();
          if (directory == null) return;
          createPlugin.mutate(directory);
          setDirectory(null);
        }}
      >
        <footer className="grid grid-cols-[minmax(0,1fr)_auto] -mx-4 py-2 px-4 border-t bg-surface-highlight border-border-subtle min-w-0">
          <SelectFile
            size="xs"
            noun="Plugin"
            directory
            onChange={({ filePath }) => setDirectory(filePath)}
            filePath={directory}
          />
          <HStack>
            {directory && (
              <Button size="xs" type="submit" color="primary" className="ml-auto">
                Add Plugin
              </Button>
            )}
            <IconButton
              size="sm"
              icon="refresh"
              title="Reload plugins"
              spin={refreshPlugins.isPending}
              onClick={() => refreshPlugins.mutate()}
            />
          </HStack>
        </footer>
      </form>
    </div>
  );
}

function PluginInfo({ plugin }: { plugin: Plugin }) {
  const pluginInfo = usePluginInfo(plugin.id);
  if (pluginInfo.data == null) return null;
  return (
    <tr className="group">
      <td className="pr-2">
        <Checkbox hideLabel checked={true} title="foo" onChange={() => null} />
      </td>
      <td className="py-2 select-text cursor-text w-full">
        <InlineCode>{pluginInfo.data?.name}</InlineCode>
      </td>
      <td className="py-2 select-text cursor-text text-right">
        <InlineCode>{pluginInfo.data?.version}</InlineCode>
      </td>
      <td className="py-2 select-text cursor-text pl-2">
        <IconButton
          size="sm"
          icon="trash"
          title="Uninstall plugin"
          className="text-text-subtlest"
        />
      </td>
    </tr>
  );
}
