import { Component, Suspense, StrictMode, type ReactNode, useEffect, useRef, useState } from 'react';
import { createRoot } from 'react-dom/client';
import { Building2, Settings, Target, PanelLeftOpen, PanelLeftClose, Sun, Moon, Minus, Square, Copy, X } from 'lucide-react';
import { isTauri } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { controlarJanela } from './window';
import { carregarTema, salvarTema, temaLegado, type Tema } from './theme';
import { Organization } from './organization';
import { Goals } from './goals';
import { Backups } from './backups';
import './styles.css';
class AreaBoundary extends Component<{children:ReactNode},{failed:boolean}> {
  state = {failed:false};
  static getDerivedStateFromError() { return {failed:true}; }
  render() {
    return this.state.failed ? <section className="painel" role="alert"><h2>Não foi possível abrir esta área</h2><p>Você pode acessar outra área ou reabrir o aplicativo para tentar novamente.</p></section> : this.props.children;
  }
}

const desktop = isTauri();

const areas = [
  ['trabalho', 'Tarefas', 'Tarefas e anotações de reuniões.', Building2],
  ['metas', 'Metas e entregas', 'Resultados e próximos passos.', Target],
  ['configuracoes', 'Configurações', 'Preferências e backups locais.', Settings],
] as const;
type Area = typeof areas[number][0];
function lerTema(): Tema {
  try { return temaLegado() ?? 'claro'; } catch { return 'claro'; }
}

