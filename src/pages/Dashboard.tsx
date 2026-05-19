import { useState, useEffect, useRef } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { useNavigate } from 'react-router-dom'
import { useTimerStore } from '../stores/timerStore'
import type { SlotStatus } from '../stores/timerStore'
import { Play, Pause, RotateCcw, ChevronDown, ClipboardList } from 'lucide-react'

/* ─── Types ─── */

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
    slots?: { id: string; name: string; first: number; loop_interval: number; warn_seconds: number; bar_color?: string; text_color?: string; hotkey?: string }[]
  }
}

interface TimerSnapshot {
  slots: unknown[]
}

/* ─── Helpers ─── */

const NORMAL_WINDOW_WIDTH = 340
const WINDOW_HEIGHT = 500

function mapStatus(s: SlotStatus): UiStatus {
  if (s === 'triggered') return 'finished'
  return s as UiStatus
}

function fmtTime(sec: number): string {
  if (sec <= 0) return '00:00'
  const h = Math.floor(sec / 3600)
  const m = Math.floor((sec % 3600) / 60)
  const s = Math.floor(sec % 60)
  if (h > 0) return `${String(h).padStart(2, '0')}:${String(m).padStart(2, '0')}:${String(s).padStart(2, '0')}`
  return `${String(m).padStart(2, '0')}:${String(s).padStart(2, '0')}`
}

function statusColor(s: UiStatus): string {
  switch (s) {
    case 'idle': return '#C7C7CC'
    case 'running': return '#34C759'
    case 'paused': return '#86868B'
    case 'warning': return '#D4AF37'
    case 'finished': return '#FF3B30'
  }
}

function statusLabel(s: UiStatus): string {
  switch (s) {
    case 'idle': return '就绪'
    case 'running': return '计时中'
    case 'paused': return '已暂停'
    case 'warning': return '警告'
    case 'finished': return '时间到'
  }
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

/* ─── Full Mode Slot Card ─── */

function SlotCard({ slot, barColor, textColor }: { slot: SlotData; barColor?: string; textColor?: string }) {
  const progress = slot.duration > 0
    ? Math.max(0, Math.min(100, (slot.remaining / slot.duration) * 100))
    : 0
  const defaultColor = statusColor(slot.status)
  const color = textColor || defaultColor
  const barC = barColor || defaultColor

  const handleToggle = () => {
    invoke('toggle_slot', { index: slot.id })
  }

  return (
    <div className="card" style={{ padding: 10, display: 'flex', flexDirection: 'column', gap: 6 }}>
      {/* Top row: name + time + controls */}
      <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
        {/* Name + status */}
        <div style={{ display: 'flex', flexDirection: 'column', gap: 2, minWidth: 0, flex: '1 1 auto' }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: 6 }}>
            <span style={{ fontSize: 10, fontWeight: 600, color: '#86868B', background: '#F5F5F7', padding: '1px 5px', borderRadius: 3, flexShrink: 0 }}>
              S{slot.id}
            </span>
            <span style={{ minWidth: 0, fontSize: 11, color: '#1D1D1F', fontWeight: 600, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }} title={slot.name}>
              {slot.name}
            </span>
          </div>
          <span style={{ fontSize: 10, color: color, fontWeight: 500, marginLeft: 24 }}>
            {statusLabel(slot.status)}
          </span>
        </div>

        {/* Timer */}
        <span style={{ width: 66, fontSize: 22, fontWeight: 300, color: color, fontVariantNumeric: 'tabular-nums', flexShrink: 0, lineHeight: 1, textAlign: 'right' }}>
          {fmtTime(slot.remaining)}
        </span>

        {/* Controls */}
        <div style={{ width: slot.status === 'running' ? 56 : 26, display: 'flex', justifyContent: 'flex-end', gap: 4, flexShrink: 0 }}>
          <button
            className="btn-icon"
            onClick={handleToggle}
            title={slot.status === 'running' ? '重置' : '开始'}
            style={{ width: 26, height: 26 }}
          >
            {slot.status === 'running' ? <RotateCcw size={12} /> : <Play size={12} />}
          </button>
          {slot.status === 'running' && (
            <button
              className="btn-icon"
              onClick={() => invoke('pause_slot', { index: slot.id })}
              title="暂停"
              style={{ width: 26, height: 26 }}
            >
              <Pause size={12} />
            </button>
          )}
        </div>
      </div>

      {/* Progress bar */}
      <div style={{ height: 6, background: '#F0F0F2', borderRadius: 3, overflow: 'hidden' }}>
        <div
          style={{
            height: '100%',
            width: `${progress}%`,
            background: barC,
            borderRadius: 3,
            transition: 'width 0.5s linear',
          }}
        />
      </div>
    </div>
  )
}

/* ─── Full Mode View ─── */

function FullModeView({ slots }: { slots: SlotData[] }) {
  const { currentSlotDefs } = useTimerStore()

  return (
    <div style={{ flex: 1, overflow: 'auto', display: 'flex', flexDirection: 'column', gap: 8, paddingBottom: 8 }}>
      {slots.map((slot) => (
        <SlotCard
          key={slot.id}
          slot={slot}
          barColor={currentSlotDefs[slot.id]?.bar_color}
          textColor={currentSlotDefs[slot.id]?.text_color}
        />
      ))}
    </div>
  )
}

