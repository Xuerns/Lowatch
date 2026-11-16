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
