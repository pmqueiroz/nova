export const OSES = [
  { id: 'mac', label: 'macOS' },
  { id: 'win', label: 'Windows' },
  { id: 'linux', label: 'Linux' },
] as const;

export type OsId = (typeof OSES)[number]['id'];

export const OS_ALIASES: Record<string, OsId> = {
  mac: 'mac',
  macos: 'mac',
  win: 'win',
  windows: 'win',
  linux: 'linux',
};
