import type { ReactNode } from 'react'
import { useLocation, useNavigate } from 'react-router-dom'
import { Timer, Users } from 'lucide-react'
import { useTimerStore } from '../stores/timerStore'
import Navbar from './Navbar'
import SimpleModeShell from './SimpleModeShell'
import TransitionOverlay from './TransitionOverlay'
import DockedBubble from './DockedBubble'

interface LayoutProps {
  children: ReactNode
}

function BottomTabBar() {
  const location = useLocation()
  const navigate = useNavigate()

  const tabs = [
    { path: '/', label: '计时器', Icon: Timer },
    { path: '/characters', label: '角色CD', Icon: Users },
  ]

  return (
    <nav
      className="shrink-0"
      style={{
        height: 52,
        display: 'flex',
        alignItems: 'stretch',
        background: 'rgba(255,255,255,0.95)',
        borderTop: '1px solid #E5E5EA',
      }}
    >
      {tabs.map((tab) => {
        const isActive = location.pathname === tab.path
        return (
          <button
            key={tab.path}
            onClick={() => navigate(tab.path)}
            style={{
              flex: 1,
              display: 'flex',
              flexDirection: 'column',
              alignItems: 'center',
              justifyContent: 'center',
              gap: 2,
              background: 'transparent',
              border: 'none',
              borderTop: isActive ? '2px solid #D4AF37' : '2px solid transparent',
              color: isActive ? '#D4AF37' : '#86868B',
              cursor: 'pointer',
              padding: '4px 0 2px',
            }}
          >
            <tab.Icon size={18} strokeWidth={isActive ? 2.5 : 2} />
            <span style={{ fontSize: 11, fontWeight: isActive ? 600 : 400 }}>
              {tab.label}
            </span>
          </button>
        )
      })}
    </nav>
  )
}

export default function Layout({ children }: LayoutProps) {
  const simpleMode = useTimerStore((s) => s.simpleMode)
  const dockedMode = useTimerStore((s) => s.dockedMode)
  const dockPreviewMode = useTimerStore((s) => s.dockPreviewMode)
  const location = useLocation()

  if (dockedMode || dockPreviewMode) {
    return <DockedBubble preview={dockPreviewMode && !dockedMode} />
  }

  if (simpleMode) {
    return (
      <>
        <SimpleModeShell />
        <TransitionOverlay signal="simple" />
      </>
    )
  }

  return (
    <div
      style={{
        height: '100dvh',
        display: 'flex',
        flexDirection: 'column',
        background: '#F5F5F7',
        overflow: 'hidden',
      }}
    >
      <Navbar />
      <main
        style={{
          flex: 1,
          display: 'flex',
          flexDirection: 'column',
          overflow: 'hidden',
          minHeight: 0,
        }}
      >
        {children}
      </main>
      <BottomTabBar />
      <TransitionOverlay signal={location.pathname} />
    </div>
  )
}
