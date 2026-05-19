import { useEffect, useLayoutEffect, useRef, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { Menu } from '@tauri-apps/api/menu'
import { Maximize2, Pause, Play, RotateCcw } from 'lucide-react'
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow'
import { useNavigate } from 'react-router-dom'
import { useTimerStore } from '../stores/timerStore'
import type { SlotStatus } from '../stores/timerStore'
import AlwaysOnTopButton from './AlwaysOnTopButton'

type UiStatus = 'idle' | 'running' | 'paused' | 'warning' | 'finished'

interface SlotData {
  id: number
  name: string
  duration: number
  remaining: number
  status: UiStatus
}

interface BackendPreset {
  id: string
  name: string
  preset_type: {
    type: 'Single' | 'Multi'
    first?: number
    loop_interval?: number
    warn_seconds?: number
    warn_text?: string
    hotkey?: string
    bar_color?: string
    text_color?: string
    slots?: {
      id: string
      name: string
      first: number
      loop_interval: number
      warn_seconds: number
      warn_text?: string
      bar_color?: string
      text_color?: string
      hotkey?: string
    }[]
  }
}

interface TimerSnapshot {
  slots: {
    id: string
    name: string
    elapsed_ms: number
    remaining_ms: number
    target_ms: number
    status: SlotStatus
    loop_count: number
  }[]
  global_status: 'idle' | 'running' | 'paused'
}

function getPresetSlots(preset?: BackendPreset) {
  if (!preset) return []
  if (preset.preset_type.type === 'Multi') return preset.preset_type.slots || []
  return [{
    id: preset.id,
    name: preset.name,
    first: preset.preset_type.first || 0,
    loop_interval: preset.preset_type.loop_interval || 0,
    warn_seconds: preset.preset_type.warn_seconds || 0,
    warn_text: preset.preset_type.warn_text,
    hotkey: preset.preset_type.hotkey,
    bar_color: preset.preset_type.bar_color,
    text_color: preset.preset_type.text_color,
  }]
}

function mapStatus(status: SlotStatus): UiStatus {
  if (status === 'triggered') return 'finished'
  return status as UiStatus
}

function fmtTime(sec: number): string {
  if (sec <= 0) return '00:00'
  const h = Math.floor(sec / 3600)
  const m = Math.floor((sec % 3600) / 60)
  const s = Math.floor(sec % 60)
  if (h > 0) return `${String(h).padStart(2, '0')}:${String(m).padStart(2, '0')}:${String(s).padStart(2, '0')}`
  return `${String(m).padStart(2, '0')}:${String(s).padStart(2, '0')}`
}

function statusColor(status: UiStatus): string {
  switch (status) {
    case 'idle': return '#C7C7CC'
    case 'running': return '#34C759'
    case 'paused': return '#86868B'
    case 'warning': return '#D4AF37'
    case 'finished': return '#FF3B30'
  }
}

function clampScale(value: number) {
  return Math.max(0.72, Math.min(1.08, Number(value.toFixed(2))))
}

function SimpleSlotRow({ slot, scale }: { slot: SlotData; scale: number }) {
  const rowRef = useRef<HTMLDivElement>(null)
  const [titleFontSize, setTitleFontSize] = useState(Math.max(12, Math.round(15 * scale)))
  const currentSlotDefs = useTimerStore((s) => s.currentSlotDefs)
  const slotDef = currentSlotDefs[slot.id]
  const progress = slot.status === 'finished'
    ? 100
    : slot.duration > 0
      ? Math.max(0, Math.min(100, (slot.remaining / slot.duration) * 100))
      : 0
  const defaultColor = statusColor(slot.status)
  const barColor = slotDef?.bar_color || defaultColor
  const textColor = slotDef?.text_color || defaultColor
  const isRunning = slot.status === 'running' || slot.status === 'warning'
  const rowRadius = Math.round(16 * scale)
  const rowGap = Math.max(8, Math.round(10 * scale))
  const actionGap = Math.max(6, Math.round(8 * scale))
  const actionSize = Math.max(36, Math.round(42 * scale))
  const actionRadius = Math.round(14 * scale)
  const timeWidth = Math.max(58, Math.round(66 * scale))
  const titleBaseSize = Math.max(12, Math.round(15 * scale))

  useLayoutEffect(() => {
    const row = rowRef.current
    if (!row) return

    const updateTitleSize = () => {
      const displayWidth = Array.from(slot.name).reduce((total, char) => {
        return total + (char.charCodeAt(0) > 255 ? 1 : 0.56)
      }, 0)
      const horizontalPadding = Math.round(28 * scale)
      const reservedWidth = timeWidth + actionSize * 2 + actionGap + rowGap * 2 + horizontalPadding
      const availableWidth = Math.max(72, row.clientWidth - reservedWidth)
      const fittedSize = Math.floor(availableWidth / Math.max(1, displayWidth))
      setTitleFontSize(Math.max(9, Math.min(titleBaseSize, fittedSize)))
    }

    updateTitleSize()
    const observer = new ResizeObserver(updateTitleSize)
    observer.observe(row)
    return () => observer.disconnect()
  }, [actionGap, actionSize, rowGap, scale, slot.name, timeWidth, titleBaseSize])

  return (
    <div
      ref={rowRef}
      title={`${slot.name} - ${fmtTime(slot.remaining)}`}
      style={{
        minHeight: 28,
        width: '100%',
        flex: '1 1 0',
        position: 'relative',
        display: 'flex',
        alignItems: 'center',
        gap: rowGap,
        padding: `0 ${Math.round(12 * scale)}px 0 ${Math.round(16 * scale)}px`,
        overflow: 'hidden',
        background: 'rgba(28,30,40,0.74)',
        borderRadius: rowRadius,
        cursor: 'move',
        color: 'inherit',
        border: '1px solid rgba(255,255,255,0.06)',
        boxShadow: 'inset 0 1px 0 rgba(255,255,255,0.05)',
        fontFamily: '"Microsoft YaHei UI", "PingFang SC", "Segoe UI", sans-serif',
      }}
    >
      <span
        style={{
          position: 'absolute',
          inset: 0,
          background: 'linear-gradient(90deg, rgba(255,255,255,0.04), rgba(255,255,255,0.015))',
        }}
      />
      <span
        style={{
          position: 'absolute',
          left: 0,
          top: 0,
          bottom: 0,
          width: `${progress}%`,
          background: barColor,
          opacity: slot.status === 'idle' ? 0.18 : 0.34,
          transition: 'width 0.5s linear',
        }}
      />
      <span
        style={{
          position: 'relative',
          zIndex: 1,
          minWidth: 0,
          flex: '1 1 auto',
          overflow: 'visible',
          whiteSpace: 'nowrap',
          fontSize: titleFontSize,
          fontWeight: 650,
          lineHeight: 1,
          color: '#F5F5F7',
          textShadow: '0 1px 1px rgba(0,0,0,0.40)',
          textAlign: 'left',
          WebkitFontSmoothing: 'antialiased',
        }}
      >
        {slot.name}
      </span>
      <span
        style={{
          position: 'relative',
          zIndex: 1,
          width: timeWidth,
          flexShrink: 0,
          color: textColor || '#fff',
          fontFamily: '"Bahnschrift", "Segoe UI Variable Display", "Segoe UI", sans-serif',
          fontSize: Math.max(18, Math.round(23 * scale)),
          fontWeight: 700,
          letterSpacing: 0,
          lineHeight: 1,
          fontVariantNumeric: 'tabular-nums',
          textAlign: 'right',
          textShadow: '0 1px 1px rgba(0,0,0,0.46)',
          WebkitFontSmoothing: 'antialiased',
        }}
      >
        {fmtTime(slot.remaining)}
      </span>
      <span
        style={{
          position: 'relative',
          zIndex: 1,
          width: actionSize * 2 + actionGap,
          display: 'flex',
          justifyContent: 'flex-end',
          gap: actionGap,
          flexShrink: 0,
        }}
      >
        <button
          onClick={() => invoke(isRunning ? 'pause_slot' : 'start_slot', { index: slot.id })}
          title={isRunning ? '停止计时' : '开始计时'}
          style={{
            width: actionSize,
            height: actionSize,
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            padding: 0,
            borderRadius: actionRadius,
            border: '1px solid rgba(255,255,255,0.06)',
            background: isRunning ? 'rgba(255,255,255,0.09)' : 'rgba(52,199,89,0.22)',
            color: '#fff',
            cursor: 'pointer',
            boxShadow: 'inset 0 1px 0 rgba(255,255,255,0.10)',
            transition: 'background 0.16s ease, border-color 0.16s ease',
          }}
        >
          {isRunning ? <Pause size={Math.max(15, Math.round(18 * scale))} /> : <Play size={Math.max(15, Math.round(18 * scale))} />}
        </button>
        <button
          onClick={() => invoke('reset_slot', { index: slot.id })}
          title="重置"
          style={{
            width: actionSize,
            height: actionSize,
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            padding: 0,
            borderRadius: actionRadius,
            border: '1px solid rgba(255,255,255,0.06)',
            background: 'rgba(255,255,255,0.09)',
            color: '#fff',
            cursor: 'pointer',
            boxShadow: 'inset 0 1px 0 rgba(255,255,255,0.10)',
            transition: 'background 0.16s ease, border-color 0.16s ease',
          }}
        >
          <RotateCcw size={Math.max(15, Math.round(18 * scale))} />
        </button>
      </span>
    </div>
  )
}

export default function SimpleModeShell() {
  const navigate = useNavigate()
  const { slots: storeSlots, setSlots, setCurrentSlotDefs, setGlobalStatus, setSimpleMode } = useTimerStore()
  const [scale, setScale] = useState(0.88)

  useEffect(() => {
    invoke<number>('get_simple_mode_scale')
      .then((value) => setScale(clampScale(value)))
      .catch(() => {})
  }, [])

  useEffect(() => {
    if (storeSlots.length > 0) return
    invoke<BackendPreset[]>('list_presets').then(async (presets) => {
      const snapshot = await invoke<TimerSnapshot>('get_timer_snapshot').catch(() => ({ slots: [], global_status: 'idle' as const }))
      if (snapshot.slots.length > 0) {
        setSlots(snapshot.slots.map((slot) => ({
          id: slot.id,
          name: slot.name,
          elapsedMs: slot.elapsed_ms,
          remainingMs: slot.remaining_ms,
          targetMs: slot.target_ms,
          status: slot.status,
          loopCount: slot.loop_count,
          warnFired: false,
        })))
        setGlobalStatus(snapshot.global_status)
        await invoke('enter_simple_mode')
        return
      }
      const preset = presets[0]
      if (!preset) return
      await invoke('select_preset', { id: preset.id })
      await invoke('enter_simple_mode')
      setCurrentSlotDefs(getPresetSlots(preset))
    })
  }, [setCurrentSlotDefs, setGlobalStatus, setSlots, storeSlots.length])

  const slots: SlotData[] = storeSlots.map((slot, index) => ({
    id: index,
    name: slot.name,
    duration: slot.targetMs / 1000,
    remaining: Math.max(0, slot.remainingMs / 1000),
    status: mapStatus(slot.status),
  }))

  const exitSimpleMode = async () => {
    setSimpleMode(false)
    await invoke('set_simple_mode', { enabled: false })
    await invoke('exit_simple_mode')
  }

  const expandAndNavigate = async (path: string) => {
    await exitSimpleMode()
    navigate(path)
  }

  const applyScale = (nextScale: number) => {
    const normalized = clampScale(nextScale)
    setScale(normalized)
    invoke<number>('set_simple_mode_scale', { scale: normalized })
      .then((value) => setScale(clampScale(value)))
      .catch(() => {})
  }

  const openNativeMenu = async () => {
    const menu = await Menu.new({
      items: [
        { id: 'timer', text: '计时器', action: () => void expandAndNavigate('/') },
        { id: 'characters', text: '角色 CD', action: () => void expandAndNavigate('/characters') },
        { id: 'settings', text: '设置', action: () => void expandAndNavigate('/settings') },
        { item: 'Separator' },
        { id: 'scale-down', text: '缩小一点', action: () => applyScale(scale - 0.08) },
        { id: 'scale-up', text: '放大一点', action: () => applyScale(scale + 0.08) },
        { id: 'scale-reset', text: '默认大小', action: () => applyScale(0.88) },
        { item: 'Separator' },
        { id: 'restore', text: '恢复正常模式', action: () => void exitSimpleMode() },
      ],
    })
    await menu.popup(undefined, getCurrentWebviewWindow())
  }

  const startDragging = (event: React.MouseEvent) => {
    if (event.button !== 0) return
    if ((event.target as HTMLElement).closest('button')) return
    void invoke('begin_window_drag', { dock_on_release: true })
  }
  const sideWidth = Math.max(46, Math.round(50 * scale))
  const sideGap = Math.max(5, Math.round(6 * scale))
  const sideButtonSize = Math.max(34, Math.round(42 * scale))
  const sideRadius = Math.round(22 * scale)

  return (
    <div
      onMouseDown={startDragging}
      onWheel={(event) => {
        if (!event.ctrlKey) return
        event.preventDefault()
        applyScale(scale + (event.deltaY > 0 ? -0.04 : 0.04))
      }}
      onContextMenu={(event) => {
        event.preventDefault()
        void openNativeMenu()
      }}
      style={{
        height: '100dvh',
        display: 'flex',
        overflow: 'hidden',
        background: 'transparent',
        color: '#fff',
        padding: Math.round(8 * scale),
        boxSizing: 'border-box',
      }}
    >
      <div
        style={{
          flex: 1,
          minWidth: 0,
          display: 'flex',
          flexDirection: 'column',
          justifyContent: 'stretch',
          gap: Math.round(10 * scale),
          padding: Math.round(10 * scale),
          borderRadius: sideRadius,
          background: 'linear-gradient(180deg, rgba(30,34,45,0.86), rgba(21,25,34,0.90))',
          border: '1px solid rgba(255,255,255,0.07)',
          boxShadow: 'inset 0 1px 0 rgba(255,255,255,0.08)',
        }}
      >
        {slots.map((slot) => (
          <SimpleSlotRow key={slot.id} slot={slot} scale={scale} />
        ))}
      </div>
      <div
        style={{
          width: sideWidth,
          flexShrink: 0,
          display: 'flex',
          flexDirection: 'column',
          alignItems: 'center',
          justifyContent: 'center',
          gap: sideGap,
          marginLeft: Math.max(5, Math.round(6 * scale)),
          borderRadius: sideRadius,
          border: '1px solid rgba(255,255,255,0.07)',
          background: 'linear-gradient(180deg, rgba(30,34,45,0.86), rgba(21,25,34,0.90))',
          boxShadow: 'inset 0 1px 0 rgba(255,255,255,0.08)',
        }}
      >
        <AlwaysOnTopButton compact compactSize={sideButtonSize} />
        <button
          onClick={exitSimpleMode}
          title="恢复正常模式"
          style={{
            width: sideButtonSize,
            height: sideButtonSize,
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            borderRadius: Math.max(10, Math.round(sideButtonSize * 0.31)),
            border: '1px solid rgba(255,255,255,0.06)',
            background: 'rgba(255,255,255,0.09)',
            color: 'rgba(255,255,255,0.82)',
            cursor: 'pointer',
          }}
        >
          <Maximize2 size={Math.max(14, Math.round(sideButtonSize * 0.4))} />
        </button>
      </div>
    </div>
  )
}
