import { invoke } from '@tauri-apps/api/core'

export const listAliases = () => invoke('list_aliases')

export const addAlias = (name, command, description) =>
  invoke('add_alias', { name, command, description })

export const updateAlias = (id, name, command, description) =>
  invoke('update_alias', { id, name, command, description })

export const deleteAlias = (id) => invoke('delete_alias', { id })

export const toggleAlias = (id, enabled) =>
  invoke('toggle_alias', { id, enabled })

export const getStatus = () => invoke('get_status')

export const ensurePath = () => invoke('ensure_path')

export const openBinDir = () => invoke('open_bin_dir')

export const testAlias = (name) => invoke('test_alias', { name })

export const exportAliases = (ids) => invoke('export_aliases', { ids })

export const importAliases = (data) => invoke('import_aliases', { data })

export const setEnabled = (ids, enabled) => invoke('set_enabled', { ids, enabled })

export const deleteAliases = (ids) => invoke('delete_aliases', { ids })
