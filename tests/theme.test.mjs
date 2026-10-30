import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import ts from 'typescript';
import { mockIPC, clearMocks } from '@tauri-apps/api/mocks';
globalThis.window = { crypto: globalThis.crypto };
let legacy = 'escuro';
globalThis.localStorage = {
  getItem: () => legacy,
  setItem: (_, value) => { legacy = value; },
  removeItem: () => { legacy = null; },
};
const compiled = ts.transpileModule(readFileSync(new URL('../src/theme.ts', import.meta.url), 'utf8'), { compilerOptions: { module: ts.ModuleKind.ESNext } }).outputText.replaceAll('@tauri-apps/api/core', import.meta.resolve('@tauri-apps/api/core'));
const { carregarTema, salvarTema } = await import(`data:text/javascript,${encodeURIComponent(compiled)}`);
globalThis.isTauri = true;
mockIPC((command, args) => {
  assert.equal(command, 'ler_tema');
  assert.deepEqual(args, { legado: 'escuro' });
  throw new Error('disco indisponível');
});
await assert.rejects(carregarTema());
assert.equal(legacy, 'escuro');
mockIPC((command, args) => {
  if (command === 'ler_tema') { assert.equal(args.legado, 'escuro'); return 'claro'; }
  assert.equal(command, 'salvar_tema');
  assert.equal(args.tema, 'escuro');
});
assert.equal(await carregarTema(), 'claro');
assert.equal(legacy, null);
await salvarTema('escuro');
mockIPC(() => { throw new Error('falha'); });
await assert.rejects(salvarTema('claro'));
globalThis.isTauri = false;
await salvarTema('escuro');
assert.equal(await carregarTema(), 'escuro');
clearMocks();
console.log('Tema: IPC simulado, migração sem perda na falha, SQLite prevalece e prévia no navegador passaram.');
