import { useEffect, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { Power } from 'lucide-react'

export default function SettingsPage() {
  const [startupEnabled, setStartupEnabled] = useState(false)
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    invoke<boolean>('is_startup_enabled')
      .then(setStartupEnabled)
      .catch(() => setStartupEnabled(false))
      .finally(() => setLoading(false))
  }, [])

  const toggleStartup = async () => {
    const next = !startupEnabled
    setStartupEnabled(next)
    try {
      await invoke('set_startup', { enable: next })
    } catch {
      setStartupEnabled(!next)
    }
  }

  return (
    <div style={{ flex: 1, display: 'flex', flexDirection: 'column', padding: 12, gap: 12, overflow: 'auto' }}>
      <h2 style={{ margin: 0, fontSize: 16, fontWeight: 600 }}>设置</h2>

      <div className="card" style={{ padding: 12, display: 'flex', alignItems: 'center', justifyContent: 'space-between', gap: 12 }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: 10, minWidth: 0 }}>
          <div style={{ width: 32, height: 32, borderRadius: 8, background: '#F5F5F7', color: '#D4AF37', display: 'flex', alignItems: 'center', justifyContent: 'center', flexShrink: 0 }}>
            <Power size={16} />
          </div>
          <div style={{ minWidth: 0 }}>
            <div style={{ fontSize: 13, fontWeight: 700, color: '#1D1D1F' }}>开机启动</div>
            <div style={{ marginTop: 4, fontSize: 11, color: '#86868B', lineHeight: 1.5 }}>
              登录 Windows 后自动启动 DN Timer。
            </div>
          </div>
        </div>

        <button
          onClick={toggleStartup}
          disabled={loading}
          aria-pressed={startupEnabled}
          style={{
            width: 48,
            height: 28,
            padding: 2,
            border: 'none',
            borderRadius: 14,
            background: startupEnabled ? '#D4AF37' : '#D2D2D7',
            cursor: loading ? 'default' : 'pointer',
            transition: 'background 0.15s ease',
            flexShrink: 0,
          }}
        >
          <span
            style={{
              display: 'block',
              width: 24,
              height: 24,
              borderRadius: '50%',
              background: '#fff',
              transform: startupEnabled ? 'translateX(20px)' : 'translateX(0)',
              transition: 'transform 0.15s ease',
              boxShadow: '0 1px 3px rgba(0,0,0,0.18)',
            }}
          />
        </button>
      </div>
    </div>
  )
}
