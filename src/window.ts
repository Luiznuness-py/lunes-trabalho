import { isTauri } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';

export async function controlarJanela(acao: 'minimize' | 'toggleMaximize' | 'close' | 'startDragging') {
  if (!isTauri()) return;
  const janela = getCurrentWindow();
  await janela[acao]();
  if (acao === 'toggleMaximize') return janela.isMaximized();
}
