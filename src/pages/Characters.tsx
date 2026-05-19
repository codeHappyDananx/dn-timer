import { useState, useEffect, useCallback, useMemo, useRef, type MouseEvent as ReactMouseEvent } from 'react'
import { createPortal } from 'react-dom'
import { invoke } from '@tauri-apps/api/core'
import { Plus, Minus, Trash2, RotateCcw, ChevronDown, ChevronLeft, ChevronRight, Filter, Save } from 'lucide-react'
import { dragonNestClasses, getClassByKey, getClassOrder } from '../data/dragonNestClasses'

/* ─── Types ─── */

interface Character { id: string; name: string; class_key?: string | null; note?: string | null }
interface Dungeon { id: string; name: string; short_name: string; max_clears: number; reset_day: number; reset_hour: number; note?: string | null }
interface ClearRecord { character_id: string; dungeon_id: string; current_clears: number }

const CHARACTER_COLUMN_WIDTH = 148
const CLASS_COLUMN_WIDTH = 112
const NOTE_COLUMN_WIDTH = 210
const DUNGEON_COLUMN_WIDTH = 82
const CHARACTER_PAGE_CHROME_WIDTH = 24
const NORMAL_WINDOW_WIDTH = 340
const WINDOW_HEIGHT = 500

const weekOptions = [
  { value: 1, label: '每周一' },
  { value: 2, label: '每周二' },
  { value: 3, label: '每周三' },
  { value: 4, label: '每周四' },
  { value: 5, label: '每周五' },
  { value: 6, label: '每周六' },
  { value: 0, label: '每周日' },
]

/* ─── Helpers ─── */

function cellColor(clears: number, max: number): { bg: string; text: string } {
  if (clears >= max) return { bg: '#F0FFF4', text: '#248A3D' }
  return { bg: '#FFF5F5', text: '#D70015' }
}

function ClassIcon({ classKey, size = 24 }: { classKey?: string | null; size?: number }) {
  const item = getClassByKey(classKey)
  if (!item) {
    return (
      <div style={{ width: size, height: size, borderRadius: 6, background: '#F5F5F7', color: '#86868B', display: 'grid', placeItems: 'center', fontSize: 10, fontWeight: 700, flexShrink: 0 }}>
        ?
      </div>
    )
  }
  return (
    <img
      src={item.icon}
      alt={item.name}
      title={`${item.base} · ${item.name}`}
      style={{ width: size, height: size, borderRadius: 6, objectFit: 'cover', flexShrink: 0, background: '#1D1D1F' }}
    />
  )
}

