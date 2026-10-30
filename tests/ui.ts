// Apenas dados sintéticos para conferir a interface; não comprova o backend.
import {mockIPC,mockWindows} from '@tauri-apps/api/mocks';
const tasks:Record<string,any>[]=[];
const notes:Record<string,any>[]=[];
const meetings:Record<string,any>[]=[];
const goals:Record<string,any>[]=[];
let serial=0;
Object.assign(window,{isTauri:true});mockWindows('main');
mockIPC((command,args:any)=>{
 if(command.startsWith('plugin:'))return false;
 if(command==='ler_tema')return 'claro';if(command==='salvar_tema')return;
 if(location.search==='?fail'&&!command.startsWith('ler_'))throw 'Falha simulada. Seu formulário foi preservado.';
 if(args?.area&&args.area!=='trabalho')throw 'Área inválida nesta versão.';
 if(command==='ler_organizacao')return structuredClone({tasks,notes,rules:[],journal:[],events:[],meetings});
 if(command==='salvar_tarefa'||command==='salvar_tarefa_meta'){
  const row=tasks.find(t=>t.id===args.task.id);if(row){Object.assign(row,args.task);return row.id;}
  const id=++serial;tasks.push({...args.task,id,goalId:args.goalId,revision:0});return id;
 }
 if(command==='salvar_nota'){const row=notes.find(n=>n.id===args.note.id);if(row){Object.assign(row,args.note);return row.id;}const id=++serial;notes.push({...args.note,id});return id;}
 if(command==='excluir_registro'){const rows=args.kind==='task'?tasks:notes;const index=rows.findIndex(t=>t.id===args.id);if(index>=0)rows.splice(index,1);return;}
 if(command==='executar_agenda'){
  const a=args.args;if(args.tool==='salvar_reuniao'){const row=meetings.find(m=>m.id===a.id);if(row){Object.assign(row,a);return {meeting:row.id};}const id=++serial;meetings.push({...a,id,converted:[],revision:0});return {meeting:id};}
  throw 'Operação não simulada.';
 }
 if(command==='ler_metas'){const rows=goals.map(g=>{const steps=tasks.filter(t=>t.goalId===g.id);const completed=steps.filter(t=>t.done).length;return {...g,total:steps.length,completed,progress:steps.length?Math.floor(completed*100/steps.length):0};});const selected=rows.find(g=>g.id===args.goalId)?.id??rows[0]?.id??null;return structuredClone({area:'trabalho',goals:rows,selected,tasks:tasks.filter(t=>t.goalId===selected)});}
 if(command==='salvar_meta'){const row=goals.find(g=>g.id===args.goal.id);if(row){Object.assign(row,args.goal);return row.id;}const id=++serial;goals.push({...args.goal,id,revision:1});return id;}
 if(command==='exportar_dados')return 'backup-simulado.sqlite3';if(command==='restaurar_dados')return;
 throw `Comando não simulado: ${command}`;
},{shouldMockEvents:true});
void import('../src/main');
