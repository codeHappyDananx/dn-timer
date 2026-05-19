import { useEffect } from "react";
import { Routes, Route } from "react-router-dom";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import Layout from "./components/Layout";
import Dashboard from "./pages/Dashboard";
import Characters from "./pages/Characters";
import Presets from "./pages/Presets";
import SettingsPage from "./pages/SettingsPage";
import { useTimerStore } from "./stores/timerStore";

interface TimerSnapshot {
  slots: {
    id: string;
    name: string;
    elapsed_ms: number;
    remaining_ms: number;
    target_ms: number;
    status: string;
    loop_count: number;
  }[];
  global_status: "idle" | "running" | "paused";
}

function App() {
  const { setSlots, setSlotState, setGlobalStatus, setSimpleMode, setDockedMode, setDockPreviewMode } = useTimerStore();

  const applySnapshot = (payload: TimerSnapshot) => {
    if (payload.slots) {
      const newSlots = payload.slots.map((slot) => ({
        id: slot.id,
        name: slot.name,
        elapsedMs: slot.elapsed_ms,
        remainingMs: slot.remaining_ms,
        targetMs: slot.target_ms,
        status: slot.status as any,
        loopCount: slot.loop_count,
        warnFired: false,
      }));
      setSlots(newSlots);
    }
    if (payload.global_status) {
      setGlobalStatus(payload.global_status);
    }
  };

  useEffect(() => {
    let cancelled = false;

    const restoreWindow = async () => {
      let enabled = false;
      try {
        enabled = await invoke<boolean>("get_simple_mode");
        if (enabled && !cancelled) {
          setSimpleMode(true);
        }
      } catch {
      } finally {
        if (!cancelled) {
          requestAnimationFrame(() => {
            invoke("frontend_ready", { simple_mode: enabled }).catch(() => {});
          });
          invoke<TimerSnapshot>("get_timer_snapshot")
            .then((snapshot) => {
              if (!cancelled && snapshot.slots.length > 0) {
                applySnapshot(snapshot);
              }
            })
            .catch(() => {});
          invoke<boolean>("is_docked_window")
            .then((docked) => {
              if (!cancelled) setDockedMode(docked);
            })
            .catch(() => {});
        }
      }
    };

    restoreWindow();

    return () => {
      cancelled = true;
    };
  }, [setDockedMode, setSimpleMode]);

  useEffect(() => {
    const unlistenTick = listen("timer:tick", (event: any) => {
      applySnapshot(event.payload);
    });

    const unlistenWarn = listen("timer:warn", (event: any) => {
      const { slot_index } = event.payload;
      setSlotState(slot_index, { status: "warning" });
    });

    const unlistenTrigger = listen("timer:trigger", (event: any) => {
      const { slot_index, loop_count } = event.payload;
      setSlotState(slot_index, { status: "triggered", loopCount: loop_count });
    });

    const unlistenHotkey = listen("hotkey:triggered", (event: any) => {
      const hotkey = event.payload as string;
      if (hotkey) {
        invoke("trigger_hotkey", { hotkey });
      }
    });

    const unlistenDocked = listen("window:docked", (event: any) => {
      setDockedMode(Boolean(event.payload));
      if (event.payload) setDockPreviewMode(false);
    });

    const unlistenDockPreview = listen("window:dock-preview", (event: any) => {
      setDockPreviewMode(Boolean(event.payload));
    });

    return () => {
      unlistenTick.then((f) => f());
      unlistenWarn.then((f) => f());
      unlistenTrigger.then((f) => f());
      unlistenHotkey.then((f) => f());
      unlistenDocked.then((f) => f());
      unlistenDockPreview.then((f) => f());
    };
  }, [setSlots, setSlotState, setGlobalStatus, setDockedMode, setDockPreviewMode]);

  return (
    <Layout>
      <Routes>
        <Route path="/" element={<Dashboard />} />
        <Route path="/characters" element={<Characters />} />
        <Route path="/presets" element={<Presets />} />
        <Route path="/settings" element={<SettingsPage />} />
      </Routes>
    </Layout>
  );
}

export default App;
