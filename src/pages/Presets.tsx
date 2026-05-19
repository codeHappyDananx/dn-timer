import { useEffect, useMemo, useState } from 'react'
import type { CSSProperties } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { Copy, Edit3, Plus, Trash2, X } from 'lucide-react'
import HotkeyCaptureInput from '../components/HotkeyCaptureInput'

type PresetKind = 'Single' | 'Multi'

interface SlotDef {
  id: string
  name: string
  first: number
  loop_interval: number
  warn_seconds: number
  warn_text?: string
  hotkey?: string
  bar_color?: string
  text_color?: string
}

interface SingleType {
  type: 'Single'
  first: number
  loop_interval: number
  followups: number[]
  warn_seconds: number
  warn_text?: string
  hotkey?: string
  bar_color?: string
  text_color?: string
}

interface MultiType {
  type: 'Multi'
  slots: SlotDef[]
  sequential: boolean
}

type PresetType = SingleType | MultiType

interface Preset {
  id: string
  name: string
  desc?: string
  preset_type: PresetType
  is_builtin: boolean
}

interface PresetForm {
  id?: string
  name: string
  desc: string
  kind: PresetKind
  single: SlotDef
  slots: SlotDef[]
  sequential: boolean
}

const emptySlot = (name = '计时槽位'): SlotDef => ({
  id: newId(),
  name,
  first: 45,
  loop_interval: 45,
  warn_seconds: 10,
  warn_text: '',
  hotkey: '',
  bar_color: '',
  text_color: '',
})

function newId() {
  return globalThis.crypto?.randomUUID?.() || `${Date.now()}-${Math.random()}`
}

function getSlots(preset: Preset) {
  if (preset.preset_type.type === 'Multi') return preset.preset_type.slots
  return [{
    id: preset.id,
    name: preset.name,
    first: preset.preset_type.first,
    loop_interval: preset.preset_type.loop_interval,
    warn_seconds: preset.preset_type.warn_seconds,
    warn_text: preset.preset_type.warn_text,
    hotkey: preset.preset_type.hotkey,
    bar_color: preset.preset_type.bar_color,
    text_color: preset.preset_type.text_color,
  }]
}

function toForm(preset?: Preset): PresetForm {
  if (!preset) {
    return {
      name: '',
      desc: '',
      kind: 'Single',
      single: emptySlot('计时器'),
      slots: [emptySlot('槽位1')],
      sequential: false,
    }
  }

  if (preset.preset_type.type === 'Single') {
    return {
      id: preset.id,
      name: preset.name,
      desc: preset.desc || '',
      kind: 'Single',
      single: {
        id: preset.id,
        name: preset.name,
        first: preset.preset_type.first,
        loop_interval: preset.preset_type.loop_interval,
        warn_seconds: preset.preset_type.warn_seconds,
        warn_text: preset.preset_type.warn_text || '',
        hotkey: preset.preset_type.hotkey || '',
        bar_color: preset.preset_type.bar_color || '',
        text_color: preset.preset_type.text_color || '',
      },
      slots: [emptySlot('槽位1')],
      sequential: false,
    }
  }

  return {
    id: preset.id,
    name: preset.name,
    desc: preset.desc || '',
    kind: 'Multi',
    single: emptySlot('计时器'),
    slots: preset.preset_type.slots.map((slot) => ({ ...slot })),
    sequential: preset.preset_type.sequential,
  }
}

function cloneForm(preset: Preset): PresetForm {
  const form = toForm(preset)
  return {
    ...form,
    id: undefined,
    name: `${preset.name} 副本`,
    single: { ...form.single, id: newId() },
    slots: form.slots.map((slot) => ({ ...slot, id: newId() })),
  }
}

function requestFromForm(form: PresetForm) {
  const clean = (value?: string) => {
    const text = value?.trim()
    return text ? text : undefined
  }

  const preset_type: PresetType = form.kind === 'Single'
    ? {
        type: 'Single',
        first: Number(form.single.first) || 0,
        loop_interval: Number(form.single.loop_interval) || 0,
        followups: [],
        warn_seconds: Number(form.single.warn_seconds) || 0,
        warn_text: clean(form.single.warn_text),
        hotkey: clean(form.single.hotkey),
        bar_color: clean(form.single.bar_color),
        text_color: clean(form.single.text_color),
      }
    : {
        type: 'Multi',
        sequential: form.sequential,
        slots: form.slots.map((slot, index) => ({
          ...slot,
          id: slot.id || newId(),
          name: slot.name.trim() || `槽位${index + 1}`,
          first: Number(slot.first) || 0,
          loop_interval: Number(slot.loop_interval) || 0,
          warn_seconds: Number(slot.warn_seconds) || 0,
          warn_text: clean(slot.warn_text),
          hotkey: clean(slot.hotkey),
          bar_color: clean(slot.bar_color),
          text_color: clean(slot.text_color),
        })),
      }

  return {
    name: form.name.trim(),
    desc: clean(form.desc),
    preset_type,
  }
}

