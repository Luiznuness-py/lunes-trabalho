# Lunes Trabalho

Aplicativo desktop local para tarefas, anotações de reuniões e metas acompanhadas como entregas. Derivado do Lunes, com implementação própria e dados separados da versão pessoal.

- Tarefas com prioridade, prazo, conclusão e repetições diárias, semanais ou mensais.
- Reuniões com pauta, notas, decisões e ações convertidas em tarefas após conferência.
- Metas com tarefas vinculadas e progresso calculado pela conclusão dessas tarefas.
- Ideias em texto simples, temas claro/escuro e backup/restauração local.

Não usa IA, conta ChatGPT, MCP, GitHub/Gmail no aplicativo, nem a ilha Linux. Não requer login ou chave de API. O calendário da tela de tarefas serve para escolher datas; não há página de agenda de eventos nesta versão.

## Executar no Windows

Instale Git, Node.js 22.12 ou superior, Rust estável com toolchain MSVC e os pré-requisitos desktop do Tauri (Microsoft C++ Build Tools e WebView2). Consulte os [pré-requisitos oficiais do Tauri](https://v2.tauri.app/start/prerequisites/).

No PowerShell:

```powershell
git clone https://github.com/Luiznuness-py/lunes-trabalho.git
cd lunes-trabalho
npm ci
npm test
npm run build
cargo test --manifest-path src-tauri/Cargo.toml --locked
npm run tauri -- dev
```

Para gerar o instalador Windows:

```powershell
npm run tauri -- build --bundles nsis
```

O instalador fica em `src-tauri/target/release/bundle/nsis/`. A compilação precisa ser feita no Windows. O instalador não possui assinatura de código; distribuição assinada não faz parte desta base.

O workflow **Windows** também verifica o código e gera um instalador como artefato do GitHub Actions. Confira o resultado da execução antes de usar o pacote. Compilação automática não substitui os testes de uso na máquina de trabalho.

## Dados e backups

Banco `lunes-trabalho.sqlite3`, pasta `backups` e trava de instância ficam no diretório local de dados do aplicativo, fora deste repositório:

- Windows: `%LOCALAPPDATA%\local.lunes.trabalho`.
- Linux: `$XDG_DATA_HOME/local.lunes.trabalho` ou `~/.local/share/local.lunes.trabalho`.

Cada instalação tem seus próprios dados. Git transporta código; não sincroniza tarefas ou reuniões. A versão de trabalho não importa o banco ou os backups da versão pessoal.

Em **Configurações**, crie um backup e guarde uma cópia em outro local. Para restaurar uma cópia desta versão, coloque o arquivo na pasta `backups`, informe seu identificador e confirme. A restauração valida esquema, integridade e regras dos registros, e cria uma cópia do estado anterior antes de substituir dados em transação. Reabra o aplicativo após restaurar para recarregar também o tema.

O banco não é criptografado. A proteção da conta Windows e do disco continua sendo necessária. Não inclua conteúdo profissional, bancos, credenciais ou backups em commits.

## Desenvolvimento

React/TypeScript/Vite na interface; Tauri/Rust e SQLite nas regras e persistência. Não há servidor web externo. `npm run dev` abre somente a prévia visual; sem Tauri, os controles de gravação ficam desabilitados.

```sh
npm ci
npm test
npm run build
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo test --manifest-path src-tauri/Cargo.toml --locked
npm run tauri -- dev
```

Consulte [a arquitetura](docs/ARCHITECTURE.md), [a validação e pendências](docs/VALIDATION.md) e o [guia para implementar a ilha no Windows](docs/ISLAND_WINDOWS.md). O guia descreve o comportamento e as etapas; a ilha ainda não está implementada. Novas mudanças de banco exigem migração versionada; a versão inicial deste produto usa esquema próprio, versão 1.

## Licença

Projeto proprietário de Luiz Gustavo Nunes. Repositório público não altera as condições de uso em [LICENSE.md](LICENSE.md). Avisos dos ícones Lucide preservados em [docs/LUCIDE_LICENSE.txt](docs/LUCIDE_LICENSE.txt).
