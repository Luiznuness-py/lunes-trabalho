# Ilha do Lunes Trabalho no Windows

> Especificação para implementação. Este documento não significa que a ilha já existe ou foi validada no Windows.

## Objetivo e escopo

Reproduzir a identidade e a interação da ilha do Lunes no Linux: um painel compacto no topo, revelado pelo mouse, expansível, com acesso rápido aos registros do aplicativo. A implementação deve ser própria, usando React/TypeScript, Tauri/Rust e integração nativa do Windows quando necessária.

O destino é o **Lunes Trabalho**, neste repositório. Sua ilha terá um único espaço profissional, com tarefas, ideias, reuniões e metas/entregas. Não recriar os ambientes Pessoal/Trabalho/Estudos. Não adicionar IA, ChatGPT, MCP, Gmail ou GitHub para obter paridade visual com o Linux. Esses serviços ficaram fora desta versão por decisão do usuário.

O pedido atual é publicar este guia; a implementação e sua validação nativa serão feitas em uma etapa posterior no Windows.

## Base existente e o que falta

O aplicativo já possui componentes de tarefas, reuniões e metas, comandos Rust, SQLite, temas e o plugin de instância única. A configuração Tauri contém somente a janela `main`. Não há janela da ilha, controlador nativo Windows, bandeja ou atalho global implementados nesta base.

A ilha Linux foi inspecionada no projeto original: frontend `src/island.tsx`, estilos `src/island.css`, estado `src/island-state.ts`, controlador `src-tauri/src/ilha.rs` e extensão `linux/gnome/lunes-island@local/extension.js`. Esses caminhos são referências do projeto original, não arquivos já presentes neste repositório.

A extensão GNOME usa APIs do compositor e D-Bus. Não transportar esse código para o Windows. Reproduzir seu contrato de interação, adaptando a integração ao Windows. Não copiar código, design ou assets protegidos de Niko ou outros produtos; preservar a identidade LN e os recursos próprios do Lunes.

## Aparência e estados

Usar a logo LN, os ícones Lucide e os tokens dos temas já existentes. Não criar outro tema, banco ou aplicativo auxiliar para a ilha.

| Estado | Apresentação e comportamento |
| --- | --- |
| Oculta | A janela da ilha fica invisível. A zona de revelação pode permanecer ativa enquanto a sessão permitir. |
| Compacta | Faixa de referência de 280 × 48 unidades lógicas, centralizada no topo do monitor escolhido. Clique expande. |
| Expandida | Painel de referência de 680 × 400 unidades lógicas, com abas e rolagem interna. Limitar ao espaço disponível. |
| Pausada | Janela e revelação por hover desativadas até o usuário reativar. Não interrompe tarefas ou a janela principal. |

No Linux atual, o controlador usa mínimo de 240 × 240 no expandido; a faixa compacta mantém altura de 48. Esses valores são referências iniciais. Conferir o tamanho mínimo real no Windows antes de fixá-lo, preservando formulários e botões acessíveis.

O tema da ilha acompanha o tema salvo no backend. Ao mudar o tema em qualquer janela, emitir/receber o mesmo evento local e atualizar a outra. Não criar uma preferência de tema independente.

## Revelação, foco e ocultação

1. Criar uma zona discreta no centro superior de cada monitor. A referência Linux é uma faixa de 240 × 8 unidades lógicas, alinhada ao topo da área útil.
2. Ao entrar nessa zona, revelar a ilha no monitor onde o ponteiro está, sem ativar a janela nem tirar o foco do aplicativo em uso. Na referência Linux, o hover solicita a apresentação expandida sem foco; o modo compacto também pode ser aberto explicitamente.
3. Clique explícito para expandir/usar um formulário pode habilitar foco e digitação. A ilha compacta não deve capturar teclado.
4. Quando o ponteiro sair da ilha e da margem de tolerância, aguardar aproximadamente 800 ms antes de ocultar, desde que a ilha não esteja em uso.
5. Não ocultar automaticamente um painel expandido com foco, editor ativo, menu aberto ou confirmação modal. Preservar texto após erros de salvamento e atualizações do resumo.
6. Escape primeiro fecha o menu ou cancela a confirmação. Sem elementos internos abertos, recolhe a ilha expandida; na compacta, oculta. A tecla não confirma exclusão ou gravação.
7. Quando a sessão estiver bloqueada ou houver fullscreen no monitor de destino, ocultar também a zona de revelação. Desbloquear não deve abrir a ilha com foco automaticamente.

A zona não pode interceptar uma faixa inteira da tela, cobrir botões de outras janelas ou impedir o uso da barra de tarefas com ocultação automática. Ela serve apenas para revelar o painel, não para executar operações de negócio.