function ClassSelect({
  value,
  onChange,
  compact = false,
  showIcon = true,
}: {
  value?: string | null
  onChange: (value: string | null) => void
  compact?: boolean
  showIcon?: boolean
}) {
  const [open, setOpen] = useState(false)
  const ref = useRef<HTMLDivElement>(null)
  const menuRef = useRef<HTMLDivElement>(null)
  const [menuPos, setMenuPos] = useState({ top: 0, left: 0 })
  const selected = getClassByKey(value)

  useEffect(() => {
    function handler(e: MouseEvent) {
      const target = e.target as Node
      if (ref.current?.contains(target) || menuRef.current?.contains(target)) return
      setOpen(false)
    }
    document.addEventListener('click', handler)
    return () => document.removeEventListener('click', handler)
  }, [])

  const openMenu = () => {
    if (ref.current) {
      const rect = ref.current.getBoundingClientRect()
      setMenuPos({
        top: Math.min(rect.bottom + 4, window.innerHeight - 288),
        left: Math.min(rect.left, window.innerWidth - 218),
      })
    }
    setOpen((v) => !v)
  }

  return (
    <div ref={ref} style={{ position: 'relative', minWidth: 0 }}>
      <button
        type="button"
        onClick={openMenu}
        style={{
          width: compact ? 30 : '100%',
          height: compact ? 30 : 30,
          display: 'flex',
          alignItems: 'center',
          justifyContent: compact ? 'center' : 'space-between',
          gap: 8,
          padding: compact ? 0 : '0 10px',
          border: '1px solid #E5E5EA',
          borderRadius: 8,
          background: '#fff',
          color: '#1D1D1F',
          cursor: 'pointer',
          minWidth: 0,
        }}
        title={selected ? `${selected.base} · ${selected.name}` : '选择职业'}
      >
        <span style={{ display: 'flex', alignItems: 'center', gap: 8, minWidth: 0 }}>
          {showIcon && <ClassIcon classKey={value} size={compact ? 24 : 22} />}
          {!compact && (
            <span style={{ minWidth: 0, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap', fontSize: 12, fontWeight: 600 }}>
              {selected ? selected.name : '选择职业'}
            </span>
          )}
        </span>
        {!compact && <ChevronDown size={12} color="#86868B" />}
      </button>
      {open && createPortal(
        <div ref={menuRef} className="card" style={{ position: 'fixed', top: menuPos.top, left: menuPos.left, width: 210, maxHeight: 280, overflow: 'auto', zIndex: 9999, padding: 4, boxShadow: '0 10px 28px rgba(0,0,0,0.16)' }}>
          <button
            type="button"
            onClick={() => { onChange(null); setOpen(false) }}
            style={{ width: '100%', height: 34, padding: '0 8px', display: 'flex', alignItems: 'center', gap: 8, border: 'none', borderRadius: 6, background: !value ? '#F5F5F7' : 'transparent', cursor: 'pointer', textAlign: 'left' }}
          >
            <ClassIcon classKey={null} size={24} />
            <span style={{ fontSize: 12, fontWeight: !value ? 700 : 500 }}>未选职业</span>
          </button>
          {dragonNestClasses.map((item) => (
            <button
              key={item.key}
              type="button"
              onClick={() => { onChange(item.key); setOpen(false) }}
              style={{ width: '100%', height: 34, padding: '0 8px', display: 'flex', alignItems: 'center', gap: 8, border: 'none', borderRadius: 6, background: value === item.key ? '#FFF7D6' : 'transparent', cursor: 'pointer', textAlign: 'left' }}
            >
              <img src={item.icon} alt={item.name} style={{ width: 24, height: 24, borderRadius: 6, objectFit: 'cover', background: '#1D1D1F', flexShrink: 0 }} />
              <span style={{ width: 46, fontSize: 11, color: '#86868B' }}>{item.base}</span>
              <span style={{ fontSize: 12, fontWeight: value === item.key ? 700 : 500 }}>{item.name}</span>
            </button>
          ))}
        </div>,
        document.body
      )}
    </div>
  )
}

function NoteCell({
  value,
  onChange,
}: {
  value: string
  onChange: (value: string) => void
}) {
  const [tooltip, setTooltip] = useState<{ top: number; left: number; width: number } | null>(null)
  const showTimer = useRef<ReturnType<typeof setTimeout> | null>(null)
  const hideTimer = useRef<ReturnType<typeof setTimeout> | null>(null)
  const text = value.trim()

  const clearTimers = () => {
    if (showTimer.current) window.clearTimeout(showTimer.current)
    if (hideTimer.current) window.clearTimeout(hideTimer.current)
    showTimer.current = null
    hideTimer.current = null
  }

  const openPanel = (event: ReactMouseEvent<HTMLDivElement>, delay = 420) => {
    if (!text) return
    const rect = event.currentTarget.getBoundingClientRect()
    const width = Math.max(300, Math.min(380, rect.width + 130))
    const height = 230
    const belowTop = rect.bottom + 8
    const top = belowTop + height <= window.innerHeight - 12
      ? belowTop
      : Math.max(12, rect.top - height - 8)
    const left = Math.max(12, Math.min(rect.left, window.innerWidth - width - 12))
    clearTimers()
    showTimer.current = window.setTimeout(() => {
      setTooltip({ top, left, width })
    }, delay)
  }

  const scheduleClose = () => {
    if (showTimer.current) window.clearTimeout(showTimer.current)
    hideTimer.current = window.setTimeout(() => setTooltip(null), 700)
  }

  useEffect(() => {
    return () => clearTimers()
  }, [])

  const panel = tooltip && createPortal(
    <div
      onMouseEnter={() => {
        if (hideTimer.current) window.clearTimeout(hideTimer.current)
      }}
      onMouseLeave={scheduleClose}
      style={{
        position: 'fixed',
        top: tooltip.top,
        left: tooltip.left,
        width: tooltip.width,
        zIndex: 9999,
        padding: 10,
        borderRadius: 12,
        border: '1px solid rgba(0,0,0,0.08)',
        background: 'rgba(255,255,255,0.96)',
        color: '#1D1D1F',
        boxShadow: '0 14px 34px rgba(0,0,0,0.18)',
        backdropFilter: 'blur(16px)',
      }}
    >
      <div style={{ marginBottom: 8, display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
        <span style={{ fontSize: 12, fontWeight: 700, color: '#1D1D1F' }}>角色备注</span>
        <span style={{ fontSize: 11, color: '#86868B' }}>可直接修改</span>
      </div>
      <textarea
        value={value}
        onChange={(event) => onChange(event.target.value)}
        autoFocus
        style={{
          width: '100%',
          height: 168,
          resize: 'none',
          padding: '9px 10px',
          border: '1px solid #E5E5EA',
          borderRadius: 9,
          background: '#FAFAFC',
          color: '#1D1D1F',
          fontSize: 12,
          lineHeight: 1.55,
          outline: 'none',
          boxSizing: 'border-box',
        }}
      />
    </div>,
    document.body
  )

  const closeNow = () => {
    clearTimers()
    setTooltip(null)
  }

  return (
    <div
      onMouseEnter={(event) => openPanel(event)}
      onMouseLeave={scheduleClose}
      style={{ width: '100%', height: 32, display: 'flex', alignItems: 'center' }}
    >
      <input
        value={value}
        onChange={(event) => onChange(event.target.value)}
        onFocus={(event) => openPanel(event as unknown as ReactMouseEvent<HTMLDivElement>, 0)}
        onBlur={() => {
          if (!tooltip) closeNow()
        }}
        placeholder="备注"
        style={{ width: '100%', height: 28, padding: '0 9px', border: tooltip ? '1px solid #D4AF37' : '1px solid transparent', borderRadius: 7, background: 'rgba(255,255,255,0.62)', color: '#1D1D1F', fontSize: 12, outline: 'none', overflow: 'hidden', textOverflow: 'ellipsis' }}
      />
      {panel}
    </div>
  )
}

/* ─── Add Character Modal ─── */

function AddCharModal({ open, onClose, onAdd }: { open: boolean; onClose: () => void; onAdd: (name: string, classKey: string | null) => void }) {
  const [name, setName] = useState('')
  const [classKey, setClassKey] = useState<string | null>(null)
  const inputRef = useRef<HTMLInputElement>(null)

  useEffect(() => {
    if (open) { setName(''); setClassKey(null); setTimeout(() => inputRef.current?.focus(), 50) }
  }, [open])

  if (!open) return null

  return (
    <div style={{ position: 'fixed', inset: 0, background: 'rgba(0,0,0,0.3)', zIndex: 5000, display: 'flex', alignItems: 'center', justifyContent: 'center' }} onClick={onClose}>
      <div className="card" style={{ width: 300, padding: 20, display: 'flex', flexDirection: 'column', gap: 16 }} onClick={(e) => e.stopPropagation()}>
        <h3 style={{ margin: 0, fontSize: 16, fontWeight: 600 }}>添加角色</h3>
        <input ref={inputRef} type="text" value={name} onChange={(e) => setName(e.target.value)}
          onKeyDown={(e) => e.key === 'Enter' && name.trim() && (onAdd(name.trim(), classKey), setName(''))}
          placeholder="角色名称" style={{ height: 40, padding: '0 12px', border: '1px solid #E5E5EA', borderRadius: 8, fontSize: 14, outline: 'none' }} />
        <ClassSelect value={classKey} onChange={setClassKey} />
        <div style={{ display: 'flex', gap: 8 }}>
          <button className="btn btn-ghost" style={{ flex: 1 }} onClick={onClose}>取消</button>
          <button className="btn btn-gold" style={{ flex: 1 }} disabled={!name.trim()} onClick={() => { onAdd(name.trim(), classKey); setName('') }}>添加</button>
        </div>
      </div>
    </div>
  )
}

function ManageDungeonModal({
  open,
  dungeons,
  onClose,
  onSave,
  onDelete,
  onAdd,
  message,
}: {
  open: boolean
  dungeons: Dungeon[]
  onClose: () => void
  onSave: (dungeon: Dungeon) => void
  onDelete: (id: string) => void
  onAdd: (name: string, shortName: string, max: number, resetDay: number, resetHour: number, note: string) => void
  message: string
}) {
  const [drafts, setDrafts] = useState<Record<string, Dungeon>>({})
  const [newName, setNewName] = useState('')
  const [newShortName, setNewShortName] = useState('')
  const [newMaxClears, setNewMaxClears] = useState(1)
  const [newResetDay, setNewResetDay] = useState(6)
  const [newResetHour, setNewResetHour] = useState(9)
  const [newNote, setNewNote] = useState('')

  useEffect(() => {
    if (!open) return
    setDrafts(Object.fromEntries(dungeons.map((d) => [d.id, { ...d }])))
    setNewName('')
    setNewShortName('')
    setNewMaxClears(1)
    setNewResetDay(6)
    setNewResetHour(9)
    setNewNote('')
  }, [dungeons, open])

  if (!open) return null

  return (
    <div style={{ position: 'fixed', inset: 0, background: 'rgba(0,0,0,0.3)', zIndex: 5000, display: 'flex', alignItems: 'center', justifyContent: 'center' }} onClick={onClose}>
      <div className="card" style={{ width: 'min(680px, calc(100vw - 32px))', maxHeight: '82dvh', padding: 16, display: 'flex', flexDirection: 'column', gap: 10 }} onClick={(e) => e.stopPropagation()}>
        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
          <h3 style={{ margin: 0, fontSize: 16, fontWeight: 600 }}>管理副本</h3>
          <button className="btn btn-ghost" style={{ height: 28, padding: '0 10px' }} onClick={onClose}>关闭</button>
        </div>
        <div style={{ display: 'grid', gridTemplateColumns: 'minmax(160px, 1fr) 90px 52px 96px 88px 64px', gap: 8, alignItems: 'center', padding: 10, borderRadius: 8, background: '#FAFAFC', border: '1px solid #E5E5EA' }}>
          <input value={newName} onChange={(e) => setNewName(e.target.value)} placeholder="副本名称" style={{ height: 30, padding: '0 8px', border: '1px solid #E5E5EA', borderRadius: 6, fontSize: 12, minWidth: 0 }} />
          <input value={newShortName} onChange={(e) => setNewShortName(e.target.value)} placeholder="简称" style={{ height: 30, padding: '0 8px', border: '1px solid #E5E5EA', borderRadius: 6, fontSize: 12, minWidth: 0 }} />
          <input type="number" min={1} max={99} value={newMaxClears} onChange={(e) => setNewMaxClears(Math.max(1, parseInt(e.target.value) || 1))} style={{ height: 30, padding: '0 6px', border: '1px solid #E5E5EA', borderRadius: 6, fontSize: 12, minWidth: 0 }} />
          <select value={newResetDay} onChange={(e) => setNewResetDay(Number(e.target.value))} style={{ width: '100%', height: 30, padding: '0 6px', border: '1px solid #E5E5EA', borderRadius: 6, fontSize: 12, background: '#fff', minWidth: 0 }}>
            {weekOptions.map((option) => <option key={option.value} value={option.value}>{option.label}</option>)}
          </select>
          <select value={newResetHour} onChange={(e) => setNewResetHour(Number(e.target.value))} style={{ width: '100%', height: 30, padding: '0 6px', border: '1px solid #E5E5EA', borderRadius: 6, fontSize: 12, background: '#fff', minWidth: 0 }}>
            {Array.from({ length: 24 }, (_, hour) => <option key={hour} value={hour}>{String(hour).padStart(2, '0')}:00</option>)}
          </select>
          <button
            className="btn btn-gold"
            style={{ height: 30, padding: '0 8px', fontSize: 12 }}
            disabled={!newName.trim()}
            onClick={() => {
              onAdd(newName.trim(), newShortName.trim() || newName.trim(), newMaxClears, newResetDay, newResetHour, newNote.trim())
              setNewName('')
              setNewShortName('')
              setNewMaxClears(1)
              setNewResetDay(6)
              setNewResetHour(9)
              setNewNote('')
            }}
          >
            新增
          </button>
          <input value={newNote} onChange={(e) => setNewNote(e.target.value)} placeholder="副本备注" style={{ gridColumn: '1 / -1', height: 30, padding: '0 8px', border: '1px solid #E5E5EA', borderRadius: 6, fontSize: 12, minWidth: 0 }} />
        </div>
        {message && (
          <div style={{ height: 24, display: 'flex', alignItems: 'center', padding: '0 8px', borderRadius: 6, background: '#F0FFF4', color: '#248A3D', fontSize: 12, fontWeight: 600 }}>
            {message}
          </div>
        )}
        <div style={{ display: 'flex', flexDirection: 'column', gap: 8, overflowY: 'auto', overflowX: 'hidden', paddingRight: 2 }}>
          {dungeons.map((d) => {
            const draft = drafts[d.id] || d
            return (
              <div key={d.id} style={{ display: 'grid', gridTemplateColumns: 'minmax(160px, 1fr) 90px 52px 96px 88px 30px 30px', gap: 8, alignItems: 'center' }}>
                <input
                  value={draft.name}
                  onChange={(e) => setDrafts((prev) => ({ ...prev, [d.id]: { ...draft, name: e.target.value } }))}
                  style={{ height: 30, padding: '0 8px', border: '1px solid #E5E5EA', borderRadius: 6, fontSize: 12, minWidth: 0 }}
                />
                <input
                  value={draft.short_name}
                  onChange={(e) => setDrafts((prev) => ({ ...prev, [d.id]: { ...draft, short_name: e.target.value } }))}
                  style={{ height: 30, padding: '0 8px', border: '1px solid #E5E5EA', borderRadius: 6, fontSize: 12, minWidth: 0 }}
                />
                <input
                  type="number"
                  min={1}
                  max={99}
                  value={draft.max_clears}
                  onChange={(e) => setDrafts((prev) => ({ ...prev, [d.id]: { ...draft, max_clears: Math.max(1, parseInt(e.target.value) || 1) } }))}
                  style={{ height: 30, padding: '0 6px', border: '1px solid #E5E5EA', borderRadius: 6, fontSize: 12, minWidth: 0 }}
                />
                <select
                  value={draft.reset_day}
                  onChange={(e) => setDrafts((prev) => ({ ...prev, [d.id]: { ...draft, reset_day: Number(e.target.value) } }))}
                  style={{ width: '100%', height: 30, padding: '0 6px', border: '1px solid #E5E5EA', borderRadius: 6, fontSize: 12, background: '#fff', minWidth: 0 }}
                >
                  {weekOptions.map((option) => <option key={option.value} value={option.value}>{option.label}</option>)}
                </select>
                <select
                  value={draft.reset_hour}
                  onChange={(e) => setDrafts((prev) => ({ ...prev, [d.id]: { ...draft, reset_hour: Number(e.target.value) } }))}
                  style={{ width: '100%', height: 30, padding: '0 6px', border: '1px solid #E5E5EA', borderRadius: 6, fontSize: 12, background: '#fff', minWidth: 0 }}
                >
                  {Array.from({ length: 24 }, (_, hour) => <option key={hour} value={hour}>{String(hour).padStart(2, '0')}:00</option>)}
                </select>
                <button className="btn-icon" style={{ width: 26, height: 26 }} onClick={() => onSave(draft)} title="保存">
                  <Save size={12} />
                </button>
                <button className="btn-icon" style={{ width: 26, height: 26, color: '#D70015' }} onClick={() => onDelete(d.id)} title="删除">
                  <Trash2 size={12} />
                </button>
                <input
                  value={draft.note || ''}
                  onChange={(e) => setDrafts((prev) => ({ ...prev, [d.id]: { ...draft, note: e.target.value } }))}
                  placeholder="副本备注"
                  style={{ gridColumn: '1 / -1', height: 30, padding: '0 8px', border: '1px solid #E5E5EA', borderRadius: 6, fontSize: 12, minWidth: 0 }}
                />
              </div>
            )
          })}
        </div>
      </div>
    </div>
  )
}

/* ─── Reset Modal ─── */

function ResetModal({ open, onClose, onConfirm }: { open: boolean; onClose: () => void; onConfirm: () => void }) {
  if (!open) return null
  return (
    <div style={{ position: 'fixed', inset: 0, background: 'rgba(0,0,0,0.3)', zIndex: 5000, display: 'flex', alignItems: 'center', justifyContent: 'center' }} onClick={onClose}>
      <div className="card" style={{ width: 300, padding: 20, textAlign: 'center' }} onClick={(e) => e.stopPropagation()}>
        <h3 style={{ margin: '0 0 8px', fontSize: 15 }}>重置所有进度？</h3>
        <p style={{ margin: '0 0 16px', fontSize: 12, color: '#86868B' }}>所有角色的巢穴通关次数将清零。</p>
        <div style={{ display: 'flex', gap: 8 }}>
          <button className="btn btn-ghost" style={{ flex: 1 }} onClick={onClose}>取消</button>
          <button className="btn" style={{ flex: 1, background: '#FFF5F5', color: '#FF3B30', border: '1px solid rgba(255,59,48,0.2)' }} onClick={onConfirm}>确认重置</button>
        </div>
      </div>
    </div>
  )
}

function ConfirmDeleteCharModal({
  character,
  onClose,
  onConfirm,
}: {
  character: Character | null
  onClose: () => void
  onConfirm: () => void
}) {
  if (!character) return null
  return (
    <div style={{ position: 'fixed', inset: 0, background: 'rgba(0,0,0,0.3)', zIndex: 5000, display: 'flex', alignItems: 'center', justifyContent: 'center' }} onClick={onClose}>
      <div className="card" style={{ width: 300, padding: 20, textAlign: 'center' }} onClick={(e) => e.stopPropagation()}>
        <h3 style={{ margin: '0 0 8px', fontSize: 15 }}>删除角色？</h3>
        <p style={{ margin: '0 0 16px', fontSize: 12, color: '#86868B' }}>{character.name} 的副本记录会一起删除。</p>
        <div style={{ display: 'flex', gap: 8 }}>
          <button className="btn btn-ghost" style={{ flex: 1 }} onClick={onClose}>取消</button>
          <button className="btn" style={{ flex: 1, background: '#FFF5F5', color: '#FF3B30', border: '1px solid rgba(255,59,48,0.2)' }} onClick={onConfirm}>确认删除</button>
        </div>
      </div>
    </div>
  )
}

/* ─── Main Page ─── */

export default function Characters() {
  const [chars, setChars] = useState<Character[]>([])
  const [dungeons, setDungeons] = useState<Dungeon[]>([])
  const [records, setRecords] = useState<ClearRecord[]>([])
  const [showAddChar, setShowAddChar] = useState(false)
  const [showManageDungeon, setShowManageDungeon] = useState(false)
  const [showReset, setShowReset] = useState(false)
  const [deleteChar, setDeleteChar] = useState<Character | null>(null)
  const [notice, setNotice] = useState('')
  const [selDungeon, setSelDungeon] = useState<string | null>(null)
  const [hideNoCdChars, setHideNoCdChars] = useState(true)
  const [ddOpen, setDdOpen] = useState(false)
  const ddRef = useRef<HTMLDivElement>(null)
  const tableWrapRef = useRef<HTMLDivElement>(null)
  const [tableWidth, setTableWidth] = useState(0)
  const [dungeonPage, setDungeonPage] = useState(0)

  const load = useCallback(async () => {
    const [c, d, r] = await Promise.all([
      invoke<Character[]>('list_characters'),
      invoke<Dungeon[]>('list_dungeon_defs'),
      invoke<ClearRecord[]>('list_clear_records'),
    ])
    setChars(c)
    setDungeons(d)
    setRecords(r)
  }, [])

  useEffect(() => { load() }, [load])

  useEffect(() => {
    const targetWidth = Math.max(
      NORMAL_WINDOW_WIDTH,
      CHARACTER_COLUMN_WIDTH + CLASS_COLUMN_WIDTH + NOTE_COLUMN_WIDTH + dungeons.length * DUNGEON_COLUMN_WIDTH + CHARACTER_PAGE_CHROME_WIDTH
    )
    invoke('resize_window', { width: targetWidth, height: WINDOW_HEIGHT }).catch(() => {})
  }, [dungeons.length])

  useEffect(() => {
    function handler(e: MouseEvent) {
      if (ddRef.current && !ddRef.current.contains(e.target as Node)) setDdOpen(false)
    }
    document.addEventListener('click', handler)
    return () => document.removeEventListener('click', handler)
  }, [])

  useEffect(() => {
    const node = tableWrapRef.current
    if (!node) return
    const updateWidth = () => setTableWidth(node.clientWidth)
    updateWidth()
    const observer = new ResizeObserver(updateWidth)
    observer.observe(node)
    return () => observer.disconnect()
  }, [])

  const getClears = (charId: string, dungeonId: string) =>
    records.find((r) => r.character_id === charId && r.dungeon_id === dungeonId)?.current_clears ?? 0

  const update = async (charId: string, dungeonId: string, delta: number) => {
    try {
      const d = dungeons.find((x) => x.id === dungeonId)
      if (!d) { console.error('Dungeon not found:', dungeonId); return }
      const cur = getClears(charId, dungeonId)
      const next = Math.max(0, Math.min(d.max_clears, cur + delta))
      await invoke('update_clear_record', { character_id: charId, dungeon_id: dungeonId, current_clears: next })
      load()
    } catch (err) {
      console.error('Failed to update clear record:', err)
    }
  }

  const addChar = async (name: string, classKey: string | null) => {
    try {
      await invoke('add_character', { name, server: undefined, class_key: classKey })
      setShowAddChar(false)
      load()
    } catch (err) {
      console.error('Failed to add character:', err)
    }
  }

  const addDungeon = async (name: string, shortName: string, max: number, resetDay: number, resetHour: number, note: string) => {
    try {
      await invoke('add_dungeon_def', { name, short_name: shortName, max_clears: max, reset_day: resetDay, reset_hour: resetHour, note })
      await load()
      setNotice('副本已新增')
      setTimeout(() => setNotice(''), 1500)
    } catch (err) {
      console.error('Failed to add dungeon:', err)
    }
  }

  const updateCharClass = async (id: string, classKey: string | null) => {
    try {
      await invoke('update_character_class', { id, class_key: classKey })
      load()
    } catch (err) {
      console.error('Failed to update character class:', err)
    }
  }

  const saveDungeon = async (dungeon: Dungeon) => {
    try {
      await invoke('update_dungeon_def', {
        id: dungeon.id,
        name: dungeon.name.trim(),
        short_name: dungeon.short_name.trim() || dungeon.name.trim(),
        max_clears: dungeon.max_clears,
        reset_day: dungeon.reset_day,
        reset_hour: dungeon.reset_hour,
        note: dungeon.note || '',
      })
      await load()
      setNotice('副本已保存')
      setTimeout(() => setNotice(''), 1500)
    } catch (err) {
      console.error('Failed to update dungeon:', err)
    }
  }

  const deleteDungeon = async (id: string) => {
    try {
      await invoke('delete_dungeon_def', { id })
      if (selDungeon === id) setSelDungeon(null)
      await load()
      setNotice('副本已删除')
      setTimeout(() => setNotice(''), 1500)
    } catch (err) {
      console.error('Failed to delete dungeon:', err)
    }
  }

  const updateCharNote = async (id: string, note: string) => {
    try {
      setChars((prev) => prev.map((char) => char.id === id ? { ...char, note } : char))
      await invoke('update_character_note', { id, note })
    } catch (err) {
      console.error('Failed to update character note:', err)
      load()
    }
  }

  const delChar = async (id: string) => {
    try {
      await invoke('delete_character', { id })
      setDeleteChar(null)
      load()
    } catch (err) {
      console.error('Failed to delete character:', err)
    }
  }

  const resetAll = async () => {
    try {
      await invoke('manual_reset_all_cds')
      setShowReset(false)
      load()
    } catch (err) {
      console.error('Failed to reset CDs:', err)
    }
  }

  const sortedChars = [...chars].sort((a, b) => {
    const byClass = getClassOrder(a.class_key) - getClassOrder(b.class_key)
    if (byClass !== 0) return byClass
    return a.name.localeCompare(b.name, 'zh-Hans-CN')
  })
  const tableChars = selDungeon && hideNoCdChars
    ? sortedChars.filter((char) => getClears(char.id, selDungeon) > 0)
    : sortedChars

  const visibleDungeonCount = Math.max(
    1,
    Math.floor(((tableWidth || 340) - CHARACTER_COLUMN_WIDTH - CLASS_COLUMN_WIDTH - NOTE_COLUMN_WIDTH) / DUNGEON_COLUMN_WIDTH)
  )
  const dungeonPages = Math.max(1, Math.ceil(dungeons.length / visibleDungeonCount))
  const visibleDungeons = useMemo(() => {
    const start = Math.min(dungeonPage, dungeonPages - 1) * visibleDungeonCount
    return dungeons.slice(start, start + visibleDungeonCount)
  }, [dungeonPage, dungeonPages, dungeons, visibleDungeonCount])

  useEffect(() => {
    if (!selDungeon) return
    const index = dungeons.findIndex((d) => d.id === selDungeon)
    if (index >= 0) {
      setDungeonPage(Math.floor(index / visibleDungeonCount))
    }
  }, [dungeons, selDungeon, visibleDungeonCount])

  useEffect(() => {
    if (selDungeon) setHideNoCdChars(true)
  }, [selDungeon])

  useEffect(() => {
    setDungeonPage((page) => Math.min(page, Math.max(0, dungeonPages - 1)))
  }, [dungeonPages])

  return (
    <div style={{ flex: 1, display: 'flex', flexDirection: 'column', overflow: 'hidden' }}>
      {/* Header */}
      <div style={{ padding: '10px 12px', display: 'flex', alignItems: 'center', justifyContent: 'space-between', borderBottom: '1px solid #E5E5EA', flexShrink: 0, background: '#fff' }}>
        <div>
          <div style={{ fontSize: 15, fontWeight: 600 }}>角色 CD</div>
          <div style={{ fontSize: 11, color: '#86868B' }}>{chars.length} 角色 · {dungeons.length} 副本</div>
        </div>
        <div style={{ display: 'flex', gap: 6 }}>
          <button className="btn btn-ghost" onClick={() => setShowManageDungeon(true)}>管理</button>
          <button className="btn btn-ghost" onClick={() => setShowReset(true)}><RotateCcw size={12} /> 重置</button>
          <button className="btn btn-gold" onClick={() => setShowAddChar(true)}><Plus size={12} /> 角色</button>
        </div>
      </div>

      {/* Filter */}
      <div style={{ padding: '8px 12px', display: 'flex', alignItems: 'center', gap: 8, borderBottom: '1px solid #E5E5EA', flexShrink: 0, background: '#fff' }}>
        <Filter size={12} color="#86868B" />
        <div ref={ddRef} style={{ position: 'relative' }}>
          <button
            className="btn btn-ghost"
            style={{
              height: 28,
              padding: '0 10px',
              fontWeight: 600,
              color: selDungeon ? '#8C6B00' : '#1D1D1F',
              background: selDungeon ? '#FFF7D6' : undefined,
              borderColor: selDungeon ? '#D4AF37' : undefined,
            }}
            onClick={() => setDdOpen((v) => !v)}
          >
            {selDungeon ? dungeons.find((d) => d.id === selDungeon)?.name : '全部副本'}
            <ChevronDown size={12} />
          </button>
          {ddOpen && (
            <div className="card" style={{ position: 'absolute', top: 'calc(100% + 4px)', left: 0, minWidth: 140, zIndex: 100, padding: '4px 0', boxShadow: '0 8px 24px rgba(0,0,0,0.1)' }}>
              <button style={{ width: '100%', height: 30, padding: '0 12px', fontSize: 12, textAlign: 'left', border: 'none', color: selDungeon === null ? '#8C6B00' : '#1D1D1F', background: selDungeon === null ? '#FFF7D6' : 'transparent', cursor: 'pointer', fontWeight: selDungeon === null ? 600 : 400 }} onClick={() => { setSelDungeon(null); setDdOpen(false) }}>全部副本</button>
              {dungeons.map((d) => (
                <button key={d.id} style={{ width: '100%', height: 30, padding: '0 12px', fontSize: 12, textAlign: 'left', border: 'none', color: selDungeon === d.id ? '#8C6B00' : '#1D1D1F', background: selDungeon === d.id ? '#FFF7D6' : 'transparent', cursor: 'pointer', fontWeight: selDungeon === d.id ? 600 : 400 }} onClick={() => { setSelDungeon(d.id); setDdOpen(false) }}>
                  {d.name}
                </button>
              ))}
            </div>
          )}
        </div>
        {selDungeon && (
          <>
            <button className="btn btn-ghost" style={{ height: 28, padding: '0 10px', fontSize: 11 }} onClick={() => setSelDungeon(null)}>清除筛选</button>
            <button
              className="btn btn-ghost"
              style={{
                height: 28,
                padding: '0 10px',
                fontSize: 11,
                fontWeight: 600,
                color: hideNoCdChars ? '#8C6B00' : '#86868B',
                background: hideNoCdChars ? '#FFF7D6' : '#fff',
                borderColor: hideNoCdChars ? '#D4AF37' : '#E5E5EA',
              }}
              onClick={() => setHideNoCdChars((value) => !value)}
            >
              隐藏无CD角色
            </button>
          </>
        )}
        {dungeonPages > 1 && (
          <div style={{ marginLeft: 'auto', display: 'flex', alignItems: 'center', gap: 4 }}>
            <button
              className="btn-icon"
              style={{ width: 26, height: 26, border: '1px solid #E5E5EA', background: '#fff' }}
              disabled={dungeonPage <= 0}
              onClick={() => setDungeonPage((page) => Math.max(0, page - 1))}
              title="上一组副本"
            >
              <ChevronLeft size={13} />
            </button>
            <span style={{ width: 42, textAlign: 'center', fontSize: 11, color: '#86868B', fontWeight: 600 }}>
              {dungeonPage + 1}/{dungeonPages}
            </span>
            <button
              className="btn-icon"
              style={{ width: 26, height: 26, border: '1px solid #E5E5EA', background: '#fff' }}
              disabled={dungeonPage >= dungeonPages - 1}
              onClick={() => setDungeonPage((page) => Math.min(dungeonPages - 1, page + 1))}
              title="下一组副本"
            >
              <ChevronRight size={13} />
            </button>
          </div>
        )}
      </div>

      {/* Table */}
      <div ref={tableWrapRef} style={{ flex: 1, overflow: 'auto', overflowX: 'hidden', position: 'relative' }}>
        {chars.length === 0 ? (
          <div style={{ height: '100%', display: 'flex', flexDirection: 'column', alignItems: 'center', justifyContent: 'center', gap: 8, color: '#86868B' }}>
            <div style={{ fontSize: 14 }}>还没有角色</div>
            <button className="btn btn-gold" onClick={() => setShowAddChar(true)}><Plus size={12} /> 添加角色</button>
          </div>
        ) : (
          <table style={{ width: '100%', tableLayout: 'fixed', borderCollapse: 'collapse' }}>
            <colgroup>
              <col style={{ width: CHARACTER_COLUMN_WIDTH }} />
              <col style={{ width: CLASS_COLUMN_WIDTH }} />
              {visibleDungeons.map((d) => (
                <col key={d.id} style={{ width: DUNGEON_COLUMN_WIDTH }} />
              ))}
              <col style={{ width: NOTE_COLUMN_WIDTH }} />
            </colgroup>
            <thead>
              <tr style={{ height: 36 }}>
                <th style={{ position: 'sticky', top: 0, left: 0, zIndex: 10, width: CHARACTER_COLUMN_WIDTH, padding: '0 8px', textAlign: 'left', fontSize: 11, fontWeight: 600, color: '#86868B', background: '#fff', borderBottom: '2px solid #E5E5EA' }}>角色</th>
                <th style={{ position: 'sticky', top: 0, left: CHARACTER_COLUMN_WIDTH, zIndex: 9, width: CLASS_COLUMN_WIDTH, padding: '0 8px', textAlign: 'left', fontSize: 11, fontWeight: 600, color: '#86868B', background: '#fff', borderBottom: '2px solid #E5E5EA' }}>职业</th>
                {visibleDungeons.map((d) => {
                  const muted = !!selDungeon && selDungeon !== d.id
                  return (
                    <th key={d.id} title={d.name} style={{ position: 'sticky', top: 0, zIndex: 5, padding: '0 4px', textAlign: 'center', fontSize: 11, fontWeight: selDungeon === d.id ? 700 : 600, color: muted ? '#B8B8BE' : '#86868B', background: muted ? '#F5F5F7' : '#fff', borderBottom: selDungeon === d.id ? '2px solid #86868B' : '2px solid #E5E5EA', cursor: 'pointer', whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis' }} onClick={() => setSelDungeon(selDungeon === d.id ? null : d.id)}>
                      {d.short_name || d.name}
                    </th>
                  )
                })}
                <th style={{ position: 'sticky', top: 0, zIndex: 6, padding: '0 8px', textAlign: 'left', fontSize: 11, fontWeight: 600, color: '#86868B', background: '#fff', borderBottom: '2px solid #E5E5EA' }}>备注</th>
              </tr>
            </thead>
            <tbody>
              {tableChars.length === 0 && (
                <tr>
                  <td colSpan={visibleDungeons.length + 3} style={{ height: 96, textAlign: 'center', color: '#86868B', fontSize: 12, borderBottom: '1px solid #F0F0F2' }}>
                    当前筛选下没有 CD 角色
                  </td>
                </tr>
              )}
              {tableChars.map((char, i) => (
                <tr key={char.id} style={{ height: 44, background: i % 2 === 0 ? '#fff' : '#FAFAFC' }}>
                  <td style={{ position: 'sticky', left: 0, padding: '0 8px', background: i % 2 === 0 ? '#fff' : '#FAFAFC', borderBottom: '1px solid #F0F0F2', width: CHARACTER_COLUMN_WIDTH }}>
                    <div style={{ display: 'flex', alignItems: 'center', gap: 6 }}>
                      <ClassIcon classKey={char.class_key} size={28} />
                      <span style={{ fontSize: 12, fontWeight: 500, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap', minWidth: 0, flex: 1 }} title={char.name}>{char.name}</span>
                      <button className="btn-icon" style={{ width: 22, height: 22, marginLeft: 'auto', color: '#D70015' }} onClick={() => setDeleteChar(char)} title="删除">
                        <Trash2 size={10} />
                      </button>
                    </div>
                  </td>
                  <td style={{ position: 'sticky', left: CHARACTER_COLUMN_WIDTH, zIndex: 4, padding: '0 6px', background: i % 2 === 0 ? '#fff' : '#FAFAFC', borderBottom: '1px solid #F0F0F2', width: CLASS_COLUMN_WIDTH }}>
                    <ClassSelect value={char.class_key} onChange={(classKey) => updateCharClass(char.id, classKey)} showIcon={false} />
                  </td>
                  {visibleDungeons.map((d) => {
                    const clears = getClears(char.id, d.id)
                    const { bg, text } = cellColor(clears, d.max_clears)
                    const maxed = clears >= d.max_clears
                    const muted = !!selDungeon && selDungeon !== d.id
                    return (
                      <td key={d.id} style={{ padding: '0 4px', textAlign: 'center', borderBottom: '1px solid #F0F0F2', background: muted ? '#F5F5F7' : bg, opacity: muted ? 0.55 : 1 }}>
                        <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', gap: 2, filter: muted ? 'grayscale(1)' : undefined }}>
                          <span style={{ fontSize: 13, fontWeight: 600, color: text }}>{maxed ? '✓' : `${clears}/${d.max_clears}`}</span>
                          <div style={{ display: 'flex', gap: 2 }}>
                            <button className="btn-icon" style={{ width: 22, height: 22, border: '1px solid #E5E5EA', borderRadius: '50%' }} onClick={() => update(char.id, d.id, -1)} disabled={clears <= 0}><Minus size={10} /></button>
                            <button className="btn-icon" style={{ width: 22, height: 22, border: '1px solid #E5E5EA', borderRadius: '50%' }} onClick={() => update(char.id, d.id, 1)} disabled={maxed}><Plus size={10} /></button>
                          </div>
                        </div>
                      </td>
                    )
                  })}
                  <td style={{ padding: '0 8px', borderBottom: '1px solid #F0F0F2', background: i % 2 === 0 ? '#fff' : '#FAFAFC' }}>
                    <NoteCell value={char.note || ''} onChange={(note) => updateCharNote(char.id, note)} />
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </div>

      <AddCharModal open={showAddChar} onClose={() => setShowAddChar(false)} onAdd={addChar} />
      <ManageDungeonModal
        open={showManageDungeon}
        dungeons={dungeons}
        onClose={() => setShowManageDungeon(false)}
        onSave={saveDungeon}
        onDelete={deleteDungeon}
        onAdd={addDungeon}
        message={notice}
      />
      <ResetModal open={showReset} onClose={() => setShowReset(false)} onConfirm={resetAll} />
      <ConfirmDeleteCharModal character={deleteChar} onClose={() => setDeleteChar(null)} onConfirm={() => deleteChar && delChar(deleteChar.id)} />
    </div>
  )
}