/* ─── Dashboard Page ─── */

export default function Dashboard() {
  const navigate = useNavigate()
  const { slots: storeSlots, setCurrentPresetId, setCurrentSlotDefs } = useTimerStore()

  useEffect(() => {
    invoke('resize_window', { width: NORMAL_WINDOW_WIDTH, height: WINDOW_HEIGHT }).catch(() => {})
  }, [])

  const slots: SlotData[] = storeSlots.map((s, i) => ({
    id: i,
    name: s.name,
    duration: s.targetMs / 1000,
    remaining: Math.max(0, s.remainingMs / 1000),
    status: mapStatus(s.status),
  }))

  const [templates, setTemplates] = useState<{ id: string; name: string }[]>([])
  const [selected, setSelected] = useState<string>('')
  const [dropdownOpen, setDropdownOpen] = useState(false)
  const dropdownRef = useRef<HTMLDivElement>(null)

  /* Load presets */
  useEffect(() => {
    invoke<BackendPreset[]>('list_presets').then(async (presets) => {
      const list = presets.map((p) => ({ id: p.id, name: p.name }))
      setTemplates(list)
      if (list.length > 0) {
        const currentPresetId = await invoke<string | null>('get_current_preset_id').catch(() => null)
        const snapshot = await invoke<TimerSnapshot>('get_timer_snapshot').catch(() => ({ slots: [] }))
        const hasRuntimeSlots = snapshot.slots.length > 0 || useTimerStore.getState().slots.length > 0
        const activeId = currentPresetId || (!hasRuntimeSlots ? list[0].id : '')
        const preset = presets.find((p) => p.id === activeId)

        if (preset) {
          setSelected(preset.name)
          setCurrentPresetId(preset.id)
          setCurrentSlotDefs(getPresetSlots(preset))
        } else {
          setSelected('')
          setCurrentPresetId(null)
          setCurrentSlotDefs([])
        }

        if (!hasRuntimeSlots && activeId) {
          invoke('select_preset', { id: activeId })
        }
      }
    })
  }, [setCurrentPresetId, setCurrentSlotDefs])

  /* Close dropdown on outside click */
  useEffect(() => {
    function handler(e: MouseEvent) {
      if (dropdownRef.current && !dropdownRef.current.contains(e.target as Node)) {
        setDropdownOpen(false)
      }
    }
    document.addEventListener('click', handler)
    return () => document.removeEventListener('click', handler)
  }, [])

  const handleSelectTemplate = async (t: { id: string; name: string }) => {
    await invoke('select_preset', { id: t.id })
    setSelected(t.name)
    setCurrentPresetId(t.id)
    setDropdownOpen(false)
    const presets = await invoke<BackendPreset[]>('list_presets')
    const preset = presets.find((p) => p.id === t.id)
    setCurrentSlotDefs(getPresetSlots(preset))
  }

  return (
    <div style={{ flex: 1, display: 'flex', flexDirection: 'column', padding: '12px 12px 0', gap: 10, overflow: 'hidden' }}>
      {/* Header: preset selector */}
      <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
        <span style={{ fontSize: 12, fontWeight: 600, color: '#86868B' }}>预设</span>
        <div ref={dropdownRef} style={{ position: 'relative', flex: 1 }}>
          <button
            className="btn btn-ghost"
            onClick={() => setDropdownOpen((v) => !v)}
            style={{ width: '100%', justifyContent: 'space-between' }}
          >
            <span style={{ overflow: 'hidden', textOverflow: 'ellipsis' }}>{selected || '选择预设'}</span>
            <ChevronDown size={12} />
          </button>
          {dropdownOpen && (
            <div
              className="card"
              style={{
                position: 'absolute',
                top: 'calc(100% + 4px)',
                left: 0,
                right: 0,
                zIndex: 100,
                padding: '4px 0',
                boxShadow: '0 8px 24px rgba(0,0,0,0.1)',
              }}
            >
              {templates.map((t) => (
                <button
                  key={t.id}
                  onClick={() => handleSelectTemplate(t)}
                  style={{
                    width: '100%',
                    height: 32,
                    padding: '0 12px',
                    display: 'flex',
                    alignItems: 'center',
                    fontSize: 12,
                    color: selected === t.name ? '#D4AF37' : '#1D1D1F',
                    background: selected === t.name ? '#F5F5F7' : 'transparent',
                    border: 'none',
                    borderLeft: selected === t.name ? '2px solid #D4AF37' : '2px solid transparent',
                    cursor: 'pointer',
                    textAlign: 'left',
                  }}
                >
                  {t.name}
                </button>
              ))}
            </div>
          )}
        </div>
        <button
          className="btn btn-ghost"
          onClick={() => navigate('/presets')}
          title="管理模板"
          style={{ flexShrink: 0, padding: '0 10px' }}
        >
          <ClipboardList size={13} />
          管理
        </button>
      </div>

      {/* Slot list */}
      <FullModeView slots={slots} />
    </div>
  )
}

