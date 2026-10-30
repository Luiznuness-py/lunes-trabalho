import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import ts from 'typescript';
import { mockIPC, mockWindows, clearMocks } from '@tauri-apps/api/mocks';

// A API oficial é simulada; este teste não comprova cliques na janela nativa.
globalThis.window = { crypto: globalThis.crypto };
const fonte = readFileSync(new URL('../src/window.ts', import.meta.url), 'utf8');
const compilado = ts.transpileModule(fonte, { compilerOptions: { module: ts.ModuleKind.ESNext } }).outputText
  .replaceAll('@tauri-apps/api/core', import.meta.resolve('@tauri-apps/api/core'))
  .replaceAll('@tauri-apps/api/window', import.meta.resolve('@tauri-apps/api/window'));
const { controlarJanela } = await import(`data:text/javascript,${encodeURIComponent(compilado)}`);
const chamadas = [];
mockWindows('main');
mockIPC((comando, args) => {
  chamadas.push(comando);
  assert.equal(args.label, 'main');
  return comando === 'plugin:window|is_maximized';
});
globalThis.isTauri = false;
await controlarJanela('close');
assert.equal(chamadas.length, 0);
globalThis.isTauri = true;
for (const acao of ['minimize', 'startDragging', 'toggleMaximize', 'close']) {
  const estado = await controlarJanela(acao);
  assert.equal(estado, acao === 'toggleMaximize' ? true : undefined);
}
assert.deepEqual(chamadas, ['plugin:window|minimize', 'plugin:window|start_dragging', 'plugin:window|toggle_maximize', 'plugin:window|is_maximized', 'plugin:window|close']);
mockIPC(() => { throw new Error('Negado'); });
await assert.rejects(controlarJanela('minimize'), /Negado/);
clearMocks();
delete globalThis.window;
delete globalThis.isTauri;
console.log('Controles: janela main, navegador sem IPC, maximização e falha simuladas passaram.');
