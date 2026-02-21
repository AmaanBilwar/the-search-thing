import { useEffect, useState } from 'react'
import { cn } from '@/lib/utils'
import { useKeybinds } from '@/app/hooks/use-keybinds'
import {
  type KeybindAction,
  type KeyCombo,
  KEYBIND_ACTIONS,
  DEFAULT_KEYBINDS,
  comboFromEvent,
  comboTokens,
  combosEqual,
  findConflict,
} from '@/lib/storage/keybind-store'

function KeyToken({ children }: { children: string }) {
  return <kbd className="px-2 py-1 text-xs text-zinc-300 bg-zinc-700/50 border border-zinc-600 rounded">{children}</kbd>
}

function ComboDisplay({ combo }: { combo: KeyCombo }) {
  const tokens = comboTokens(combo)
  return (
    <div className="flex items-center gap-2">
      {tokens.map((token, i) => (
        <span key={i} className="flex items-center gap-2">
          {i > 0 && <span className="text-xs text-zinc-500">+</span>}
          <KeyToken>{token}</KeyToken>
        </span>
      ))}
    </div>
  )
}

type KeybindRowProps = {
  action: KeybindAction
  label: string
  description: string
  combo: KeyCombo
  isRecording: boolean
  onStartRecording: () => void
  onCancelRecording: () => void
  onRecorded: (combo: KeyCombo) => void
  conflict: string | null
}

function KeybindRow({
  label,
  description,
  combo,
  isRecording,
  onStartRecording,
  onCancelRecording,
  onRecorded,
  conflict,
}: KeybindRowProps) {
  useEffect(() => {
    if (!isRecording) return

    const handleKeyDown = (e: KeyboardEvent) => {
      e.preventDefault()
      e.stopPropagation()

      if (e.key === 'Escape') {
        onCancelRecording()
        return
      }

      const newCombo = comboFromEvent(e)
      if (newCombo) {
        onRecorded(newCombo)
      }
    }

    window.addEventListener('keydown', handleKeyDown, true)
    return () => window.removeEventListener('keydown', handleKeyDown, true)
  }, [isRecording, onCancelRecording, onRecorded])

  return (
    <div
      className={cn(
        'flex items-center justify-between gap-4 px-3 py-2 rounded-md transition-colors',
        isRecording && 'bg-zinc-700/40 ring-1 ring-gray-500/50'
      )}
    >
      <div className="flex flex-col gap-0.5 min-w-0">
        <div className="text-sm text-zinc-200">{label}</div>
        <div className="text-xs text-zinc-500">{description}</div>
        {conflict && <div className="text-xs text-gray-400 mt-0.5">Conflicts with: {conflict}</div>}
      </div>

      <div className="flex items-center gap-3 flex-shrink-0">
        {isRecording ? (
          <div className="flex items-center gap-2">
            <span className="text-xs text-gray-400 animate-pulse">Press keys…</span>
            <button
              onClick={onCancelRecording}
              className="text-xs text-zinc-500 hover:text-zinc-300 transition-colors px-1.5 py-0.5 rounded border border-zinc-600/50 hover:border-zinc-500"
            >
              Esc
            </button>
          </div>
        ) : (
          <button
            onClick={onStartRecording}
            className={cn(
              'flex items-center gap-2 group cursor-pointer',
              'rounded-md px-2 py-1 -mx-2 -my-1',
              'hover:bg-zinc-700/50 transition-colors'
            )}
            title="Click to rebind"
          >
            <ComboDisplay combo={combo} />
          </button>
        )}
      </div>
    </div>
  )
}

export default function Keybinds() {
  const { keybinds, updateKeybind, resetKeybinds } = useKeybinds()
  const [recordingAction, setRecordingAction] = useState<KeybindAction | null>(null)
  const [conflict, setConflict] = useState<{ action: KeybindAction; conflictsWith: string } | null>(null)

  const handleStartRecording = (action: KeybindAction) => {
    setConflict(null)
    setRecordingAction(action)
  }

  const handleCancelRecording = () => {
    setRecordingAction(null)
  }

  const handleRecorded = (action: KeybindAction, combo: KeyCombo) => {
    const conflictingAction = findConflict(combo, keybinds, action)
    if (conflictingAction) {
      const meta = KEYBIND_ACTIONS.find((m) => m.action === conflictingAction)
      setConflict({ action, conflictsWith: meta?.label ?? conflictingAction })
      // Still stop recording so the user can see the conflict message
      setRecordingAction(null)
      return
    }

    setConflict(null)
    updateKeybind(action, combo)
    setRecordingAction(null)
  }

  const hasCustomBindings = KEYBIND_ACTIONS.some(
    ({ action }) => !combosEqual(keybinds[action], DEFAULT_KEYBINDS[action])
  )

  return (
    <div
      className={cn(
        'flex flex-col gap-4',
        'w-full h-full',
        'border-1 border-zinc-700/80 bg-zinc-800/60',
        'p-4 shadow-[0_0_0_1px_rgba(255,255,255,0.03)]'
      )}
    >
      <div className="flex items-center justify-between">
        <div className="text-xs uppercase tracking-wider text-zinc-500">Keybinds</div>
        {hasCustomBindings && (
          <button
            onClick={() => {
              setConflict(null)
              setRecordingAction(null)
              resetKeybinds()
            }}
            className="text-xs text-zinc-500 hover:text-zinc-300 transition-colors px-2 py-1 rounded border border-zinc-700 hover:border-zinc-500"
          >
            Reset all
          </button>
        )}
      </div>

      <div className="flex flex-col gap-1">
        {KEYBIND_ACTIONS.map(({ action, label, description }) => (
          <KeybindRow
            key={action}
            action={action}
            label={label}
            description={description}
            combo={keybinds[action]}
            isRecording={recordingAction === action}
            onStartRecording={() => handleStartRecording(action)}
            onCancelRecording={handleCancelRecording}
            onRecorded={(combo) => handleRecorded(action, combo)}
            conflict={conflict?.action === action ? conflict.conflictsWith : null}
          />
        ))}
      </div>

      {/* Static Enter row — not customizable per user request */}
      <div className="flex items-center justify-between gap-4 px-3 py-2 opacity-50">
        <div className="flex flex-col gap-0.5">
          <div className="text-sm text-zinc-200">Open selected result</div>
          <div className="text-xs text-zinc-500">Open the highlighted result.</div>
        </div>
        <div className="flex items-center gap-2">
          <KeyToken>Enter</KeyToken>
        </div>
      </div>

      <div className="text-[11px] text-zinc-600 mt-auto">
        Click a shortcut to rebind it. Press <kbd className="px-1 text-zinc-500">Esc</kbd> to cancel.
      </div>
    </div>
  )
}
