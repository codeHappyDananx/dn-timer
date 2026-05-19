import { useNavigate } from 'react-router-dom'
import { invoke } from '@tauri-apps/api/core'
import { Settings, Minus, X, Minimize2, Maximize2 } from 'lucide-react'
import { useTimerStore } from '../stores/timerStore'
import AlwaysOnTopButton from './AlwaysOnTopButton'

export default function Navbar() {
  const navigate = useNavigate()
  const { simpleMode, setSimpleMode } = useTimerStore()

  const handleMouseDown = (e: React.MouseEvent) => {
    // Only left-click, and not on buttons
    if (e.button !== 0) return
    const target = e.target as HTMLElement
    if (target.closest('button')) return
    void invoke('begin_window_drag', { dock_on_release: true })
  }

  const toggleSimpleMode = async () => {
    const newMode = !simpleMode
    setSimpleMode(newMode)
    await invoke('set_simple_mode', { enabled: newMode })
    if (newMode) {
      await invoke('enter_simple_mode')
    } else {
      await invoke('exit_simple_mode')
    }
  }

  return (
    <nav
      onMouseDown={handleMouseDown}
      style={{
        height: 44,
        padding: '0 12px',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'space-between',
        flexShrink: 0,
        background: 'rgba(245,245,247,0.98)',
        borderBottom: '1px solid #E5E5EA',
        cursor: 'default',
        userSelect: 'none',
      }}
    >
      <span
        style={{
          fontSize: 13,
          fontWeight: 600,
          letterSpacing: '0.15em',
          color: '#1D1D1F',
        }}
      >
        DN TIMER
      </span>

      <div style={{ display: 'flex', gap: 4 }}>
        <AlwaysOnTopButton />
        <button
          className="btn-icon"
          onClick={toggleSimpleMode}
          title={simpleMode ? '展开' : '极简模式'}
        >
          {simpleMode ? <Maximize2 size={14} /> : <Minimize2 size={14} />}
        </button>
        <button
          className="btn-icon"
          onClick={() => navigate('/settings')}
        >
          <Settings size={14} />
        </button>
        <button
          className="btn-icon"
          onClick={() => invoke('minimize_window')}
        >
          <Minus size={14} />
        </button>
        <button
          className="btn-icon"
          onClick={() => invoke('close_window')}
        >
          <X size={14} />
        </button>
      </div>
    </nav>
  )
}
