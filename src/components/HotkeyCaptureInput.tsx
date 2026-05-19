import { useState, useEffect, useRef, useCallback } from 'react'

interface HotkeyCaptureInputProps {
  value: string
  onChange: (value: string) => void
  placeholder?: string
}

const SPECIAL_KEY_MAP: Record<string, string> = {
  Space: 'Space',
  Enter: 'Enter',
  Escape: 'Esc',
  ArrowUp: 'Up',
  ArrowDown: 'Down',
  ArrowLeft: 'Left',
  ArrowRight: 'Right',
  Backspace: 'Backspace',
  Tab: 'Tab',
  Insert: 'Insert',
  Delete: 'Delete',
  Home: 'Home',
  End: 'End',
  PageUp: 'PageUp',
  PageDown: 'PageDown',
}

export default function HotkeyCaptureInput({ value, onChange, placeholder }: HotkeyCaptureInputProps) {
  const [capturing, setCapturing] = useState(false)
  const containerRef = useRef<HTMLDivElement>(null)

  const handleKeyDown = useCallback(
    (e: KeyboardEvent) => {
      if (!capturing) return
      e.preventDefault()
      e.stopPropagation()

      if (e.key === 'Escape') {
        setCapturing(false)
        return
      }
      if (e.key === 'Backspace' || e.key === 'Delete') {
        onChange('')
        setCapturing(false)
        return
      }
      if (e.key === 'Enter') {
        setCapturing(false)
        return
      }

      const modifiers: string[] = []
      if (e.ctrlKey) modifiers.push('Ctrl')
      if (e.altKey) modifiers.push('Alt')
      if (e.shiftKey) modifiers.push('Shift')

      let key = ''
      if (e.code.startsWith('Key')) {
        key = e.code.replace('Key', '')
      } else if (e.code.startsWith('Digit')) {
        key = e.code.replace('Digit', '')
      } else if (e.code.startsWith('F') && e.code.length <= 3) {
        key = e.code
      } else if (e.code.startsWith('Numpad')) {
        key = e.code
      } else {
        key = SPECIAL_KEY_MAP[e.code] || ''
      }

      if (!key) return

      const hotkey = modifiers.length > 0 ? `${modifiers.join('+')}+${key}` : key
      onChange(hotkey)
      setCapturing(false)
    },
    [capturing, onChange]
  )

  const handleMouseDown = useCallback(
    (e: MouseEvent) => {
      if (!capturing) return
      e.preventDefault()
      e.stopPropagation()

      let key = ''
      if (e.button === 1) key = 'Middle'
      else if (e.button === 3) key = 'XButton1'
      else if (e.button === 4) key = 'XButton2'
      else return

      const modifiers: string[] = []
      if (e.ctrlKey) modifiers.push('Ctrl')
      if (e.altKey) modifiers.push('Alt')
      if (e.shiftKey) modifiers.push('Shift')

      const hotkey = modifiers.length > 0 ? `${modifiers.join('+')}+${key}` : key
      onChange(hotkey)
      setCapturing(false)
    },
    [capturing, onChange]
  )

  useEffect(() => {
    if (!capturing) return
    window.addEventListener('keydown', handleKeyDown, true)
    window.addEventListener('mousedown', handleMouseDown, true)
    return () => {
      window.removeEventListener('keydown', handleKeyDown, true)
      window.removeEventListener('mousedown', handleMouseDown, true)
    }
  }, [capturing, handleKeyDown, handleMouseDown])

  return (
    <div
      ref={containerRef}
      onClick={() => setCapturing(true)}
      style={{
        width: 100,
        height: 28,
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        border: capturing ? '2px solid #D4AF37' : '1px solid #E5E5EA',
        borderRadius: 6,
        fontSize: 12,
        textAlign: 'center',
        cursor: 'pointer',
        background: capturing ? '#FFFDF5' : value ? '#fff' : '#FAFAFC',
        color: value ? '#1D1D1F' : '#C7C7CC',
        userSelect: 'none',
        transition: 'all 0.15s',
      }}
    >
      {capturing ? '按下按键...' : value || placeholder || '点击设置'}
    </div>
  )
}