Não usar clique através de toda a janela da ilha: isso impediria interação com seus controles. Se houver janela auxiliar para a zona de hover, ela não contém WebView, dados, formulários ou lógica de gravação; o controlador Rust é responsável por seu ciclo de vida.

## Bandeja, atalho e ciclo de vida

Usar o recurso de [bandeja do Tauri](https://v2.tauri.app/learn/system-tray/) como equivalente funcional ao menu do painel GNOME. Habilitar a feature correspondente somente ao implementar essa etapa.

Menu esperado:

- Abrir Lunes Trabalho.
- Mostrar/ocultar ilha.
- Nova tarefa.
- Configurações.
- Pausar/ativar ilha.
- Encerrar Lunes Trabalho.

O [plugin oficial de atalho global](https://v2.tauri.app/plugin/global-shortcut/) é uma opção para alternar a ilha. A referência Linux é `Ctrl+Alt+L`; validar conflitos no Windows. Se o registro falhar, avisar na interface e manter a bandeja como acesso. Não capturar teclas de digitação nem implementar um keylogger.

Fechar a janela principal pode apenas escondê-la **quando** a ilha estiver habilitada e a bandeja tiver sido criada com sucesso. Se não houver uma forma de recuperar o aplicativo, manter o fechamento normal. A ação Encerrar fecha todas as janelas, remove callbacks/atalhos e encerra o processo.

Reutilizar `tauri-plugin-single-instance`: abrir o aplicativo de novo recupera a janela existente, sem abrir outro dono do banco. O callback atual sempre apresenta `main`; adaptá-lo somente se forem necessárias ações de lançamento fixas. Não adicionar argumentos de shell, caminhos arbitrários ou um servidor HTTP para controlar a ilha.

Iniciar com o Windows é uma melhoria separada e deve ser opt-in. Não modificar o Registro, tarefas agendadas ou inicialização do sistema como efeito colateral de habilitar a ilha.

## Posicionamento e integração nativa

Tentar primeiro as APIs Tauri disponíveis na versão instalada. Se a janela não puder ser apresentada sem ativação, completar somente essa parte com Win32 no backend, sob `#[cfg(target_os = "windows")]`.

[SetWindowPos](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwindowpos) oferece controle de posição e ordem das janelas, incluindo a opção `SWP_NOACTIVATE`. Essa flag é uma ferramenta de implementação; sua presença no código não comprova ausência de roubo de foco na WebView. Testar hover com outro aplicativo recebendo digitação. Não tentar corrigir foco ativando repetidamente a janela anterior.

Escolher o monitor a partir do ponteiro ao revelar; usar as informações de monitor/área útil obtidas por [GetMonitorInfoW](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getmonitorinfow) ou equivalente Tauri. Não assumir monitor primário, coordenadas positivas ou origem `(0, 0)`.

Referência de geometria Linux, expressa em coordenadas lógicas:

```text
largura = min(largura desejada, largura útil - 24)
altura  = min(altura desejada, altura útil - 16)
x       = área útil.x + (largura útil - largura) / 2
y       = área útil.y + 6
```

No Windows, tratar monitores pequenos e áreas úteis menores que essas margens sem produzir dimensões negativas ou deixar botões fora da tela. O limite superior é o espaço realmente disponível, mesmo quando menor que um mínimo preferido.

Não misturar coordenadas físicas Win32 com unidades lógicas Tauri. Respeitar a consciência de DPI já configurada pelo runtime e tratar mudanças de escala, inclusive [WM_DPICHANGED](https://learn.microsoft.com/en-us/windows/win32/hidpi/wm-dpichanged). Não alterar a política de DPI do processo depois de as janelas terem sido criadas.

Recalcular posição ao conectar/desconectar monitor, mudar resolução/escala ou alterar a área útil da barra de tarefas. Se o monitor escolhido desaparecer, recolocar a ilha num monitor disponível. Não reservar espaço de desktop como AppBar: a referência Linux é uma sobreposição, não um novo painel permanente.

Para bloqueio/desbloqueio, avaliar [WTSRegisterSessionNotification](https://learn.microsoft.com/en-us/windows/win32/api/wtsapi32/nf-wtsapi32-wtsregistersessionnotification) e as notificações de sessão correspondentes. Para fullscreen, verificar a janela em primeiro plano e sua ocupação do monitor; decidir e testar também o comportamento em fullscreen sem bordas. Não tratar toda janela maximizada como fullscreen. O algoritmo deve ser validado com aplicativos reais, sem prometer cobertura universal por uma única API.

Conferir também a troca de desktops virtuais. Não presumir que uma janela topmost acompanha todos eles. Se não houver uma solução documentada e validada, registrar esse limite e manter acesso por bandeja/atalho; não depender de interfaces privadas do Explorer.

Não tentar desenhar sobre a tela de bloqueio, desktop seguro ou prompts do UAC. Não exigir execução como administrador. Se a integração nativa falhar, informar a indisponibilidade e manter o acesso pela janela principal e pela bandeja.

## Conteúdo da ilha profissional

| Aba | Escopo e reaproveitamento |
| --- | --- |
| Hoje | Até cinco tarefas pendentes, priorizadas pelas regras existentes, e resumo do próximo prazo/reunião. |
| Nova tarefa | Título, data e prioridade pelo comando `salvar_tarefa`. Sem uma segunda lista de tarefas. |
| Captura | Salva ideia em texto simples pelo comando `salvar_nota`, sem IA. |
| Metas | Reutiliza `Goals` e seus comandos; conclusão atualiza a mesma tarefa usada na janela principal. |
| Reuniões/prazos | Exibe somente registros já disponíveis. Abrir navega para a reunião/tarefa real. |

Manter abas simples e rótulos em pt-BR. A configuração inicial não precisa de uma aba para cada tela do aplicativo. O projeto não tem agenda completa de eventos ou avisos: não apresentar esse recurso como existente para preencher uma aba Agenda.

A aba Mídia do Lunes Linux depende de MPRIS/Gio. Não compila como uma solução Windows. Mídia não é requisito da primeira ilha profissional; se solicitada depois, exige um adaptador Windows próprio e validação dos players. Não trazer Gio, D-Bus ou comandos Linux para implementá-la.

IA, Gmail/GitHub e materiais acadêmicos não fazem parte desta ilha. Esse recorte mantém o que foi pedido para o aplicativo de trabalho; a paridade buscada aqui é visual e de interação nativa.

## Estado, comandos e dados

Uma janela `island` deve usar o mesmo processo Tauri, o mesmo `Storage` e o mesmo SQLite de `main`. A rota pode ser `index.html?ilha=1`. Desviar essa entrada antes de renderizar o layout principal, evitando outra barra lateral dentro da ilha.

Ao configurar/criar a janela, começar invisível, sem decoração, fora da barra de tarefas, sem foco inicial e com dimensões compactas. Fundo transparente e cantos arredondados dependem da conferência no WebView2; não confundir transparência com clique através. Elevar a janela sobre janelas comuns apenas quando visível, mantendo as regras de fullscreen e bloqueio.

Contrato sugerido, equivalente ao controlador Linux:

```text
controlar_ilha(action) -> { visible, expanded, enabled, revision }
ler_resumo_ilha()      -> { tasks, upcoming }
abrir_pela_ilha(page)  -> confirmação de navegação permitida
encerrar_lunes()      -> encerra o processo
```

Ações de estado: `estado`, `mostrar`, `expandir`, `recolher`, `ocultar`, `alternar`, `pausar`, `ativar`. Não receber área: esta versão só tem um espaço. Destinos de navegação devem ser uma lista fixa de páginas válidas, nunca URLs ou comandos livres.

Eventos locais: estado atualizado da ilha, alteração de tema e atualização de metas. Usar revisão monotônica para ignorar respostas atrasadas. Coletar o resumo no backend, com projeção limitada: título, data e prioridade, sem transportar corpos de notas ou banco inteiro.

Atualizar o resumo ao revelar e, enquanto visível, a cada 15 segundos como referência inicial. Suspender a consulta periódica quando oculta/pausada. Para hover, preferir notificações nativas; se precisar consultar o ponteiro, usar uma cadência limitada e cancelar o temporizador ao pausar/encerrar, evitando loop ocupado.

Cada formulário mantém seu rascunho em caso de falha. Bloquear envios simultâneos enquanto salva e conservar o token da tentativa, respeitando a idempotência existente. Uma gravação bem-sucedida seguida de falha de atualização da lista não pode provocar nova criação automática.

Regras, validação, revisão concorrente, exclusões e transações continuam no Rust. Criar/excluir pela ilha deve atualizar os mesmos registros; não salvar um espelho no localStorage ou abrir outra conexão independente ao banco.

O banco continua em `%LOCALAPPDATA%\local.lunes.trabalho`, fora do repositório. A ilha não precisa de migração de esquema para existir. Não adicionar tabelas vazias ou copiar dados do Lunes pessoal.

## Arquivos que a implementação deverá alcançar

| Arquivo | Alteração esperada |
| --- | --- |
| `src/main.tsx` | Escolher a entrada principal ou a ilha, sincronizar tema e receber navegação. |
| `src/island.tsx` e `src/island.css` | Criar o painel e seus estados usando os componentes/tokens existentes. |
| `src/island-state.ts` | Compartilhar o contrato de estado/eventos se ambas as janelas o consumirem. |
| `src/goals.tsx` | Adaptar/reutilizar apresentação compacta, sem duplicar regras de metas. |
| `src-tauri/src/ilha.rs` | Controlar estados, resumo e destinos permitidos. Criar quando começar a implementação. |
| Adaptador Windows | Manter detalhes Win32 no mesmo módulo ou separar somente se houver complexidade real. |
| `src-tauri/src/main.rs` | Registrar estado, comandos, bandeja, fechamento e integração de instância única. |
| `src-tauri/tauri.conf.json` | Declarar `island` ou sua criação controlada no backend, sem duplicar janelas. |
| `src-tauri/capabilities/main-window.json` | Revisar as permissões de `main`; preferir capacidade separada e mínima para `island`. |
| `src-tauri/Cargo.toml` e lock | Adicionar somente os recursos efetivamente usados; dependência Win32 sob target Windows. |
| `tests/` | Regressões do controlador/formulários e uma prévia sintética fora da entrada de produção. |
| `docs/VALIDATION.md` | Registrar evidências e limitações reais da implementação Windows. |

Preservar o instalador NSIS e o workflow Windows existentes. Não adicionar Python, Electron, serviço elevado ou extensão de shell apenas para criar a ilha.

## Ordem de execução no Windows

Registrar primeiro a versão do Windows, arquitetura, versão do WebView2 e monitores/escalas da máquina de destino. Não declarar suporte a versões ou arquiteturas que não foram testadas.

1. **Base nativa:** criar janela oculta e bandeja, testar abrir/ocultar/encerrar e instância única. Só então mudar o comportamento do botão fechar.
2. **Interface:** compacto/expandido, identidade, temas, foco e Escape. Usar dados sintéticos apenas na prévia de testes.
3. **Dados reais locais:** conectar tarefas, captura e metas aos comandos existentes, incluindo falha/repetição e atualização entre janelas.
4. **Integração de desktop:** implementar hover, monitor do ponteiro, posicionamento, DPI, atalho e bloqueio/fullscreen.
5. **Pacote:** compilar NSIS e instalar em uma conta Windows limpa. Validar processo de encerramento e acesso pela bandeja após fechar `main`.

Concluir e registrar cada etapa antes de avançar. Uma janela com `alwaysOnTop` não comprova paridade com a ilha Linux.

## Critérios de aceitação

- [ ] Claro/escuro, texto legível, teclado e foco visível; menus e confirmações cabem na janela.
- [ ] Hover revela sem interromper a digitação em outro aplicativo. Clique em formulário permite digitar.
- [ ] Compactar, expandir, ocultar, pausar e reativar têm comportamento consistente.
- [ ] Ocultação após saída do ponteiro não descarta formulário, menu ou confirmação em uso.
- [ ] Escape cancela controles internos antes de recolher a ilha.
- [ ] Monitor principal/secundário, coordenadas negativas e escalas 100%, 125%, 150% e 200%.
- [ ] Troca de escala, reconexão de monitor, suspensão/retomada e barra de tarefas com ocultação automática.
- [ ] Troca de desktops virtuais, registrando o comportamento observado e qualquer limitação.
- [ ] Fullscreen e fullscreen sem bordas escondem a ilha no monitor afetado; maximização normal não é confundida.
- [ ] Bloqueio/desbloqueio não expõe registros nem rouba foco ao voltar.
- [ ] Atalho ocupado informa falha; bandeja continua disponível.
- [ ] Fechar a janela principal mantém acesso recuperável; Encerrar libera processo, recursos e trava do banco.
- [ ] Abrir o app duas vezes recupera a instância existente, sem duplicar bandeja, ilha ou registros.
- [ ] Tarefa, ideia e meta criadas/alteradas na ilha aparecem na janela principal e persistem após reabrir.
- [ ] Clique repetido, falha de disco, erro de leitura após salvar e exclusão cancelada não causam duplicação/perda.
- [ ] Permissões da WebView não expõem shell, processos ou caminhos livres.
- [ ] Build/testes do workflow e instalador NSIS passam; comportamento nativo é conferido na máquina de destino.

Testes de geometria/estado com monitores simulados e IPC simulado ajudam na regressão. Eles não comprovam foco, hover, WebView2, DPI, bandeja ou fullscreen no Windows real.

## Referências

Os valores de interação Linux foram conferidos no código próprio do Lunes em 29/10/2026. O recorte profissional segue o README e a arquitetura deste repositório.

- [Arquitetura atual do Lunes Trabalho](ARCHITECTURE.md).
- [Evidências e pendências atuais](VALIDATION.md).
- [Bandeja Tauri](https://v2.tauri.app/learn/system-tray/) e [atalho global](https://v2.tauri.app/plugin/global-shortcut/).
- [SetWindowPos](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwindowpos), [GetMonitorInfoW](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getmonitorinfow) e [WM_DPICHANGED](https://learn.microsoft.com/en-us/windows/win32/hidpi/wm-dpichanged).
- [Notificações de sessão Windows](https://learn.microsoft.com/en-us/windows/win32/api/wtsapi32/nf-wtsapi32-wtsregistersessionnotification).
