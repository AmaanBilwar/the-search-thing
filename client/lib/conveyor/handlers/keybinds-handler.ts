import { app } from 'electron'
import { join } from 'path'
import { handle } from '@/lib/main/shared'
import { createBetterSqliteAdapter } from '@/lib/storage/sqlite-adapter'
import { createKeybindsStore } from '@/lib/storage/keybinds-db-store'

let store: ReturnType<typeof createKeybindsStore> | null = null

const getStore = () => {
  if (store) {
    return store
  }

  const dbPath = join(app.getPath('userData'), 'keybinds.db')
  const adapter = createBetterSqliteAdapter(dbPath)
  store = createKeybindsStore(adapter)
  store.init()

  return store
}

export const registerKeybindsHandlers = () => {
  app.on('before-quit', () => {
    store?.close?.()
  })

  handle('keybinds/get', async () => {
    return getStore().getKeybinds()
  })

  handle('keybinds/set', async (map) => {
    return getStore().setKeybinds(map)
  })

  handle('keybinds/update', async (action, combo) => {
    return getStore().updateKeybind(action, combo)
  })

  handle('keybinds/reset', async () => {
    return getStore().resetKeybinds()
  })
}
