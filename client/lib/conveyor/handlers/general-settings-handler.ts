import { app } from 'electron'
import { join } from 'path'
import { handle } from '@/lib/main/shared'
import { createBetterSqliteAdapter } from '@/lib/storage/sqlite-adapter'
import { createGeneralSettingsStore } from '@/lib/storage/general-settings-db-store'

let store: ReturnType<typeof createGeneralSettingsStore> | null = null

const getStore = () => {
  if (store) {
    return store
  }

  const dbPath = join(app.getPath('userData'), 'general-settings.db')
  const adapter = createBetterSqliteAdapter(dbPath)
  store = createGeneralSettingsStore(adapter)
  store.init()

  return store
}

export const registerGeneralSettingsHandlers = () => {
  app.on('before-quit', () => {
    store?.close?.()
  })

  handle('general-settings/get', async () => {
    return getStore().getGeneralSettings()
  })

  handle('general-settings/set', async (settings) => {
    return getStore().setGeneralSettings(settings)
  })

  handle('general-settings/update', async (key, value) => {
    return getStore().updateGeneralSetting(key, value)
  })

  handle('general-settings/reset', async () => {
    return getStore().resetGeneralSettings()
  })
}
