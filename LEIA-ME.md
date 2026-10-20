# Merenda • Pau Brasil

Aplicativo de mesa nativo para Windows, escrito em Rust, com interface egui e banco SQLite. Funciona offline, sem navegador, servidor, conta externa ou conexão com a internet. O executável pronto funciona offline; compilar o código-fonte requer baixar as dependências.

## Iniciar

Abra MerendaPauBrasil.exe. Na primeira execução, crie o usuário administrador. O banco fica em %LOCALAPPDATA%\MerendaPauBrasil\merenda.sqlite, separado do executável. O programa não inclui cadastros fictícios.

1. Cadastre as escolas e os alimentos ou materiais da rede.
2. Para cada item, informe a unidade de compra (g, kg, L, mL, cx ou un) e quantos gramas equivalem a uma unidade. Use o peso líquido informado na embalagem; para litros, confira a densidade do produto. O estoque é calculado internamente em gramas e exibido também na unidade escolhida.
3. Cadastre fornecedores com CPF/CNPJ e registre recebimentos com nota, processo, contrato, fonte de recurso, lote, validade e quantidade.
4. Registre perdas e transferências identificando o lote. Quantidades podem ser digitadas na unidade cadastrada do item.
5. Cadastre receitas e ingredientes em gramas por porção, aprove fichas técnicas e cardápios com o profissional responsável e registre as refeições servidas por escola.
6. Consulte o estoque por data, acompanhe competências, exporte relatórios e faça cópias de segurança.

Datas são digitadas no padrão DD/MM/AAAA; valores monetários e quantidades recebem formatação brasileira durante a digitação. A conversão e os dados nutricionais precisam ser conferidos por quem opera e pelo responsável técnico.

## Datas e histórico

Os lançamentos podem ser retroativos. O sistema recalcula os saldos diários e rejeita uma saída que deixe saldo negativo em qualquer data. Entradas e saídas do mesmo dia são somadas. Lançamentos operacionais não podem ser feitos em datas futuras. Alterar a receita não muda o consumo já registrado. Estornos preservam o registro original e exigem justificativa.

O CSV apresenta saldos e movimentos em kg, além das porções por escola/refeição. Os relatórios são apoio à conferência, não formulários oficiais de prestação de contas.

## Fornecedores e histórico de compras

A lista de fornecedores mostra nome, CPF/CNPJ, enquadramento familiar, CAF declarado e estado da conferência. Cadastros podem ser editados. A conferência da agricultura familiar é invalidada quando CPF/CNPJ, CAF ou enquadramento mudam. Compras antigas preservam uma fotografia do fornecedor registrada no recebimento. A exclusão é bloqueada quando há compras ou documentos vinculados.

## Backup e restauração

Em Relatórios e segurança, use a opção de criar backup SQLite e copie o arquivo para um dispositivo seguro. Para restaurar, feche o aplicativo, guarde o banco atual, substitua merenda.sqlite pelo backup escolhido e reabra o sistema. A restauração substitui os registros atuais pelos dados do backup.

Na primeira atualização de bancos antigos, o aplicativo cria uma cópia de segurança antes da migração. Não compartilhe o banco SQLite por pasta de rede, OneDrive ou outro sincronizador. Os dados não são criptografados; restrinja o acesso ao computador e às cópias.

## Uso institucional

O aplicativo é uma ferramenta de apoio à gestão municipal, não uma certificação de conformidade com o PNAE ou com um pregão específico. Não consulta serviços federais, não conduz licitação ou chamada pública e não substitui documentos originais, conferência contábil, controle interno ou atuação do nutricionista. Leia AVALIACAO-FNDE.md e valide fluxos e regras com as equipes responsáveis antes de uso oficial. O banco é local e deve ter uma rotina municipal de backup e controle de acesso.

## Código-fonte

Requer Rust stable e ferramentas C/C++ para Windows (MSVC). Na pasta do projeto, execute:

    cargo fmt --all --check
    cargo clippy --locked --all-targets -- -D warnings
    cargo test --locked
    cargo build --release --locked

O executável é criado em target\release\merenda-pau-brasil.exe. O Cargo.lock fixa as versões usadas; não é necessário instalar SQLite.
