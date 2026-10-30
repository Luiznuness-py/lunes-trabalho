import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import ts from 'typescript';

const source = readFileSync(new URL('../src/calendar.ts', import.meta.url), 'utf8');
const { outputText } = ts.transpileModule(source, { compilerOptions: { module: ts.ModuleKind.ESNext } });
const { monthDays, moveMonth, compareTasks, brazilianDate, parseBrazilianDate, calendarDisplayDate } = await import('data:text/javascript;base64,' + Buffer.from(outputText).toString('base64'));
assert.equal(moveMonth('2026-12', 1), '2027-01');
assert.equal(moveMonth('2026-01', -1), '2025-12');
const leap = monthDays('2024-02');
assert.equal(leap.length, 42);
assert.equal(leap[0], '2024-01-29');
assert.equal(new Set(leap).size, 42);
assert.ok(leap.includes('2024-02-29'));
assert.equal(monthDays('2026-02').filter(day => day.startsWith('2026-02')).length, 28);
assert.equal(monthDays('2026-03')[0], '2026-02-23');
assert.equal(monthDays('2026-06')[0], '2026-06-01');
console.log('Calendário: semanas, ano e fevereiro validados.');

const rows=[{due:'2026-10-15',priority:'alta',done:false},{due:'2026-10-14',priority:'baixa',done:false},{due:'2026-10-01',priority:'baixa',done:false},{due:'2026-10-13',priority:'alta',done:false}];
assert.deepEqual([...rows].sort((a,b)=>compareTasks(a,b,'overview','2026-10-14')).map(t=>t.due),['2026-10-13','2026-10-01','2026-10-14','2026-10-15']);
assert.ok(compareTasks({due:'2026-10-01',priority:'alta',done:true,completedAt:100},{due:'2026-10-14',priority:'baixa',done:true,completedAt:200},'done','2026-10-14')>0);

assert.equal(brazilianDate('2026-10-14'),'14/10/2026');
assert.equal(parseBrazilianDate('14/10/2026'),'2026-10-14');
assert.equal(parseBrazilianDate('29/02/2024'),'2024-02-29');
for(const value of ['29/02/2026','31/04/2026','10/14/2026','1/2/2026','2026-10-14','01/01/0000',''])assert.equal(parseBrazilianDate(value),null);
console.log('Datas brasileiras: formato e datas inválidas validados.');

assert.equal(calendarDisplayDate('2026-10-14',null,0),'2026-10-14');
assert.equal(calendarDisplayDate('2026-10-14','01:30',0),'2026-10-13');
assert.equal(calendarDisplayDate('2026-10-14','01:30',-180),'2026-10-14');
