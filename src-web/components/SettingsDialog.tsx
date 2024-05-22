import { useActiveWorkspace } from '../hooks/useActiveWorkspace';
import { useAppInfo } from '../hooks/useAppInfo';
import { useCheckForUpdates } from '../hooks/useCheckForUpdates';
import { useSettings } from '../hooks/useSettings';
import { useUpdateSettings } from '../hooks/useUpdateSettings';
import { useUpdateWorkspace } from '../hooks/useUpdateWorkspace';
import { trackEvent } from '../lib/analytics';
import { Checkbox } from './core/Checkbox';
import { Heading } from './core/Heading';
import { IconButton } from './core/IconButton';
import { Input } from './core/Input';
import { KeyValueRow, KeyValueRows } from './core/KeyValueRow';
import { Select } from './core/Select';
import { Separator } from './core/Separator';
import { VStack } from './core/Stacks';

export const SettingsDialog = () => {
  const workspace = useActiveWorkspace();
  const updateWorkspace = useUpdateWorkspace(workspace?.id ?? null);
  const settings = useSettings();
  const updateSettings = useUpdateSettings();
  const appInfo = useAppInfo();
  const checkForUpdates = useCheckForUpdates();

  if (settings == null || workspace == null) {
    return null;
  }

  return (
    <VStack space={2} className="mb-4">
      <Select
        name="appearance"
        label="Appearance"
        labelPosition="left"
        size="sm"
        value={settings.appearance}
        onChange={async (appearance) => {
          await updateSettings.mutateAsync({ ...settings, appearance });
          trackEvent('setting', 'update', { appearance });
        }}
        options={[
          {
            label: 'System',
            value: 'system',
          },
          {
            label: 'Light',
            value: 'light',
          },
          {
            label: 'Dark',
            value: 'dark',
          },
        ]}
      />

      <div className="grid grid-cols-[minmax(0,1fr)_auto] gap-1">
        <Select
          name="updateChannel"
          label="Update Channel"
          labelPosition="left"
          size="sm"
          value={settings.updateChannel}
          onChange={async (updateChannel) => {
            trackEvent('setting', 'update', { update_channel: updateChannel });
            await updateSettings.mutateAsync({ ...settings, updateChannel });
          }}
          options={[
            {
              label: 'Release',
              value: 'stable',
            },
            {
              label: 'Early Bird (Beta)',
              value: 'beta',
            },
          ]}
        />
        <IconButton
          variant="border"
          size="sm"
          title="Check for updates"
          icon="refresh"
          spin={checkForUpdates.isPending}
          onClick={() => checkForUpdates.mutateAsync()}
        />
      </div>

      <Separator className="my-4" />

      <Heading size={2}>
        Workspace{' '}
        <div className="inline-block ml-1 bg-background-highlight px-2 py-0.5 text-sm rounded text-fg">
          {workspace.name}
        </div>
      </Heading>
      <VStack className="mt-1 w-full" space={3}>
        <Input
          size="sm"
          name="requestTimeout"
          label="Request Timeout (ms)"
          placeholder="0"
          labelPosition="left"
          defaultValue={`${workspace.settingRequestTimeout}`}
          validate={(value) => parseInt(value) >= 0}
          onChange={(v) => updateWorkspace.mutateAsync({ settingRequestTimeout: parseInt(v) || 0 })}
        />

        <Checkbox
          checked={workspace.settingValidateCertificates}
          title="Validate TLS Certificates"
          onChange={async (settingValidateCertificates) => {
            trackEvent('workspace', 'update', {
              validate_certificates: JSON.stringify(settingValidateCertificates),
            });
            await updateWorkspace.mutateAsync({ settingValidateCertificates });
          }}
        />

        <Checkbox
          checked={workspace.settingFollowRedirects}
          title="Follow Redirects"
          onChange={async (settingFollowRedirects) => {
            trackEvent('workspace', 'update', {
              follow_redirects: JSON.stringify(settingFollowRedirects),
            });
            await updateWorkspace.mutateAsync({ settingFollowRedirects });
          }}
        />
      </VStack>

      <Separator className="my-4" />

      <Heading size={2}>App Info</Heading>
      <KeyValueRows>
        <KeyValueRow label="Version" value={appInfo.data?.version} />
        <KeyValueRow label="Data Directory" value={appInfo.data?.appDataDir} />
        <KeyValueRow label="Logs Directory" value={appInfo.data?.appLogDir} />
      </KeyValueRows>
    </VStack>
  );
};
