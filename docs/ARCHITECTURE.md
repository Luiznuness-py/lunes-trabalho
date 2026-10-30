# Arquitetura implementada

Lunes Trabalho é uma derivação independente do Lunes, para uso local profissional. A interface mantém a identidade LN e os componentes de tarefas, reuniões e metas. O Lunes pessoal não é modificado por este repositório.

## Camadas

- `src/main.tsx`: janela, navegação entre Tarefas, Metas e entregas e Configurações; tema.
- `src/organization.tsx` e `src/agenda-panels.tsx`: tarefas, repetições, ideias, reuniões e conferência das ações propostas manualmente nas reuniões.
- `src/goals.tsx`: metas e suas tarefas. O progresso usa a mesma tarefa exibida na tela de tarefas.
- `src/backups.tsx`: criação de cópia local e confirmação de restauração.
- `src-tauri/src/main.rs`: comandos locais específicos e janela única. Não inicia serviços de IA, login ou conexões.
- `organizacao.rs`, `agenda.rs`, `metas.rs`: validações, revisões concorrentes, transações, recorrência, conversão de reunião e progresso.
- `dados.rs`: abertura do banco, trava de instância, esquema próprio, tema e backup/restauração.

## Isolamento

Identificador Tauri `local.lunes.trabalho`; banco `lunes-trabalho.sqlite3`; application_id SQLite `LNWT` (0x4c4e5754). A chave de tema da prévia é `lunes-trabalho.tema`. As operações e o banco aceitam apenas a área `trabalho`.

O esquema inicial é exclusivo desta aplicação. Não altera nem migra dados do Lunes pessoal; cópias desse produto são rejeitadas. Não há tabelas acadêmicas, conversas, modelos ou conexões. Os comandos IPC não expõem processos ou caminhos arbitrários.

## Persistência

SQLite com chaves estrangeiras, synchronous FULL e transações. O banco é bloqueado contra uso simultâneo por outra instância. No Unix, diretórios/arquivos privados usam 0700/0600. No Windows, usam a pasta local da conta e suas permissões herdadas; ACLs e uso em ambiente corporativo requerem conferência na máquina de destino.

Backup usa a API SQLite, sem copiar banco aberto de forma inconsistente. A restauração aceita apenas identificadores locais, valida identidade/versão, esquema exato (incluindo rejeição de triggers extras), integridade, relações e registros, cria backup preventivo e substitui dados em uma transação. A versão 1 não aceita outros esquemas. O diretório de dados não pode ser o checkout.

Excluir meta preserva suas tarefas. Excluir reunião preserva tarefas já convertidas. Recorrências mensais usam o último dia de meses curtos e retornam ao dia original depois. Uma ocorrência excluída manualmente não é recriada pela preparação da regra.

## Limites

Somente o calendário de seleção de datas das tarefas foi preservado; eventos, notificações de agenda e ilha estão fora do produto. A tela de reuniões continua dentro da página Tarefas. Não há sincronização entre computadores nem importação do banco pessoal. Banco e backups não são criptografados.

A configuração de instalação é NSIS para Windows. A validação nativa no Windows é registrada separadamente da compilação e da prévia no navegador.
