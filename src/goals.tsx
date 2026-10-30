import {useCallback,useEffect,useRef,useState} from 'react';
import {invoke,isTauri} from '@tauri-apps/api/core';
import {listen} from '@tauri-apps/api/event';
import {Target,Plus,ArrowLeft,ArrowUpRight,Check,Pencil,Trash2,X,CalendarDays,Flag} from 'lucide-react';
import {DateInput} from './date-input';
import {ModelPicker} from './model-picker';
import {ConfirmDelete} from './confirm-delete';
import {today,priorities,taskInput,type Scope,type Task} from './organization';
import {brazilianDate} from './calendar';
import './goals.css';

export type Goal={id:number;title:string;description:string;due:string|null;token:string;revision:number;total:number;completed:number;progress:number};
type GoalInput=Pick<Goal,'title'|'description'|'due'|'token'>&{id:number|null};
type GoalData={area:string;goals:Goal[];selected:number|null;tasks:Task[]};
const empty:GoalData={area:'',goals:[],selected:null,tasks:[]};
const failure=(error:unknown)=>typeof error==='string'?error:'Não foi possível concluir. Tente novamente.';
export const goalComplete=(goal:Goal)=>goal.total>0&&goal.completed===goal.total;
function Progress({goal}:{goal:Goal}){return <div className="goal-progress"><div><span>{goal.total===0?'Adicione sua primeira tarefa':`${goal.completed} de ${goal.total} ${goal.total===1?'tarefa concluída':'tarefas concluídas'}`}</span><strong>{goal.progress}%</strong></div><progress max={100} value={goal.progress} aria-label={`Progresso de ${goal.title}`}>{goal.progress}%</progress></div>;}
export function useGoals(area:string){
    const [data,setData]=useState<GoalData>(empty);
    const [selected,setSelected]=useState<number|null>(null);
    const selection=useRef(selected);selection.current=selected;
    const owner=useRef(area);owner.current=area;
    const generation=useRef(0);
    const [loading,setLoading]=useState(isTauri);
    const [error,setError]=useState('');
    const refresh=useCallback(async(id=selection.current)=>{
        if(!isTauri()||!['trabalho'].includes(area)){setLoading(false);return;}
        const request=++generation.current;setLoading(true);
        try{const result=await invoke<GoalData>('ler_metas',{area,goalId:id});if(owner.current===area&&generation.current===request){setData(result);setError('');}}
        catch(error){if(owner.current===area&&generation.current===request)setError(failure(error));}
        finally{if(owner.current===area&&generation.current===request)setLoading(false);}
    },[area]);
    useEffect(()=>{void refresh();return()=>{generation.current++;};},[refresh,selected]);
    useEffect(()=>{
        if(!isTauri())return;
        let alive=true;let off:(()=>void)|undefined;
        const update=()=>void refresh();window.addEventListener('focus',update);
        const timer=setInterval(update,15000);
        void listen<string>('metas-alteradas',event=>{if(event.payload===area)update();}).then(fn=>{if(alive)off=fn;else fn();}).catch(()=>{});
        return()=>{alive=false;off?.();clearInterval(timer);window.removeEventListener('focus',update);};
    },[refresh,area]);
    return {data:data.area===area?data:empty,loading,error,refresh,setSelected};
}
export function Goals({area='trabalho',onAreaChange,compact=false,onOpen}:{area?:Scope;onAreaChange?:(area:Scope)=>void;compact?:boolean;onOpen?:()=>void}){
    const {data,loading,error:readError,refresh,setSelected}=useGoals(area);
    const [filter,setFilter]=useState('active');
    const [detailOpen,setDetailOpen]=useState(!compact);
    const [goalDraft,setGoalDraft]=useState<{owner:Scope;goal:GoalInput;revision:number|null}|null>(null);
    const [taskDraft,setTaskDraft]=useState<{owner:Scope;goal:Goal;task:Task}|null>(null);
    const [deleting,setDeleting]=useState<{owner:Scope;goal:Goal;task?:Task}|null>(null);
    const [busy,setBusy]=useState(false);const working=useRef(false);
    const currentArea=useRef(area);currentArea.current=area;
    const [error,setError]=useState('');const [notice,setNotice]=useState('');
    const desktop=isTauri();const scope:Scope='trabalho';
    const goal=data.goals.find(goal=>goal.id===data.selected);
    const active=data.goals.filter(goal=>!goalComplete(goal));
    const remaining=data.goals.reduce((n,goal)=>n+goal.total-goal.completed,0);
    const accomplished=data.goals.length-active.length;
    const visible=compact?data.goals:data.goals.filter(goal=>filter==='all'||goalComplete(goal)===(filter==='done'));
    const editor=useRef<HTMLElement>(null);
    const detail=useRef<HTMLDivElement>(null);const scrollToDetail=useRef(false);
    useEffect(()=>{if(!loading&&scrollToDetail.current){scrollToDetail.current=false;detail.current?.scrollIntoView({block:'start'});detail.current?.focus();}},[loading,data.selected]);
    useEffect(()=>{if(goalDraft||taskDraft){editor.current?.scrollIntoView({block:'nearest'});editor.current?.querySelector<HTMLInputElement>('input')?.focus();}},[!!goalDraft,!!taskDraft]);
    const editorOwner=goalDraft?.owner??taskDraft?.owner;
    const editing=Boolean(goalDraft||taskDraft);
    async function commit(owner:Scope,write:()=>Promise<unknown>,after:(result:unknown)=>void,message:string){
        if(working.current)return;
        working.current=true;setBusy(true);setError('');setNotice('');
        try{const result=await write();after(result);if(currentArea.current===owner){setNotice(message);void refresh();}}
        catch(error){setError(`${'Trabalho'}: ${failure(error)}`);}
        finally{working.current=false;setBusy(false);}
    }
    function createGoal(){setTaskDraft(null);setGoalDraft({owner:scope,revision:null,goal:{id:null,title:'',description:'',due:null,token:crypto.randomUUID()}});setDetailOpen(true);setError('');setNotice('');}
    function editGoal(goal:Goal){setTaskDraft(null);setGoalDraft({owner:scope,revision:goal.revision,goal:{id:goal.id,title:goal.title,description:goal.description,due:goal.due,token:goal.token}});setError('');}
    function editTask(goal:Goal,task?:Task){setGoalDraft(null);setTaskDraft({owner:scope,goal,task:task?{...task}:{id:null,title:'',due:goal.due&&goal.due>=today()?goal.due:today(),priority:'media',done:false,token:crypto.randomUUID()}});setError('');setNotice('');}
    async function saveGoal(){
        if(!goalDraft||goalDraft.owner!==area)return;
        const snapshot={...goalDraft,goal:{...goalDraft.goal}};
        await commit(snapshot.owner,()=>invoke<number>('salvar_meta',{area:snapshot.owner,goal:snapshot.goal,expectedRevision:snapshot.revision}),result=>{setGoalDraft(null);if(currentArea.current===snapshot.owner){setSelected(result as number);setFilter('all');}},'Meta salva. Agora avance uma tarefa de cada vez.');
    }
    async function saveTask(){
        if(!taskDraft||taskDraft.owner!==area)return;
        const snapshot={...taskDraft,task:taskInput(taskDraft.task),revision:taskDraft.task.revision};
        await commit(snapshot.owner,()=>snapshot.task.id===null?invoke('salvar_tarefa_meta',{area:snapshot.owner,goalId:snapshot.goal.id,task:snapshot.task,expectedRevision:snapshot.goal.revision}):invoke('salvar_tarefa',{area:snapshot.owner,task:snapshot.task,expectedRevision:snapshot.revision}),()=>setTaskDraft(null),'Tarefa salva e vinculada à meta.');
    }
    async function toggle(task:Task){
        const owner=scope;const snapshot={...taskInput(task),done:!task.done};
        await commit(owner,()=>invoke('salvar_tarefa',{area:owner,task:snapshot,expectedRevision:task.revision}),()=>{},snapshot.done?'Mais um passo concluído.':'Tarefa reaberta.');
    }
    const taskRow=(task:Task)=><li className="goal-task" key={task.id}><button className="goal-task-check" disabled={busy||loading||editing} aria-label={`${task.done?'Reabrir':'Concluir'} tarefa ${task.title}`} onClick={()=>void toggle(task)}>{task.done?<Check size={17} aria-hidden="true"/>:<span/>}</button><div><strong className={task.done?'goal-task-done':''}>{task.title}</strong><small><span>{priorities.find(priority=>priority.id===task.priority)?.nome}</span><span>{brazilianDate(task.due)}{!task.done&&task.due<today()?' · Atrasada':''}</span></small></div>{!compact&&<><button disabled={busy||loading||editing} aria-label={`Editar tarefa ${task.title}`} onClick={()=>goal&&editTask(goal,task)}><Pencil size={16}/></button><button disabled={busy||loading||editing} aria-label={`Excluir tarefa ${task.title}`} onClick={()=>goal&&setDeleting({owner:scope,goal,task})}><Trash2 size={16}/></button></>}</li>;
    return <section className={`goals-page${compact?' goals-compact':''}`} aria-label="Suas metas" aria-busy={loading}>
        <header className="goals-toolbar"><div>{compact&&detailOpen?<button className="goals-back" disabled={busy} onClick={()=>setDetailOpen(false)}><ArrowLeft size={14}/> Metas</button>:<><span className="sobrelinha">UM PASSO DE CADA VEZ</span><h2>{compact?'Metas':'Dê uma direção às suas tarefas.'}</h2></>}{!compact&&<p>Defina o que quer alcançar e acompanhe cada avanço.</p>}</div><div className="goals-actions">{compact?<button onClick={onOpen}>Gerenciar<ArrowUpRight size={14}/></button>:<button className="botao-primario" disabled={!desktop||busy||editing||loading} onClick={createGoal}><Plus size={16}/> Nova meta</button>}</div></header>
        {!desktop&&<p className="aviso">Abra o Lunes desktop para criar e acompanhar suas metas.</p>}
        {editorOwner&&editorOwner!==area&&<p role="status" className="aviso">Há uma edição em {'Trabalho'}. <button onClick={()=>onAreaChange?.(editorOwner)}>Retomar edição</button></p>}
        {(readError||error)&&!deleting&&<p role="alert" className="aviso">{error||readError}{readError&&<button disabled={busy} onClick={()=>void refresh()}>Tentar novamente</button>}</p>}
        {notice&&<p role="status" className="goals-notice"><Check size={15} aria-hidden="true"/>{notice}</p>}
        {loading&&<p role="status" className="nota">Atualizando metas…</p>}
        {!compact&&data.goals.length>0&&<div className="goals-stats"><div><Target size={18}/><span><strong>{active.length}</strong> {active.length===1?'meta em andamento':'metas em andamento'}</span></div><div><Flag size={18}/><span><strong>{remaining}</strong> {remaining===1?'tarefa pendente':'tarefas pendentes'}</span></div><div><Check size={18}/><span><strong>{accomplished}</strong> {accomplished===1?'meta alcançada':'metas alcançadas'}</span></div></div>}
        {!data.goals.length&&!loading&&!readError&&!goalDraft?<div className="goals-empty"><span><Target size={30} aria-hidden="true"/></span><h3>Uma meta começa com um próximo passo.</h3><p>{compact?'Crie uma meta no Lunes e acompanhe suas tarefas por aqui.':'Pode ser um projeto, uma nova habilidade ou uma entrega do trabalho. Divida sua meta em tarefas possíveis.'}</p><button className="botao-primario" disabled={!desktop} onClick={compact?onOpen:createGoal}>{compact?'Criar no Lunes':'Criar minha primeira meta'}<Plus size={16}/></button></div>:<div className={`goals-layout${data.goals.length?'':' goals-first'}`}>
            {(!compact||!detailOpen)&&<aside className="goals-list" aria-label="Lista de metas">{!compact&&<div className="goals-filters" role="group" aria-label="Filtrar metas">{[['active','Em andamento'],['done','Alcançadas'],['all','Todas']].map(([id,name])=><button key={id} aria-pressed={filter===id} disabled={busy||editing} onClick={()=>setFilter(id)}>{name}</button>)}</div>}{!visible.length&&!goalDraft&&<p className="goals-list-empty">Nenhuma meta neste filtro.</p>}{visible.map(item=><button key={item.id} className="goal-card" aria-current={detailOpen&&data.selected===item.id?'true':undefined} disabled={busy||editing||loading} onClick={()=>{setSelected(item.id);setDetailOpen(true);setNotice('');setError('');if(!compact&&window.matchMedia('(max-width:760px)').matches){if(data.selected===item.id){detail.current?.scrollIntoView({block:'start'});detail.current?.focus();}else scrollToDetail.current=true;}}}><div className="goal-card-title"><Target size={17}/><strong>{item.title}</strong>{goalComplete(item)&&<Check size={16}/>}</div><Progress goal={item}/><small>{item.due?`Até ${brazilianDate(item.due)}`:'Sem prazo definido'}{goalComplete(item)?' · Alcançada':item.due&&item.due<today()?' · Prazo passou':''}</small></button>)}</aside>}
            {detailOpen&&<div className="goal-detail" ref={detail} tabIndex={-1} role="region" aria-label="Detalhes da meta">
                {goalDraft?.owner===area?<section className="goal-editor" ref={editor}><header><h3>{goalDraft.goal.id?'Editar meta':'Qual é a sua próxima meta?'}</h3><button disabled={busy} aria-label="Cancelar edição da meta" onClick={()=>setGoalDraft(null)}><X size={17}/></button></header><form onSubmit={event=>{event.preventDefault();void saveGoal();}}><label htmlFor="goal-title">O que você quer alcançar?</label><input id="goal-title" required maxLength={200} disabled={busy} value={goalDraft.goal.title} placeholder="Ex.: Lançar meu portfólio" onChange={event=>setGoalDraft({...goalDraft,goal:{...goalDraft.goal,title:event.target.value}})}/><label htmlFor="goal-description">Por que essa meta é importante? <span>(opcional)</span></label><textarea id="goal-description" rows={3} maxLength={8000} disabled={busy} value={goalDraft.goal.description} placeholder="Descreva o resultado que você quer construir." onChange={event=>setGoalDraft({...goalDraft,goal:{...goalDraft.goal,description:event.target.value}})}/><label htmlFor="goal-due">Prazo <span>(opcional)</span></label><DateInput id="goal-due" value={goalDraft.goal.due??''} disabled={busy} onChange={due=>setGoalDraft({...goalDraft,goal:{...goalDraft.goal,due:due||null}})}/><div className="goal-editor-actions"><button type="button" disabled={busy} onClick={()=>setGoalDraft(null)}>Cancelar</button><button className="botao-primario" disabled={busy||!goalDraft.goal.title.trim()}>{busy?'Salvando…':'Salvar meta'}</button></div></form></section>:goal?<><header className="goal-detail-heading"><div><span className="sobrelinha">{goalComplete(goal)?'META ALCANÇADA':'SUA DIREÇÃO'}</span><h3>{goal.title}</h3>{goal.due&&<p><CalendarDays size={14}/>{`Até ${brazilianDate(goal.due)}`}{!goalComplete(goal)&&goal.due<today()?' · Prazo passou':''}</p>}</div>{!compact&&<div><button disabled={busy||editing||loading} aria-label={`Editar meta ${goal.title}`} onClick={()=>editGoal(goal)}><Pencil size={16}/></button><button disabled={busy||editing||loading} aria-label={`Excluir meta ${goal.title}`} onClick={()=>setDeleting({owner:scope,goal})}><Trash2 size={16}/></button></div>}</header>{goal.description&&!compact&&<p className="goal-description">{goal.description}</p>}<Progress goal={goal}/><div className="goal-task-heading"><h4>{goalComplete(goal)?'Você chegou lá.':'Passos para chegar lá'}</h4>{!compact&&<button disabled={busy||editing||loading} onClick={()=>editTask(goal)}><Plus size={15}/> Adicionar tarefa</button>}</div>{!data.tasks.length&&!taskDraft&&<div className="goal-no-tasks"><p>Qual é o primeiro passo possível?</p>{compact?<button onClick={onOpen}>Adicionar no Lunes<ArrowUpRight size={14}/></button>:<button disabled={busy||loading} onClick={()=>editTask(goal)}>Adicionar primeira tarefa<Plus size={15}/></button>}</div>}<ul className="goal-tasks">{(compact?data.tasks.slice(0,5):data.tasks).map(taskRow)}</ul>{compact&&data.tasks.length>5&&<button className="goals-more" onClick={onOpen}>Ver as {data.tasks.length} tarefas no Lunes<ArrowUpRight size={14}/></button>}
                    {taskDraft?.owner===area&&<section ref={editor} className="goal-editor"><header><h4>{taskDraft.task.id?'Editar tarefa':'Um novo passo'}</h4><button disabled={busy} aria-label="Cancelar edição da tarefa" onClick={()=>setTaskDraft(null)}><X size={17}/></button></header><form onSubmit={event=>{event.preventDefault();void saveTask();}}><label htmlFor="goal-task-title">Tarefa</label><input id="goal-task-title" required maxLength={200} disabled={busy} value={taskDraft.task.title} placeholder="Ex.: Escolher três projetos para apresentar" onChange={event=>setTaskDraft({...taskDraft,task:{...taskDraft.task,title:event.target.value}})}/><div className="goal-task-fields"><div><label htmlFor="goal-task-due">Data</label><DateInput id="goal-task-due" required value={taskDraft.task.due} disabled={busy} onChange={due=>setTaskDraft({...taskDraft,task:{...taskDraft.task,due}})}/></div><div><label id="goal-task-priority-label" htmlFor="goal-task-priority">Prioridade</label><ModelPicker id="goal-task-priority" value={taskDraft.task.priority} options={priorities} disabled={busy} onChange={priority=>setTaskDraft({...taskDraft,task:{...taskDraft.task,priority}})}/></div></div><p className="nota">Esta tarefa também aparece em {'Trabalho'}, na lista de tarefas e no calendário da área.</p><div className="goal-editor-actions"><button type="button" disabled={busy} onClick={()=>setTaskDraft(null)}>Cancelar</button><button className="botao-primario" disabled={busy||!taskDraft.task.title.trim()}>{busy?'Salvando…':'Salvar tarefa'}</button></div></form></section>}</>:!loading&&<div className="goal-no-tasks"><Target size={28}/><p>Escolha uma meta para acompanhar seus próximos passos.</p></div>}
            </div>}
        </div>}
        {deleting&&<ConfirmDelete title={deleting.task?'Excluir tarefa?':'Excluir meta?'} description={deleting.task?`“${deleting.task.title}” será excluída da meta e de ${'Trabalho'}.`:`“${deleting.goal.title}” será excluída. Suas tarefas continuam em ${'Trabalho'}, sem vínculo com a meta.`} busy={busy} error={error} onCancel={()=>{setDeleting(null);setError('');}} onConfirm={()=>{const target=deleting;void commit(target.owner,()=>target.task?invoke('excluir_registro',{area:target.owner,kind:'task',id:target.task.id}):invoke('excluir_meta',{area:target.owner,id:target.goal.id,expectedRevision:target.goal.revision}),()=>{setDeleting(null);if(!target.task)setSelected(null);},target.task?'Tarefa excluída.':'Meta excluída. Suas tarefas foram preservadas.');}}/>}
    </section>;
}
