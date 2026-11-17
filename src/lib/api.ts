import { invoke } from "@tauri-apps/api/core";

export type StopwatchStatus = "idle" | "running" | "paused";

export interface StopwatchDto {
  id: number;
  name: string;
  accumulated_ms: number;
  started_at_ms: number | null;
  status: StopwatchStatus;
  position: number;
}

export async function listStopwatches(): Promise<StopwatchDto[]> {
  return invoke<StopwatchDto[]>("list_stopwatches");
}

export async function createStopwatch(name?: string): Promise<StopwatchDto> {
  const clean = name !== undefined && name.trim().length > 0 ? name : undefined;
  return invoke<StopwatchDto>("create_stopwatch", { name: clean ?? null });
}

export async function renameStopwatch(id: number, name: string): Promise<StopwatchDto> {
  return invoke<StopwatchDto>("rename_stopwatch", { id, name });
}

export async function startStopwatch(id: number): Promise<StopwatchDto> {
  return invoke<StopwatchDto>("start_stopwatch", { id });
}

export async function pauseStopwatch(id: number): Promise<StopwatchDto> {
  return invoke<StopwatchDto>("pause_stopwatch", { id });
}

export async function resetStopwatch(id: number): Promise<StopwatchDto> {
  return invoke<StopwatchDto>("reset_stopwatch", { id });
}

export async function deleteStopwatch(id: number): Promise<void> {
  return invoke<void>("delete_stopwatch", { id });
}

export async function enterFocusMode(id: number): Promise<void> {
  return invoke<void>("enter_focus_mode", { id });
}

export async function exitFocusMode(): Promise<void> {
  return invoke<void>("exit_focus_mode");
}

export type TimerStatus = "idle" | "running" | "paused" | "completed";

export interface TimerDto {
  id: number;
  name: string;
  duration_ms: number;
  remaining_ms: number;
  started_at_ms: number | null;
  status: TimerStatus;
  position: number;
}

export async function listTimers(): Promise<TimerDto[]> {
  return invoke<TimerDto[]>("list_timers");
}

export async function createTimer(name?: string, durationMs?: number): Promise<TimerDto> {
  const clean = name !== undefined && name.trim().length > 0 ? name : undefined;
  return invoke<TimerDto>("create_timer", {
    name: clean ?? null,
    durationMs: durationMs ?? null,
  });
}

export async function renameTimer(id: number, name: string): Promise<TimerDto> {
  return invoke<TimerDto>("rename_timer", { id, name });
}

export async function setTimerDuration(id: number, durationMs: number): Promise<TimerDto> {
  return invoke<TimerDto>("set_timer_duration", { id, durationMs });
}

export async function startTimer(id: number): Promise<TimerDto> {
  return invoke<TimerDto>("start_timer", { id });
}

export async function pauseTimer(id: number): Promise<TimerDto> {
  return invoke<TimerDto>("pause_timer", { id });
}

export async function completeTimer(id: number): Promise<TimerDto> {
  return invoke<TimerDto>("complete_timer", { id });
}

export async function resetTimer(id: number): Promise<TimerDto> {
  return invoke<TimerDto>("reset_timer", { id });
}

export async function deleteTimer(id: number): Promise<void> {
  return invoke<void>("delete_timer", { id });
}
