import { DateInput } from './date-input';
import { useEffect, useState, useRef } from 'react';
import { invoke, isTauri } from '@tauri-apps/api/core';
import { Plus, Pencil, Trash2, Check, Save, X, ChevronLeft, ChevronRight } from 'lucide-react';
import { ConfirmDelete } from './confirm-delete';
import { monthDays, moveMonth, compareTasks, brazilianDate } from './calendar';
import { ModelPicker } from './model-picker';
import { AgendaPanels, emptyAgenda, repeatOptions, runAgenda, type AgendaData, type Meeting, type MeetingAction } from './agenda-panels';
export type Scope = 'trabalho';
export type Task = {
    id: number | null;
    title: string;
    priority: string;
    due: string;
    done: boolean;
    token: string;
    area?:Scope; completedAt?:number|null; scheduledDue?:string; ruleId?:number|null; meetingId?:number|null; revision?:number;
};
type Note = {
    id: number | null;
    title: string;
    body: string;
    token: string;
};
export function taskInput(t:Task) {return {id:t.id,title:t.title,priority:t.priority,due:t.due,done:t.done,token:t.token};}
export function today() { return new Intl.DateTimeFormat('sv-SE', { timeZone: 'America/Bahia' }).format(new Date()); }

export const priorities = [{ id: 'alta', nome: 'Alta' }, { id: 'media', nome: 'Média' }, { id: 'baixa', nome: 'Baixa' }];
const err = (e: unknown) => typeof e === 'string' ? e : 'Não foi possível salvar. Tente novamente.';
export function Organization({ area }: {
    area: Scope;
}) {
    const [data, setData] = useState<{
        tasks: Task[];
        notes: Note[];
    } & AgendaData>({ tasks: [], notes: [], ...emptyAgenda });
    const [date, setDate] = useState(today);
    const [month, setMonth] = useState(() => today().slice(0, 7));
    const [filter, setFilter] = useState<'day' | 'pending' | 'done'>('day');
    const [tab,setTab]=useState<'tasks'|'rules'|'meetings'>('tasks');
    const [taskArea,setTaskArea]=useState<Scope>(area);
    const [repeat,setRepeat]=useState('none');const [until,setUntil]=useState('');
    const [meetingToOpen,setMeetingToOpen]=useState<number|undefined>();
    const [conversion,setConversion]=useState<{meeting:Meeting;action:MeetingAction}|null>(null);
    const [visibleCount,setVisibleCount]=useState(50);
    const [now,setNow]=useState(today);
    const [calendarOpen,setCalendarOpen]=useState(()=>window.matchMedia('(min-width:721px)').matches);
    useEffect(()=>{const media=window.matchMedia('(min-width:721px)');const update=()=>setCalendarOpen(media.matches);media.addEventListener('change',update);return()=>media.removeEventListener('change',update);},[]);
    const current=useRef(`${area}:${month}`);current.current=`${area}:${month}`;
    const [error, setError] = useState('');
    const [loading,setLoading]=useState(isTauri);
    const [busy, setBusy] = useState(false);
    const [task, setTask] = useState<Task | null>(null);
    const [note, setNote] = useState<Note | null>(null);
    const [deleting, setDeleting] = useState<{
        kind: string;
        id: number;
        area?:Scope;
    } | null>(null);
    const desktop = isTauri();
    useEffect(() => { if (task)
        document.getElementById('task-title')?.focus(); }, [task?.id, !!task]);
    useEffect(() => { if (note)
        document.getElementById('note-title')?.focus(); }, [note?.id, !!note]);
    function through() {const d=new Date(now+'T12:00:00Z');d.setUTCDate(d.getUTCDate()+366);const max=d.toISOString().slice(0,10);const base=new Date(now+'T12:00:00Z');base.setUTCDate(base.getUTCDate()+90);const end=new Date(Number(month.slice(0,4)),Number(month.slice(5)),0,12);const selected=`${month}-${String(end.getDate()).padStart(2,'0')}`;return [max,[base.toISOString().slice(0,10),selected].sort().at(-1)!].sort()[0];}
    async function fetchData() { return {...emptyAgenda,...await invoke<typeof data>('ler_organizacao',{area,through:through()})}; }
    async function load() {if(!desktop)return;setLoading(true);const key=`${area}:${month}`;try{const rows=await fetchData();if(current.current===key)setData(rows);}finally{if(current.current===key)setLoading(false);}}
    useEffect(()=>{setData({tasks:[],notes:[],...emptyAgenda});setTask(null);setNote(null);setDeleting(null);setError('');setTab('tasks');setConversion(null);setFilter('day');},[area]);
    useEffect(()=>{let live=true;if(desktop){setLoading(true);void fetchData().then(d=>{if(live)setData(d);}).catch(e=>{if(live)setError(err(e));}).finally(()=>{if(live)setLoading(false);});}return()=>{live=false;};},[area,desktop,month,now]);
    useEffect(()=>{const timer=setInterval(()=>setNow(today()),60000);return()=>clearInterval(timer);},[]);
    useEffect(()=>{if(!desktop)return;const refresh=()=>void load().catch(e=>setError(err(e)));window.addEventListener('focus',refresh);return()=>window.removeEventListener('focus',refresh);},[area,month]);
    useEffect(()=>setVisibleCount(50),[filter,date,area]);
    async function action(fn: () => Promise<unknown>) { setBusy(true); setError(''); try {
        await fn();
        await load();
    }
    catch (e) {
        setError(err(e));
    }
    finally {
        setBusy(false);
    } }
    useEffect(()=>{if(loading)return;const raw=sessionStorage.getItem('lunes.open.record');if(!raw)return;let target:{area:string;kind:string;id:number};try{target=JSON.parse(raw);}catch{sessionStorage.removeItem('lunes.open.record');return;}if(target.area!==area)return;sessionStorage.removeItem('lunes.open.record');if(target.kind==='task'){const row=data.tasks.find(t=>t.id===target.id);if(row){setTaskArea(area);setTask({...row});setRepeat('none');setDate(row.due);setMonth(row.due.slice(0,7));setTab('tasks');}else setError('A tarefa foi removida.');}else if(target.kind==='meeting'&&area==='trabalho'){if(data.meetings.some(m=>m.id===target.id)){setMeetingToOpen(target.id);setTab('meetings');}else setError('A reunião foi removida.');}},[data,loading,area]);
    function newTask() {setTaskArea(area);setTab('tasks');setRepeat('none');setUntil('');setConversion(null); setTask({ id: null, title: '', priority: 'media', due: date, done: false, token: crypto.randomUUID() }); }
    function newNote() { setNote({ id: null, title: '', body: '', token: crypto.randomUUID() }); }
    const pending = data.tasks.filter(t => !t.done && t.due === date);
    const visible = data.tasks.filter(t => t.due === date && (filter === 'done' ? t.done : filter === 'pending' ? !t.done : true)).sort((a,b)=>compareTasks(a,b,filter,now));
    function editTask(t:Task){setTaskArea(t.area??area);setTask({...t});setRepeat('none');setConversion(null);}
    function convertMeeting(meeting:Meeting,proposed:MeetingAction){setTab('tasks');setRepeat('none');setUntil('');setTaskArea('trabalho');setConversion({meeting,action:proposed});setTask({id:null,title:proposed.title,priority:proposed.priority,due:proposed.due,done:false,token:crypto.randomUUID()});}
    async function saveTask(){if(!task)return;if(conversion){await runAgenda(taskArea,'converter_reuniao',{id:conversion.meeting.id,action_token:conversion.action.token,title:task.title,priority:task.priority,due:task.due},conversion.meeting.revision);}else if(!task.id&&repeat!=='none'){await runAgenda(taskArea,'criar_recorrencia',{title:task.title,priority:task.priority,due:task.due,frequency:repeat,until:until||null,token:task.token});}else{await invoke('salvar_tarefa',{area:taskArea,task:taskInput(task),expectedRevision:task.revision??null});}setTask(null);setConversion(null);}
    const dateLabel = (value: string, options: Intl.DateTimeFormatOptions) => new Intl.DateTimeFormat('pt-BR', { ...options, timeZone: 'UTC' }).format(new Date(value + 'T12:00:00Z'));
    function moveDay(offset:number){const day=new Date(date+'T12:00:00Z');day.setUTCDate(day.getUTCDate()+offset);selectDate(day.toISOString().slice(0,10));}
    function selectDate(value: string) { setDate(value); setMonth(value.slice(0, 7)); setFilter('day'); }
    return <div className="organization" aria-busy={loading}><header className="organization-heading"><h2>{`${dateLabel(date, { weekday: 'long' })}, ${brazilianDate(date)}`}</h2><div className="agenda-actions"><button disabled={!desktop || busy || loading} onClick={newTask}><Plus size={16}/> Nova tarefa</button><button disabled={!desktop || busy || loading} onClick={newNote}><Plus size={16}/> Nova ideia</button></div></header>
 {loading&&<p role="status" className="nota">Carregando registros…</p>}
 <div className="agenda-date-navigation" role="group" aria-label="Escolher data"><button disabled={date<='1900-01-01'} onClick={()=>moveDay(-1)}><ChevronLeft size={16}/> Anterior</button><button onClick={()=>selectDate(now)}>Hoje</button><label className="visually-hidden" htmlFor="organization-date">Data escolhida</label><DateInput id="organization-date" min="1900-01-01" max="9999-12-31" value={date} onChange={value=>{if(value)selectDate(value);}}/><button disabled={date>='9999-12-31'} onClick={()=>moveDay(1)}>Próximo <ChevronRight size={16}/></button></div>
 {!desktop && <p className="aviso">Cadastro e salvamento disponíveis no desktop. Esta prévia não inventa registros.</p>}{error && <p role="alert" className="aviso">{error}</p>}{data.generationError&&<p role="alert" className="aviso">As próximas ocorrências não foram preparadas: {data.generationError} Os registros existentes continuam disponíveis.</p>}
 <nav className="organization-tabs" aria-label="Seções da área">{([{id:'tasks',label:'Tarefas'},{id:'rules',label:'Repetições'},{id:'meetings',label:'Reuniões'}] as {id:typeof tab;label:string}[]).map(t=><button key={t.id} aria-pressed={tab===t.id} onClick={()=>{setTab(t.id);setTask(null);setNote(null);}}>{t.label}</button>)}</nav>

 {tab!=='tasks'&&<AgendaPanels key={`${area}:${tab}`} area={area} data={data} tab={tab} refresh={load} desktop={desktop} onConvert={convertMeeting} openMeeting={meetingToOpen}/>}
 {tab==='tasks'&&<><div className="agenda-layout"><aside className="agenda-calendar" aria-label="Calendário de tarefas"><details className="calendar-details" open={calendarOpen} onToggle={e=>setCalendarOpen(e.currentTarget.open)}><summary>Escolher dia · {brazilianDate(date)}</summary><div className="calendar-heading"><button aria-label="Mês anterior" onClick={() => setMonth(moveMonth(month, -1))}><ChevronLeft size={16}/></button><strong>{dateLabel(month + '-01', { month: 'long', year: 'numeric' }).replace(/^./,letter=>letter.toUpperCase())}</strong><button aria-label="Próximo mês" onClick={() => setMonth(moveMonth(month, 1))}><ChevronRight size={16}/></button></div><div className="calendar-week" aria-hidden="true">{['S','T','Q','Q','S','S','D'].map((d,i) => <span key={i}>{d}</span>)}</div><div className="calendar-days">{monthDays(month).map(day => <button key={day} aria-label={dateLabel(day, { dateStyle: 'full' })} aria-pressed={day === date} aria-current={day === today() ? 'date' : undefined} className={day.slice(0,7) === month ? '' : 'outside-month'} onClick={() => selectDate(day)}><span>{Number(day.slice(8))}</span>{data.tasks.some(t => t.due === day && !t.done) && <span className="calendar-dot" aria-label="Tarefas pendentes"/>}</button>)}</div><button className="calendar-today" onClick={() => selectDate(today())}>Ir para hoje</button></details><section className="agenda-priorities" aria-label="Prioridades pendentes"><h3>Pendências por prioridade</h3>{priorities.map(p => <div key={p.id}><span>{p.nome}</span><strong>{pending.filter(t => t.priority === p.id).length}</strong></div>)}</section>{area==='trabalho'&&data.meetings.some(m=>m.date>=now)&&<section className="upcoming-meetings"><h3>Próximas reuniões</h3><p className="nota">Rascunhos por data</p>{data.meetings.filter(m=>m.date>=now).sort((a,b)=>a.date.localeCompare(b.date)).slice(0,3).map(m=><button key={m.id} onClick={()=>{setTab('meetings');}}><strong>{m.title}</strong><small>{brazilianDate(m.date)}</small></button>)}</section>}</aside>
 <section className="agenda-tasks" aria-label="Tarefas"><div className="agenda-filters" aria-label="Filtrar tarefas">{([{id:'day',label:'Todas do dia'},{id:'pending',label:'Pendentes do dia'},{id:'done',label:'Concluídas do dia'}] as {id:typeof filter;label:string}[]).map(f => <button key={f.id} aria-pressed={filter === f.id} onClick={() => setFilter(f.id)}>{f.label}</button>)}</div><div className="agenda-list-heading"><h2>{filter === 'day' ? 'Seu dia' : filter === 'pending' ? 'Pendentes neste dia' : 'Tarefas concluídas'}</h2><span>{visible.length} {visible.length === 1 ? 'tarefa' : 'tarefas'}</span></div>
 {!visible.length && !loading && <div className="agenda-empty"><h3>{filter === 'done' ? 'Nenhuma tarefa concluída ainda' : filter === 'pending' ? 'Tudo em dia' : 'Um dia livre por aqui'}</h3><p>{filter === 'done' ? 'As tarefas que você concluir aparecerão aqui.' : 'Crie uma tarefa ou escolha outro dia no calendário.'}</p></div>}{visible.slice(0,visibleCount).map(t => <div className="task-group" key={t.id}><div className="task-row"><button disabled={busy} aria-label={t.done ? 'Reabrir tarefa ' + t.title : 'Concluir tarefa ' + t.title} onClick={() => void action(() => invoke('salvar_tarefa', { area:t.area??area, task: { ...taskInput(t), done: !t.done },expectedRevision:t.revision??null }))}>{t.done ? <Check size={18}/> : <span className="task-check"/>}</button><div><strong className={t.done ? 'completed-task' : ''}>{t.title}</strong><div className="task-meta"><small className={`priority-label priority-${t.priority}`}>{priorities.find(p => p.id === t.priority)?.nome}</small><small>{brazilianDate(t.due)}{!t.done && t.due < now ? ' · Atrasada' : ''}</small>{t.ruleId&&<small>Recorrente</small>}{t.meetingId&&<small>Da reunião</small>}{t.completedAt&&<small>Concluída em {new Intl.DateTimeFormat('pt-BR',{dateStyle:'short',timeZone:'America/Bahia'}).format(new Date(t.completedAt*1000))}</small>}{t.done&&!t.completedAt&&<small>Conclusão anterior · data não registrada</small>}</div></div><button disabled={busy} aria-label={'Editar tarefa ' + t.title} onClick={() => editTask(t)}><Pencil size={16}/></button><button disabled={busy} aria-label={'Excluir tarefa ' + t.title} onClick={() => { setError(''); setDeleting({ kind: 'task', id: t.id!, area:t.area??area }); }}><Trash2 size={16}/></button></div></div>)}{visible.length>visibleCount&&<button onClick={()=>setVisibleCount(visibleCount+50)}>Mostrar mais tarefas</button>}</section></div>
 {<section className="ideas"><div className="section-heading"><div><h2>Suas ideias</h2><p>Um lugar para guardar o que você quer desenvolver.</p></div><button disabled={!desktop || busy || loading} onClick={newNote}><Plus size={16}/> Nova ideia</button></div>{!data.notes.length && !loading && <p>Nenhuma ideia salva ainda.</p>}<div className="ideas-grid">{data.notes.map(n => <article className="idea-card" key={n.id}><h3><button className="idea-title" disabled={busy} onClick={() => setNote(n)}>{n.title}</button></h3><p>{n.body.length > 220 ? n.body.slice(0, 220) + '…' : n.body}</p><div><button disabled={busy} onClick={() => setNote(n)}><Pencil size={16}/> Editar</button><button disabled={busy} aria-label={'Excluir ideia ' + n.title} onClick={() => { setError(''); setDeleting({ kind: 'note', id: n.id! }); }}><Trash2 size={16}/></button></div></article>)}</div></section>}
 </>}
 {task && <section className="painel record-editor" aria-label="Editor de tarefa"><div className="section-heading"><h2>{conversion?'Conferir tarefa da reunião':task.id ? 'Editar ocorrência' : 'Nova tarefa'}</h2><button aria-label="Fechar editor de tarefa" disabled={busy} onClick={() => setTask(null)}><X size={18}/></button></div><form onSubmit={e => { e.preventDefault(); void action(async () => { await saveTask(); }); }}><label htmlFor="task-title">Título</label><input id="task-title" disabled={busy} value={task.title} onChange={e => setTask({ ...task, title: e.target.value })} required maxLength={200}/><div className="editor-fields"><div><label id="task-priority-label" htmlFor="task-priority">Prioridade</label><ModelPicker id="task-priority" options={priorities} value={task.priority} disabled={busy} onChange={priority => setTask({ ...task, priority })}/></div><div><label htmlFor="task-due">Data</label><DateInput id="task-due" disabled={busy} value={task.due} required onChange={value => setTask({ ...task, due: value })}/></div></div>{!task.id&&!conversion&&<div className="editor-fields"><div><label id="task-repeat-label" htmlFor="task-repeat">Repetição</label><ModelPicker id="task-repeat" options={repeatOptions} value={repeat} disabled={busy} onChange={setRepeat}/></div>{repeat!=='none'&&<div><label htmlFor="task-until">Repetir até (opcional)</label><DateInput id="task-until" min={task.due} value={until} disabled={busy} onChange={value=>setUntil(value)}/></div>}</div>}{repeat==='weekly'&&<p className="nota">Repete no mesmo dia da semana da data inicial.</p>}{repeat==='monthly'&&<p className="nota">Usa o dia da data inicial. Nos meses mais curtos, usa o último dia e depois volta ao dia original.</p>}{task.ruleId&&<p className="nota">Esta edição altera somente esta ocorrência. Para alterar as próximas, abra Repetições.</p>}{conversion&&<p className="nota">Confira título, prioridade e prazo antes de criar. Reunião: {conversion.meeting.title}.</p>}<button disabled={busy} type="submit"><Save size={16}/> Salvar tarefa</button></form></section>}
 {note && <section className="painel record-editor" aria-label="Editor de ideia"><div className="section-heading"><h2>{note.id ? 'Editar ideia' : 'Nova ideia'}</h2><button aria-label="Fechar editor de ideia" disabled={busy} onClick={() => setNote(null)}><X size={18}/></button></div><form onSubmit={e => { e.preventDefault(); void action(async () => { await invoke('salvar_nota', { area, note }); setNote(null); }); }}><label htmlFor="note-title">Título da ideia</label><input id="note-title" disabled={busy} value={note.title} onChange={e => setNote({ ...note, title: e.target.value })} required maxLength={200}/><label htmlFor="note-body">Sua ideia</label><textarea id="note-body" disabled={busy} rows={10} value={note.body} onChange={e => setNote({ ...note, body: e.target.value })} required maxLength={65536} placeholder="Escreva sua ideia…"/><p className="nota">Texto simples, até 64 KiB. O conteúdo permanece nesta área.</p><button type="submit" disabled={busy}><Save size={16}/> Salvar ideia</button></form></section>}
 {deleting && <ConfirmDelete title={deleting.kind === 'note' ? 'Excluir ideia?' : 'Excluir tarefa?'} description={`“${(deleting.kind === 'note' ? data.notes : data.tasks).find(item => item.id === deleting.id)?.title ?? 'Registro'}” será removido desta área. Esta ação não pode ser desfeita.`} busy={busy} error={error} onCancel={() => setDeleting(null)} onConfirm={() => void action(async () => { await invoke('excluir_registro', { area:deleting.area??area, kind:deleting.kind,id:deleting.id }); if (deleting.kind === 'note' && note?.id === deleting.id) setNote(null); if (deleting.kind === 'task' && task?.id === deleting.id) setTask(null); setDeleting(null); })}/>}

 </div>;
}
