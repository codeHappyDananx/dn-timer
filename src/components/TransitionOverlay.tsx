import { useEffect, useState } from 'react'

interface TransitionOverlayProps {
  signal: string
}

export default function TransitionOverlay({ signal }: TransitionOverlayProps) {
  const [visible, setVisible] = useState(false)

  useEffect(() => {
    setVisible(true)
    const timer = window.setTimeout(() => setVisible(false), 170)
    return () => window.clearTimeout(timer)
  }, [signal])

  return (
    <div
      style={{
        pointerEvents: 'none',
        position: 'fixed',
        inset: 0,
        zIndex: 9000,
        opacity: visible ? 1 : 0,
        transition: 'opacity 120ms ease-out',
        background: 'rgba(245,245,247,0.72)',
        backdropFilter: 'blur(8px)',
        WebkitBackdropFilter: 'blur(8px)',
      }}
    />
  )
}
