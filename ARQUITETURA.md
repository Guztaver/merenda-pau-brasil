# Organização do projeto

```
src/
  main.rs              Inicialização, pasta dos dados e janela nativa
  app.rs               Estado do aplicativo, navegação e formulários independentes
  validation.rs        Validação de datas e pesos e formatação em kg
  db/
    mod.rs             Conexão SQLite e transações de estoque/refeições
    schema.sql         Estrutura das tabelas e índices
    queries.rs         Cadastros, consultas de histórico, ingredientes e backup
    reports.rs         Consultas e geração do relatório CSV
    tests.rs           Testes de histórico, transações e compatibilidade
  ui/
    mod.rs             Declaração dos módulos de interface
    theme.rs           Paleta, fontes, espaçamentos e aparência dos controles
    components.rs      Cartões, campos, seletores, botões, indicadores e tabela
    shell.rs           Menu lateral, cabeçalho, notificações e área de conteúdo
    dashboard.rs       Indicadores e atalhos do painel
    foods.rs           Cadastro, busca e consulta do estoque por data
    movements.rs       Entradas, perdas e histórico de movimentações
    recipes.rs         Receitas e composição por porção
    servings.rs        Registro de refeições, prévia do consumo e histórico
    schools.rs         Unidades escolares
    reports.rs         Tela de exportação e backup
```

As telas acessam métodos de `Db`; nenhuma tela executa SQL nem acessa a conexão diretamente. As transações que dão baixa nos alimentos e a validação dos saldos históricos permanecem no banco. Os consumos passados armazenam o peso registrado, mesmo se a receita for alterada.

Cada formulário tem seu próprio estado. Editar a data de consulta do estoque não altera a data da movimentação ou do registro de refeições. Trocar de tela preserva os campos ainda não salvos; salvar limpa apenas os campos pertinentes àquela operação.

O tema é aplicado na inicialização. Ajustes de cores, raios e tipografia ficam em `ui/theme.rs`; componentes visuais comuns ficam em `ui/components.rs`. Cada tela tem seu próprio arquivo e reutiliza esses componentes.

O banco continua no mesmo caminho e com o mesmo esquema da primeira versão. Não é necessário importar ou recriar os dados para usar esta atualização.

## Verificação

`cargo test --locked` executa os testes de saldo retroativo, gravação atômica, validação, preservação do consumo após alterações de receita e reabertura do esquema existente.

`cargo clippy --locked -- -D warnings` verifica o código. `cargo fmt --check` verifica a formatação.

A interface foi conferida no Windows com banco de teste separado, sem alterar os dados operacionais.
