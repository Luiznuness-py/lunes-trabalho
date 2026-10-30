import { readFileSync } from 'node:fs';
import assert from 'node:assert/strict';

const css = readFileSync(new URL('../src/styles.css', import.meta.url), 'utf8');
function luminancia(hex) {
  const rgb = hex.match(/\w{2}/g).map(v => parseInt(v, 16) / 255)
    .map(v => v <= .04045 ? v / 12.92 : ((v + .055) / 1.055) ** 2.4);
  return rgb[0] * .2126 + rgb[1] * .7152 + rgb[2] * .0722;
}
function contraste(a, b) {
  const valores = [luminancia(a), luminancia(b)].sort((x, y) => y - x);
  return (valores[0] + .05) / (valores[1] + .05);
}
for (const seletor of [':root', ':root[data-tema=escuro]']) {
  const bloco = css.slice(css.indexOf(seletor)).split('}')[0];
  const tokens = Object.fromEntries([...bloco.matchAll(/--([\w-]+):\s*(#[\da-f]{6})/g)].map(m => [m[1], m[2]]));
  for (const fundo of ['fundo', 'superficie', 'lateral', 'destaque']) {
    for (const texto of ['texto', 'secundario', 'destaque-texto']) {
      const ratio = contraste(tokens[texto], tokens[fundo]);
      assert.ok(ratio >= 4.5, `${seletor}: ${texto}/${fundo} ${ratio.toFixed(2)} < 4.5`);
    }
    assert.ok(contraste(tokens.foco, tokens[fundo]) >= 3, `${seletor}: foco/${fundo}`);
  }
}
console.log('Contraste dos dois temas: textos ≥ 4.5:1 e foco ≥ 3:1.');
