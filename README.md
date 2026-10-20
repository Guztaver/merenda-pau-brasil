# Merenda Pau Brasil

Aplicativo nativo para Windows, desenvolvido em Rust com SQLite embarcado e interface em português. Opera offline: não exige servidor, navegador ou conta para uso local.

Controle de alimentos por peso, movimentações retroativas, receitas por porção, consumo por escola, consulta de saldo por data, relatórios CSV e backup.

**Ferramenta auxiliar de estoque e consumo. Aderência integral ao PNAE/FNDE e a um edital específico não está demonstrada.** Consulte [AVALIACAO-FNDE.md](AVALIACAO-FNDE.md) para a revisão das fontes oficiais e as lacunas identificadas.

## Executar

Baixe o pacote Windows x64 em Releases, extraia e abra `MerendaPauBrasil.exe`. Os dados ficam em `%LOCALAPPDATA%\MerendaPauBrasil\merenda.sqlite`, fora da pasta do executável. Atualizar o executável não substitui o banco.

## Desenvolvimento

Instale Rust e as ferramentas C++ de compilação do Windows (MSVC). O arquivo `rust-toolchain.toml` seleciona o canal stable com rustfmt e Clippy.

```powershell
git switch dev
cargo run --locked
cargo fmt --all --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
cargo build --release --locked
```

Trabalhe normalmente na `dev`. Para publicar uma versão, abra um pull request da `dev` para a `main`, confira os testes e faça o merge. Não existe merge automático da `dev`.

## CI/CD

O workflow `.github/workflows/windows.yml` executa em pushes e pull requests de `dev` e `main`, além de execução manual:

1. Verifica formatação e análise de código.
2. Executa os testes e compila o Windows x64 com as dependências do Cargo.lock.
3. Gera um ZIP do executável e manuais, identificação do commit e arquivo SHA-256.
4. Mantém o pacote como artefato do Actions por 30 dias.
5. Após validação, pushes na `main` publicam automaticamente uma Release com tag exclusiva por execução. Execuções manuais na `main` também publicam. A `dev` gera artefatos, sem publicar Releases.

O deploy é a distribuição do pacote Windows via GitHub Releases. Não há servidor de produção nem instalação automática nos computadores da prefeitura. O aplicativo continua totalmente offline; apenas a compilação no GitHub usa rede.

O workflow usa o `GITHUB_TOKEN` automático, com escrita limitada ao job de Release; não exige cadastrar token pessoal em Secrets. A publicação depende de Actions habilitado e de disponibilidade/cota dos runners na conta. O executável não é assinado digitalmente.

## Documentação

- [Uso e backup](LEIA-ME.md)
- [Arquitetura do código](ARQUITETURA.md)
- [Avaliação FNDE e próximos passos](AVALIACAO-FNDE.md)

Nunca versione bancos operacionais, backups, dados pessoais ou credenciais. O `.gitignore` exclui esses arquivos usuais.