const inputStyle: CSSProperties = {
  height: 34,
  padding: '0 10px',
  border: '1px solid #E5E5EA',
  borderRadius: 8,
  fontSize: 12,
  outline: 'none',
  background: '#fff',
}

const labelStyle: CSSProperties = {
  display: 'flex',
  flexDirection: 'column',
  gap: 5,
  fontSize: 11,
  color: '#86868B',
  fontWeight: 600,
}

function SlotEditor({
  slot,
  index,
  onChange,
  onRemove,
  canRemove,
  hideName,
}: {
  slot: SlotDef
  index: number
  onChange: (slot: SlotDef) => void
  onRemove?: () => void
  canRemove?: boolean
  hideName?: boolean
}) {
  const patch = (data: Partial<SlotDef>) => onChange({ ...slot, ...data })

  return (
    <div className="card" style={{ padding: 10, display: 'flex', flexDirection: 'column', gap: 8 }}>
      <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
        <span style={{ fontSize: 11, fontWeight: 700, color: '#86868B' }}>S{index + 1}</span>
        {!hideName && (
          <input
            value={slot.name}
            onChange={(e) => patch({ name: e.target.value })}
            style={{ ...inputStyle, flex: 1 }}
            placeholder="槽位名称"
          />
        )}
        {onRemove && (
          <button className="btn-icon" onClick={onRemove} disabled={!canRemove} title="删除槽位">
            <Trash2 size={14} />
          </button>
        )}
      </div>

      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(3, minmax(0, 1fr))', gap: 8 }}>
        <label style={labelStyle}>
          首次秒数
          <input type="number" min={0} value={slot.first} onChange={(e) => patch({ first: Number(e.target.value) })} style={inputStyle} />
        </label>
        <label style={labelStyle}>
          循环秒数
          <input type="number" min={0} value={slot.loop_interval} onChange={(e) => patch({ loop_interval: Number(e.target.value) })} style={inputStyle} />
        </label>
        <label style={labelStyle}>
          提前提醒
          <input type="number" min={0} value={slot.warn_seconds} onChange={(e) => patch({ warn_seconds: Number(e.target.value) })} style={inputStyle} />
        </label>
      </div>

      <label style={labelStyle}>
        提醒文本
        <input value={slot.warn_text || ''} onChange={(e) => patch({ warn_text: e.target.value })} style={inputStyle} placeholder="准备躲避" />
      </label>

      <div style={{ display: 'grid', gridTemplateColumns: '1fr 80px 80px', gap: 8, alignItems: 'end' }}>
        <label style={labelStyle}>
          热键
          <HotkeyCaptureInput value={slot.hotkey || ''} onChange={(value) => patch({ hotkey: value })} placeholder="热键" />
        </label>
        <label style={labelStyle}>
          进度条
          <input type="color" value={slot.bar_color || '#34C759'} onChange={(e) => patch({ bar_color: e.target.value })} style={{ ...inputStyle, padding: 3 }} />
        </label>
        <label style={labelStyle}>
          文字
          <input type="color" value={slot.text_color || '#1D1D1F'} onChange={(e) => patch({ text_color: e.target.value })} style={{ ...inputStyle, padding: 3 }} />
        </label>
      </div>
    </div>
  )
}

