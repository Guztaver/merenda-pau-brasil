# Merenda • Pau Brasil

Interface revisada com cartões e botões arredondados, formulários por seção, pesquisa de alimentos, painel de indicadores e prévia dos ingredientes antes da baixa. O código foi separado por responsabilidade; consulte `ARQUITETURA.md` para a estrutura dos arquivos. O banco da primeira versão é mantido no mesmo local e pode ser reutilizado diretamente.

Aplicativo de mesa para Windows, escrito em Rust, com interface egui e SQLite embarcado. Funciona sem navegador, servidor, conta ou conexão com a internet. A compilação do código-fonte requer baixar dependências; o executável pronto funciona offline.

## Iniciar

Abra `MerendaPauBrasil.exe`. O banco é criado automaticamente em `%LOCALAPPDATA%\MerendaPauBrasil\merenda.sqlite`, separado do executável. Os cadastros começam vazios, sem dados fictícios.

1. Cadastre as escolas.
2. Cadastre os alimentos e o estoque mínimo em gramas.
3. Lance as entradas, informando a data, peso e origem/documento.
4. Crie receitas e adicione os ingredientes em gramas por porção. Exemplo: 100 g de arroz cru por porção; 50 porções consomem 5 kg do estoque.
5. Registre as refeições servidas por escola e data. A baixa dos ingredientes é automática e ocorre em uma única transação.
6. Consulte o saldo por data e exporte os relatórios do período.

Datas usam `AAAA-MM-DD`. Os pesos informados em gramas são arredondados ao grama mais próximo, e os saldos são apresentados em kg com três casas decimais. Aceita vírgula decimal no peso. Um saldo igual ao mínimo é considerado suficiente.

## Histórico e períodos

Os lançamentos podem ser retroativos. O aplicativo recalcula os saldos diários e rejeita uma saída que deixe qualquer dia com saldo negativo, inclusive em dias posteriores. A ordem dentro do mesmo dia não é considerada; entradas e saídas desse dia são somadas. Alterar uma receita não altera os consumos já registrados. Datas futuras são permitidas, portanto confira a data antes de salvar.

O CSV informa saldo anterior ao início, entradas, saídas e saldo no fim, além das porções por escola/refeição. Início e fim são inclusivos. As saídas incluem consumo e perdas. O relatório é um apoio de conferência, não um formulário oficial de prestação de contas.

## Backup e restauração

Em Relatórios e segurança, clique em Criar backup do SQLite. A cópia é feita pela API de backup do SQLite. Use Abrir pasta dos dados e copie o arquivo de backup para um dispositivo seguro.

Para restaurar: feche completamente o aplicativo; guarde uma cópia do banco atual; copie o backup desejado para a pasta dos dados e renomeie a cópia para `merenda.sqlite`, substituindo o banco. Reabra o aplicativo e confira os saldos. A restauração substitui todos os registros atuais pelos registros da cópia escolhida.

## Escopo desta primeira versão

Estoque municipal centralizado, com consumo identificado por escola. Não há estoque separado por escola, controle por lote/validade, fornecedores estruturados, custos, compras/licitações, responsável por lançamento, autenticação, sincronização ou integração com sistemas federais. Não há edição/estorno de refeições já lançadas nesta versão: confira os dados antes de salvar. Os relatórios e os requisitos administrativos devem ser validados com a equipe da prefeitura antes do uso oficial. Receitas e quantidades devem ser definidas pelo profissional responsável.

Use em um computador por banco. Não compartilhe o SQLite por pasta de rede, OneDrive ou outro sincronizador. Os dados não são criptografados; proteja o acesso ao computador e às cópias de segurança.

## Código-fonte

Requer Rust e ferramentas C/C++ para Windows (MSVC). Na pasta do projeto:

```
cargo test --locked
cargo build --release --locked
```

O resultado fica em `target\release\merenda-pau-brasil.exe`. O arquivo Cargo.lock fixa as versões usadas. Não é necessário instalar SQLite.
