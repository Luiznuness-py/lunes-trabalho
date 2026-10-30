import {useState} from 'react';
import {invoke,isTauri} from '@tauri-apps/api/core';
import {ConfirmDelete} from './confirm-delete';
export function Backups(){
 const [busy,setBusy]=useState(false),[error,setError]=useState(''),[notice,setNotice]=useState(''),[backup,setBackup]=useState(''),[confirm,setConfirm]=useState(false);
 async function run(command:string){
  setBusy(true);setError('');setNotice('');
  try{const result=await invoke<string|void>(command,command==='restaurar_dados'?{backup}:{});setNotice(command==='exportar_dados'?`Backup criado: ${result}`:'Dados restaurados. Reabra o aplicativo para carregar também o tema restaurado.');setConfirm(false);}
  catch(e){setError(typeof e==='string'?e:'Não foi possível concluir a operação.');}finally{setBusy(false);}
 }
 return <section className="organization painel record-editor"><h2>Backups locais</h2><p>As cópias ficam na pasta backups do diretório de dados do Lunes Trabalho. Guarde uma cópia dessa pasta em outro local para se proteger de falhas do computador.</p><p className="nota">Windows: %LOCALAPPDATA%\local.lunes.trabalho. Linux: diretório de dados do sistema, em local.lunes.trabalho.</p>
 {!isTauri()&&<p className="aviso">Backups estão disponíveis no aplicativo desktop.</p>}
 {error&&!confirm&&<p role="alert" className="aviso">{error}</p>}{notice&&<p role="status">{notice}</p>}
 <button disabled={!isTauri()||busy} onClick={()=>void run('exportar_dados')}>{busy?'Aguarde…':'Criar backup'}</button>
 <form onSubmit={e=>{e.preventDefault();setError('');setConfirm(true);}}><label htmlFor="backup-id">Identificador da cópia a restaurar</label><input id="backup-id" required maxLength={80} value={backup} disabled={busy} placeholder="backup-….sqlite3" onChange={e=>setBackup(e.target.value)}/><button disabled={!isTauri()||busy||!backup}>Restaurar backup</button></form>
 {confirm&&<ConfirmDelete title="Restaurar os dados?" description={`A cópia “${backup}” substituirá os registros atuais. Antes disso, o aplicativo criará um backup do estado atual. Cópias do Lunes pessoal são incompatíveis.`} confirmLabel="Restaurar" busyLabel="Restaurando…" busy={busy} error={error} onCancel={()=>setConfirm(false)} onConfirm={()=>void run('restaurar_dados')}/>}
 </section>;
}