export default function Presets() {
  const [presets, setPresets] = useState<Preset[]>([])
  const [form, setForm] = useState<PresetForm | null>(null)
  const [saving, setSaving] = useState(false)

  const load = async () => {
    const list = await invoke<Preset[]>('list_presets')
    setPresets(list)
  }

  useEffect(() => {
    load().catch(() => {})
  }, [])

  const summary = useMemo(() => ({
    total: presets.length,
    builtin: presets.filter((p) => p.is_builtin).length,
  }), [presets])

  const save = async () => {
    if (!form || !form.name.trim() || (form.kind === 'Multi' && form.slots.length === 0)) return
    setSaving(true)
    try {
      const data = requestFromForm(form)
      if (form.id) {
        await invoke('update_preset', { id: form.id, data })
      } else {
        await invoke('create_preset', { data })
      }
      setForm(null)
      await load()
    } finally {
      setSaving(false)
    }
  }

  const remove = async (preset: Preset) => {
    if (!confirm(`删除模板「${preset.name}」？`)) return
    await invoke('delete_preset', { id: preset.id })
    await load()
  }

  return (
    <div style={{ flex: 1, display: 'flex', flexDirection: 'column', padding: 12, gap: 10, overflow: 'hidden' }}>
      <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', gap: 8 }}>
        <div>
          <h2 style={{ margin: 0, fontSize: 16, fontWeight: 700 }}>模板管理</h2>
          <div style={{ marginTop: 3, fontSize: 11, color: '#86868B' }}>
            共 {summary.total} 个，内置 {summary.builtin} 个
          </div>
        </div>
        <button className="btn btn-gold" onClick={() => setForm(toForm())}>
          <Plus size={14} />
          新增
        </button>
      </div>

      <div style={{ flex: 1, minHeight: 0, overflow: 'auto', display: 'flex', flexDirection: 'column', gap: 8 }}>
        {presets.map((preset) => {
          const slots = getSlots(preset)
          const hotkeys = slots.map((slot) => slot.hotkey).filter(Boolean)
          return (
            <div key={preset.id} className="card" style={{ padding: 10, display: 'flex', alignItems: 'center', gap: 10 }}>
              <div style={{ minWidth: 0, flex: 1 }}>
                <div style={{ display: 'flex', alignItems: 'center', gap: 6, minWidth: 0 }}>
                  <span style={{ fontSize: 13, fontWeight: 700, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
                    {preset.name}
                  </span>
                  <span style={{ fontSize: 10, color: preset.is_builtin ? '#D4AF37' : '#86868B', border: '1px solid #E5E5EA', borderRadius: 4, padding: '1px 5px', flexShrink: 0 }}>
                    {preset.is_builtin ? '内置' : '自定义'}
                  </span>
                  <span style={{ fontSize: 10, color: '#86868B', flexShrink: 0 }}>
                    {preset.preset_type.type === 'Single' ? '单槽' : '多槽'} · {slots.length} 槽
                  </span>
                </div>
                <div style={{ marginTop: 4, fontSize: 11, color: '#86868B', overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
                  {preset.desc || slots.map((s) => s.name).join(' / ') || '无描述'}
                  {hotkeys.length > 0 ? ` · 热键 ${hotkeys.join(' / ')}` : ''}
                </div>
              </div>
              <div style={{ display: 'flex', gap: 2, flexShrink: 0 }}>
                <button className="btn-icon" onClick={() => setForm(toForm(preset))} title="编辑">
                  <Edit3 size={14} />
                </button>
                <button className="btn-icon" onClick={() => setForm(cloneForm(preset))} title="复制">
                  <Copy size={14} />
                </button>
                <button className="btn-icon" onClick={() => remove(preset)} title="删除">
                  <Trash2 size={14} />
                </button>
              </div>
            </div>
          )
        })}
      </div>

      {form && (
        <div style={{ position: 'fixed', inset: 0, background: 'rgba(0,0,0,0.24)', zIndex: 200, display: 'flex', alignItems: 'flex-end' }}>
          <div style={{ width: '100%', maxHeight: '88dvh', background: '#F5F5F7', borderTopLeftRadius: 12, borderTopRightRadius: 12, padding: 12, display: 'flex', flexDirection: 'column', gap: 10 }}>
            <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
              <h3 style={{ margin: 0, fontSize: 15, fontWeight: 700 }}>{form.id ? '编辑模板' : '新增模板'}</h3>
              <button className="btn-icon" onClick={() => setForm(null)}>
                <X size={16} />
              </button>
            </div>

            <div style={{ overflow: 'auto', display: 'flex', flexDirection: 'column', gap: 10, paddingBottom: 2 }}>
              <label style={labelStyle}>
                模板名称
                <input value={form.name} onChange={(e) => setForm({ ...form, name: e.target.value })} style={inputStyle} placeholder="模板名称" />
              </label>
              <label style={labelStyle}>
                描述
                <input value={form.desc} onChange={(e) => setForm({ ...form, desc: e.target.value })} style={inputStyle} placeholder="可选" />
              </label>

              <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 8 }}>
                {(['Single', 'Multi'] as PresetKind[]).map((kind) => (
                  <button
                    key={kind}
                    className={`btn ${form.kind === kind ? 'btn-gold' : 'btn-ghost'}`}
                    onClick={() => setForm({ ...form, kind })}
                  >
                    {kind === 'Single' ? '单槽模板' : '多槽模板'}
                  </button>
                ))}
              </div>

              {form.kind === 'Single' ? (
                <SlotEditor
                  slot={form.single}
                  index={0}
                  hideName
                  onChange={(single) => setForm({ ...form, single })}
                />
              ) : (
                <>
                  <label style={{ display: 'flex', alignItems: 'center', gap: 8, fontSize: 12, color: '#1D1D1F' }}>
                    <input type="checkbox" checked={form.sequential} onChange={(e) => setForm({ ...form, sequential: e.target.checked })} />
                    顺序触发
                  </label>
                  {form.slots.map((slot, index) => (
                    <SlotEditor
                      key={slot.id}
                      slot={slot}
                      index={index}
                      canRemove={form.slots.length > 1}
                      onChange={(next) => {
                        const slots = [...form.slots]
                        slots[index] = next
                        setForm({ ...form, slots })
                      }}
                      onRemove={() => setForm({ ...form, slots: form.slots.filter((_, i) => i !== index) })}
                    />
                  ))}
                  <button className="btn btn-ghost" onClick={() => setForm({ ...form, slots: [...form.slots, emptySlot(`槽位${form.slots.length + 1}`)] })}>
                    <Plus size={14} />
                    增加槽位
                  </button>
                </>
              )}
            </div>

            <button className="btn btn-gold" onClick={save} disabled={saving || !form.name.trim()}>
              {saving ? '保存中' : '保存'}
            </button>
          </div>
        </div>
      )}
    </div>
  )
}
