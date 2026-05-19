import { create } from "zustand";
import { immer } from "zustand/middleware/immer";

export type SlotStatus = "idle" | "running" | "paused" | "warning" | "triggered";

export interface TimerSlotState {
  id: string;
  name: string;
  elapsedMs: number;
  remainingMs: number;
  targetMs: number;
  status: SlotStatus;
  loopCount: number;
  warnFired: boolean;
}

export interface TimerSlotDef {
  id: string;
  name: string;
  first: number;
  loop_interval: number;
  warn_seconds: number;
  warn_text?: string;
  hotkey?: string;
  bar_color?: string;
  text_color?: string;
}

interface TimerState {
  slots: TimerSlotState[];
  globalStatus: "idle" | "running" | "paused";
  currentPresetId: string | null;
  currentSlotDefs: TimerSlotDef[];
  compactMode: boolean;
  simpleMode: boolean;
  dockedMode: boolean;
  dockPreviewMode: boolean;

  setSlots: (slots: TimerSlotState[]) => void;
  setSlotState: (index: number, update: Partial<TimerSlotState>) => void;
  setGlobalStatus: (status: TimerState["globalStatus"]) => void;
  setCurrentPresetId: (id: string | null) => void;
  setCurrentSlotDefs: (defs: TimerSlotDef[]) => void;
  setCompactMode: (compact: boolean) => void;
  setSimpleMode: (simple: boolean) => void;
  setDockedMode: (docked: boolean) => void;
  setDockPreviewMode: (preview: boolean) => void;
}

export const useTimerStore = create<TimerState>()(
  immer((set) => ({
    slots: [],
    globalStatus: "idle",
    currentPresetId: null,
    currentSlotDefs: [],
    compactMode: false,
    simpleMode: false,
    dockedMode: false,
    dockPreviewMode: false,

    setSlots: (slots) =>
      set((state) => {
        state.slots = slots;
      }),

    setSlotState: (index, update) =>
      set((state) => {
        while (state.slots.length <= index) {
          state.slots.push({
            id: "",
            name: "",
            elapsedMs: 0,
            remainingMs: 0,
            targetMs: 0,
            status: "idle",
            loopCount: 0,
            warnFired: false,
          });
        }
        Object.assign(state.slots[index], update);
      }),

    setGlobalStatus: (status) =>
      set((state) => {
        state.globalStatus = status;
      }),

    setCurrentPresetId: (id) =>
      set((state) => {
        state.currentPresetId = id;
      }),

    setCurrentSlotDefs: (defs) =>
      set((state) => {
        state.currentSlotDefs = defs;
      }),

    setCompactMode: (compact) =>
      set((state) => {
        state.compactMode = compact;
      }),

    setSimpleMode: (simple) =>
      set((state) => {
        state.simpleMode = simple;
      }),

    setDockedMode: (docked) =>
      set((state) => {
        state.dockedMode = docked;
      }),

    setDockPreviewMode: (preview) =>
      set((state) => {
        state.dockPreviewMode = preview;
      }),
  }))
);
