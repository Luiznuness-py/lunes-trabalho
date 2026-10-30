import { invoke, isTauri } from '@tauri-apps/api/core';
export type Tema = 'claro' | 'escuro';
export function temaLegado(): Tema | null {
  const value = localStorage.getItem('lunes-trabalho.tema');
  return value === 'claro' || value === 'escuro' ? value : null;
}
export async function carregarTema(): Promise<Tema> {
  let legado: Tema | null = null;
  try { legado = temaLegado(); } catch { /* SQLite continua disponível sem localStorage. */ }
  if (!isTauri()) return legado ?? 'claro';
  const tema = await invoke<Tema>('ler_tema', { legado });
  // Só remove a preferência antiga depois de confirmar a transação no backend.
  try { localStorage.removeItem('lunes-trabalho.tema'); } catch { /* Importação idempotente. */ }
  return tema;
}
export async function salvarTema(tema: Tema): Promise<void> {
  if (isTauri()) await invoke('salvar_tema', { tema });
  else localStorage.setItem('lunes-trabalho.tema', tema);
}
