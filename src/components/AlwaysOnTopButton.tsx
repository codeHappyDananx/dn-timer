import { useEffect, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { Pin } from 'lucide-react'

export default function AlwaysOnTopButton({ compact = false, compactSize = 42 }: { compact?: boolean; compactSize?: number }) {
  const [enabled, setEnabled] = useState(false)
  const size = compact ? compactSize : 28
  const radius = compact ? Math.max(10, Math.round(compactSize * 0.31)) : 8
  const iconSize = compact ? Math.max(13, Math.round(compactSize * 0.38)) : 14

  useEffect(() => {
    invoke<boolean>('get_always_on_top').then(setEnabled).catch(() => {})
  }, [])

  const toggle = async () => {
    const next = !enabled
    setEnabled(next)
    await invoke('set_always_on_top', { always_on_top: next })
  }

  return (
    <button
      onClick={toggle}
      title={enabled ? '取消置顶' : '置顶窗口'}
      style={{
        width: size,
        height: size,
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        padding: 0,
        borderRadius: radius,
        border: enabled ? '1px solid rgba(212,175,55,0.55)' : '1px solid rgba(134,134,139,0.22)',
        background: enabled ? 'rgba(212,175,55,0.18)' : compact ? 'rgba(255,255,255,0.08)' : '#FFFFFF',
        color: enabled ? '#D4AF37' : compact ? 'rgba(255,255,255,0.70)' : '#86868B',
        cursor: 'pointer',
        boxShadow: enabled ? 'inset 0 1px 0 rgba(255,255,255,0.10), 0 0 0 2px rgba(212,175,55,0.12)' : compact ? 'inset 0 1px 0 rgba(255,255,255,0.08)' : 'none',
      }}
    >
      <Pin size={iconSize} fill={enabled ? 'currentColor' : 'none'} />
    </button>
  )
}
