import { useRef } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { Timer } from 'lucide-react'
import { useTimerStore } from '../stores/timerStore'

export default function DockedBubble({ preview = false }: { preview?: boolean }) {
  const setDockedMode = useTimerStore((s) => s.setDockedMode)
  const pointerDownRef = useRef<{ x: number; y: number } | null>(null)
  const runningCount = useTimerStore((s) =>
    s.slots.filter((slot) => slot.status === 'running' || slot.status === 'warning').length
  )

  const restoreWindow = async () => {
    setDockedMode(false)
    await invoke('undock_window').catch(() => {
      setDockedMode(true)
    })
  }

  const handleMouseDown = (event: React.MouseEvent) => {
    if (preview) return
    if (event.button !== 0) return
    pointerDownRef.current = { x: event.screenX, y: event.screenY }
    void invoke('begin_window_drag', { dock_on_release: false })
  }

  const handleClick = (event: React.MouseEvent) => {
    const start = pointerDownRef.current
    pointerDownRef.current = null
    if (preview) return
    if (!start) return

    const moved = Math.hypot(event.screenX - start.x, event.screenY - start.y)
    if (moved <= 4) {
      void restoreWindow()
    }
  }

  return (
    <div
      style={{
        width: '100dvw',
        height: '100dvh',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        background: 'transparent',
        overflow: 'hidden',
        userSelect: 'none',
        pointerEvents: 'auto',
      }}
    >
      <div
        role="button"
        tabIndex={0}
        onMouseDown={handleMouseDown}
        onClick={handleClick}
        title="点击展开"
        style={{
          position: 'relative',
          width: 48,
          height: 48,
          borderRadius: '50%',
          border: '1px solid rgba(255,255,255,0.9)',
          background: 'linear-gradient(145deg, rgba(255,255,255,0.98) 0%, rgba(238,241,246,0.96) 58%, rgba(219,225,235,0.95) 100%)',
          color: '#B08A16',
          opacity: preview ? 0.76 : 1,
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          padding: 0,
          outline: 'none',
          appearance: 'none',
          cursor: preview ? 'default' : 'pointer',
          boxShadow: 'inset 0 1px 0 rgba(255,255,255,0.95), inset 0 -8px 14px rgba(80,92,116,0.18)',
          transition: 'opacity 120ms ease, transform 120ms ease',
          transform: preview ? 'scale(0.94)' : 'scale(1)',
        }}
      >
        <span
          style={{
            position: 'absolute',
            inset: 3,
            borderRadius: '50%',
            background: 'radial-gradient(circle at 30% 18%, rgba(255,255,255,0.95), rgba(255,255,255,0.32) 34%, rgba(255,255,255,0) 64%)',
            pointerEvents: 'none',
          }}
        />
        <span
          style={{
            position: 'absolute',
            inset: 7,
            borderRadius: '50%',
            border: '1px solid rgba(176,138,22,0.20)',
            background: 'rgba(255,255,255,0.24)',
            pointerEvents: 'none',
          }}
        />
        <Timer size={19} strokeWidth={2.35} style={{ position: 'relative', zIndex: 1 }} />
        {runningCount > 0 && (
          <span
            style={{
              position: 'absolute',
              right: 7,
              top: 7,
              width: 7,
              height: 7,
              borderRadius: '50%',
              background: '#34C759',
              boxShadow: '0 0 0 2px rgba(255,255,255,0.95)',
              zIndex: 2,
            }}
          />
        )}
      </div>
    </div>
  )
}