function App() {
  const [area, setArea] = useState<Area>(() => areas.find(item => `#${item[0]}` === window.location.hash)?.[0] ?? 'trabalho');
  const [tema, setTema] = useState<Tema>(lerTema);
  const [recolhida, setRecolhida] = useState(() => window.innerWidth <= 900);
  const [erroTema, setErroTema] = useState('');
  const [temaPronto, setTemaPronto] = useState(!desktop);
  const filaTema = useRef(Promise.resolve());
  const [maximizada, setMaximizada] = useState(false);
  const [erroJanela, setErroJanela] = useState(false);
  const titulo = useRef<HTMLHeadingElement>(null);
  const focarArea = useRef(false);
  const atual = areas.find(item => item[0] === area)!;
  const IconeAtual = atual[3];

  async function acaoJanela(acao: Parameters<typeof controlarJanela>[0]) {
    try {
      const estado = await controlarJanela(acao);
      if (typeof estado === "boolean") setMaximizada(estado);
      setErroJanela(false);
    } catch { setErroJanela(true); }
  }

  useEffect(() => {
    if (!desktop) return;
    let ativo = true;
    let remover: (() => void) | undefined;
    const janela = getCurrentWindow();
    const atualizar = async () => {
      try {
        const estado = await janela.isMaximized();
        if (ativo) setMaximizada(estado);
      } catch { if (ativo) setErroJanela(true); }
    };
    void atualizar();
    void janela.onResized(() => { void atualizar(); }).then(fn => {
      if (ativo) remover = fn; else fn();
    }).catch(() => { if (ativo) setErroJanela(true); });
    return () => { ativo = false; remover?.(); };
  }, []);

  useEffect(() => {
    let ativo = true;
    void carregarTema().then(salvo => {
      if (ativo) { setTema(salvo); setTemaPronto(true); setErroTema(''); }
    }).catch(erro => { if (ativo) setErroTema(typeof erro === 'string' ? erro : 'Não foi possível recuperar o tema salvo.'); });
    return () => { ativo = false; };
  }, []);
  useEffect(() => { document.documentElement.dataset.tema = tema; }, [tema]);
  function mudarTema(novo: Tema) {
    setTema(novo);
    filaTema.current = filaTema.current.catch(() => {}).then(() => salvarTema(novo)).then(() => setErroTema('')).catch(erro => {
      setErroTema(typeof erro === 'string' ? erro : 'O tema foi aplicado, mas não foi possível salvar a preferência.');
    });
  }
  useEffect(() => { document.title = `Lunes Trabalho - ${atual[1]}`; }, [atual]);

  useEffect(() => {
    const mudarArea = () => {
      if (window.location.hash === '#conteudo') return;
      focarArea.current = true;
      setArea(areas.find(item => `#${item[0]}` === window.location.hash)?.[0] ?? 'trabalho');
    };
    window.addEventListener('hashchange', mudarArea);
    return () => window.removeEventListener('hashchange', mudarArea);
  }, []);
  useEffect(() => {
    if (focarArea.current) {
      titulo.current?.focus();
      focarArea.current = false;
    }
  }, [area]);
  const dashboard = area === 'trabalho';

  return <div className={`app ${recolhida ? 'recolhida' : ''} ${desktop ? 'desktop' : ''}`}>
    <a className="pular" href="#conteudo" onClick={e => { e.preventDefault(); focarArea.current = false; document.getElementById('conteudo')?.focus(); }}>Pular para o conteúdo</a>
      <header className="cabecalho"
        onMouseDown={e => { if (desktop && e.button === 0 && e.detail === 1 && !(e.target as HTMLElement).closest('button')) void acaoJanela('startDragging'); }}
        onDoubleClick={e => { if (desktop && !(e.target as HTMLElement).closest('button')) void acaoJanela('toggleMaximize'); }}><span className="titulo-janela"><IconeAtual size={14} aria-hidden="true" />{atual[1]}</span><span className="status">{desktop ? 'Local' : 'Prévia'}</span>{desktop && <div className="controles-janela" aria-label="Controles da janela">
        <button aria-label="Minimizar janela" title="Minimizar" onClick={() => void acaoJanela('minimize')}><Minus size={16} aria-hidden="true" /></button>
        <button aria-label={maximizada ? 'Restaurar janela' : 'Maximizar janela'} title={maximizada ? 'Restaurar' : 'Maximizar'} onClick={() => void acaoJanela('toggleMaximize')}>{maximizada ? <Copy size={15} aria-hidden="true" /> : <Square size={15} aria-hidden="true" />}</button>
        <button aria-label="Fechar janela" title="Fechar" onClick={() => void acaoJanela('close')}><X size={18} aria-hidden="true" /></button>
      </div>}</header>
    <aside className="lateral">
      <div className="marca"><img src="/logo-transparente.png" alt="Logo Lunes" width="24" height="24" /><span translate="no">Lunes Trabalho</span>
        <button className="recolher" title={recolhida ? 'Expandir menu' : 'Recolher menu'} aria-label={recolhida ? 'Expandir barra lateral' : 'Recolher barra lateral'} aria-expanded={!recolhida} onClick={() => setRecolhida(!recolhida)}>{recolhida ? <PanelLeftOpen size={14} aria-hidden="true" /> : <PanelLeftClose size={14} aria-hidden="true" />}</button>
      </div>
      <nav aria-label="Áreas do Lunes Trabalho"><div className="grupo-nav" role="group" aria-label="Organização">{areas.map(([id,nome,,Icone])=><a key={id} href={`#${id}`} title={nome} aria-label={nome} aria-current={area===id?'page':undefined}><span className="icone" aria-hidden="true"><Icone size={18}/></span><span className="rotulo">{nome}</span></a>)}</div></nav>
      <div className="lateral-base"><button className="tema" title={`Usar tema ${tema === 'claro' ? 'escuro' : 'claro'}`} aria-label={`Usar tema ${tema === 'claro' ? 'escuro' : 'claro'}`} disabled={!temaPronto} onClick={() => mudarTema(tema === 'claro' ? 'escuro' : 'claro')}>{tema === 'claro' ? <Moon size={17} aria-hidden="true" /> : <Sun size={17} aria-hidden="true" />}</button></div>
    </aside>
    <main id="conteudo" tabIndex={-1}>

      <div className={`pagina ${dashboard ? 'pagina-agenda' : ''}`}><h1 ref={titulo} tabIndex={-1}>{atual[1]}</h1>{!dashboard && <p className="subtitulo">{atual[2]}</p>}
      {erroJanela && <p role="alert" className="aviso">Não foi possível controlar a janela. Tente novamente.</p>}
      {erroTema && <p role="status" className="aviso">{erroTema}</p>}
      <AreaBoundary key={area}><Suspense fallback={<p role="status" className="nota">Carregando área…</p>}>
      {area === 'trabalho' ? <Organization area="trabalho"/> : area === 'metas' ? <Goals area="trabalho"/> : <Backups/>}
      </Suspense></AreaBoundary>
      </div>
    </main>
  </div>;
}

createRoot(document.getElementById('root')!).render(<StrictMode><App/></StrictMode>);
