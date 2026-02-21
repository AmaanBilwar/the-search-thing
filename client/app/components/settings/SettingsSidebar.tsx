import { cn } from '@/lib/utils'
import type { ComponentType } from 'react'
import { Settings, Command } from 'lucide-react'
import about from '@/resources/about.svg'

const items = ['General', 'Keybinds', 'About'] as const

type IconSpec =
  | { type: 'lucide'; Icon: ComponentType<{ className?: string }> }
  | { type: 'image'; src: string; alt: string }

const icons: Record<(typeof items)[number], IconSpec> = {
  General: { type: 'lucide', Icon: Settings },
  About: { type: 'image', src: about, alt: 'About' },
  Keybinds: { type: 'lucide', Icon: Command },
}

type SettingsSideBarProps = {
  selectedItem: string
  onSelect: (item: string) => void
}

export default function SettingsSidebar({ selectedItem, onSelect }: SettingsSideBarProps) {
  return (
    <div
      className={cn(
        'flex flex-col gap-2',
        'w-56 flex-none',
        'border-1 border-zinc-700/80 bg-zinc-800/60',
        'p-3 shadow-[0_0_0_1px_rgba(255,255,255,0.03)]'
      )}
    >
      <div className="text-xs uppercase tracking-wider text-zinc-500 px-1">Settings</div>
      <nav className="flex flex-col gap-1">
        {items.map((label) => {
          const icon = icons[label]

          return (
            <button
              key={label}
              type="button"
              onClick={() => onSelect(label)}
              className={cn(
                'flex items-center justify-start gap-2',
                'h-9 w-full rounded-md px-2',
                selectedItem === label ? 'bg-zinc-700/60 text-zinc-100' : 'text-zinc-300 hover:text-zinc-100',
                'hover:bg-zinc-700/60',
                'transition-colors duration-150 opacity-95'
              )}
            >
              {icon.type === 'lucide' ? (
                <icon.Icon className="h-4 w-4 opacity-75" />
              ) : (
                <img src={icon.src} alt={icon.alt} className="w-4 h-4 opacity-75" />
              )}
              {label}
            </button>
          )
        })}
      </nav>
    </div>
  )
}
