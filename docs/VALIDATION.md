# Validação da base — 29/10/2026

## Executado no Linux

- Instalação limpa com `npm ci`: passou, com diretório node_modules próprio desta cópia.
- `npm test`: passaram os cinco arquivos de teste de contraste, janela, tema, datas/calendário e metas.
- `npm run build`: tipos TypeScript e bundle Vite passaram.
- `cargo fmt --manifest-path src-tauri/Cargo.toml --check`: passou.
- `cargo test --manifest-path src-tauri/Cargo.toml --locked --offline -j 2`: seis testes passaram.
- `git diff --cached --check`: passou antes do envio.

Os testes Rust usam SQLite real em pastas temporárias, não os dados pessoais: rejeição de áreas Pessoal/Estudos, trava de instância, tema e registros após reabrir, edição desatualizada, datas inválidas, repetição mensal em fevereiro/meses curtos, exclusão manual preservada, conversão explícita e idempotente de ações de reunião, preservação de tarefas ao excluir reunião/meta, progresso e restauração de backup. Cópia com identidade pessoal ou trigger adicional é rejeitada sem perda do estado atual.

## Conferência no navegador

Conferidos temas claro/escuro, largura de 480 px, navegação, formulários de tarefa/reunião/meta, conclusão de tarefa, foco no cancelamento de exclusão, Escape e preservação de formulário após falha simulada. Nenhum erro JavaScript foi observado na conferência final da página principal. O formulário de backup usa os mesmos estilos dos demais editores.

As gravações nessa conferência usaram IPC simulado com dados sintéticos. Não comprovam integração da WebView com Rust. A página principal sem Tauri deixa gravações desabilitadas. Para reproduzir a prévia simulada, execute `npm run dev` e abra `/tests/ui.html`; `?fail` simula falha de gravação. Essa página não é incluída na entrada de produção do Vite.

## Pendências reais

- O workflow Windows verifica testes/build e gera NSIS; consulte o resultado no GitHub Actions. A configuração do workflow não é evidência de execução bem-sucedida.
- Validar na máquina Windows: instalação, abertura, controles de janela, criação/edição/exclusão, tarefas recorrentes, anotações de reunião, metas, reinício, backups e permissões da pasta de dados.
- Não houve execução nativa desta versão na WebView Linux nem no Windows nesta etapa.
- Instalador sem assinatura de código; não há atualização automática, sincronização entre computadores ou migração de dados pessoais.
- Mudanças futuras no esquema precisam de migrações e validação compatível de backups; a base inicial aceita somente seu próprio esquema v1.
