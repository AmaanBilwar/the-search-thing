import { useCallback, useEffect, useState } from 'react'

import {
  type KeybindMap,
  type KeybindAction,
  type KeyCombo,
  loadKeybinds,
  saveKeybinds,
  updateKeybind as storeUpdateKeybind,
  resetKeybinds as storeResetKeybinds,
  KEYBIND_CHANGE_EVENT,
} from '@/lib/storage/keybind-store'

export function useKeybinds() {
  const [keybinds, setKeybinds] = useState<KeybindMap>(loadKeybinds)

  // Re-read from localStorage whenever any part of the app writes new bindings.
  useEffect(() => {
    const sync = () => setKeybinds(loadKeybinds())

    window.addEventListener(KEYBIND_CHANGE_EVENT, sync)
    window.addEventListener('storage', sync)

    return () => {
      window.removeEventListener(KEYBIND_CHANGE_EVENT, sync)
      window.removeEventListener('storage', sync)
    }
  }, [])

  const updateKeybind = useCallback((action: KeybindAction, combo: KeyCombo) => {
    storeUpdateKeybind(action, combo)
    setKeybinds((prev) => ({ ...prev, [action]: combo }))
  }, [])

  const setAllKeybinds = useCallback((map: KeybindMap) => {
    saveKeybinds(map)
    setKeybinds(map)
  }, [])

  const resetKeybinds = useCallback(() => {
    storeResetKeybinds()
    setKeybinds(loadKeybinds())
  }, [])

  return {
    keybinds,
    updateKeybind,
    setAllKeybinds,
    resetKeybinds,
  } as const
}
