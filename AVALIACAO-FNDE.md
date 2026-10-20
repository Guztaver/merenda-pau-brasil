# Apoio de conformidade PNAE/FNDE — revisão de 20/10/2026

## Conclusão

Esta versão acrescenta controles operacionais e relatórios para apoiar a execução municipal do PNAE. Ela **não é certificada, homologada ou integrada pelo FNDE** e não comprova conformidade integral da prefeitura. Sem edital/termo de referência e sem análise dos processos, extratos, documentos e cardápios municipais, não é possível afirmar atendimento a pregão, contrato ou prestação de contas específica.

## Referência normativa consultada

- [Resolução CD/FNDE nº 4, de 26/02/2026 (PDF oficial)](https://www.gov.br/fnde/pt-br/acesso-a-informacao/legislacao/resolucoes/2026/resolucao-cd_fnde-no-4-de-26-de-fevereiro-de-2026.pdf/@@download/file), publicada no DOU em 02/03/2026. Ela substituiu integralmente a Resolução 6/2020. Os arts. 15–18 tratam de RT e planejamento nutricional; arts. 27–39, da aquisição; arts. 58–60, da prestação de contas. A seção V e os anexos orientam a chamada pública da agricultura familiar.
- [FAQ oficial do FNDE sobre agricultura familiar](https://www.gov.br/fnde/pt-br/acesso-a-informacao/acoes-e-programas/programas/pnae/agricultura-familiar-no-pnae): mínimo de 45% dos recursos financeiros federais do PNAE aplicados diretamente na compra de alimentos da agricultura familiar, conforme regras vigentes.
- [Índice oficial de resoluções do FNDE de 2026](https://www.gov.br/fnde/pt-br/acesso-a-informacao/legislacao/resolucoes/2026), a consultar antes de cada exercício e procedimento, inclusive para atos complementares e alterações posteriores.

## Controles que o aplicativo oferece

| Área | Controle | Limite |
|---|---|---|
| Planejamento | Fichas técnicas versionadas por receita, etapa/faixa etária e dieta; ingredientes em peso; rendimento, custo e nutrientes por porção; preparo e apresentação; aprovação associada a usuário nutricionista e CRN. Cardápios por escola, data, refeição, ficha e porções; revisões preservam versões anteriores. | Os dados e cálculos devem ser fornecidos e validados pelo RT. Não há motor validado que calcule adequação de nutrientes às metas de cada etapa/jornada, alergias ou restrições; campos de dieta são declaratórios. |
| Compra e recebimento | Fornecedor, CPF/CNPJ com dígito verificador, marcação de agricultura familiar, CAF declarado, processo, contrato, modalidade, nota, categoria do alimento, recurso, valor, peso, lote e validade. Pagamento separado do recebimento, com documento e estorno. | O sistema não consulta CAF, PNCP, Compras.gov.br, BB Gestão Ágil, SIGPC/SIGPC Ágil, bancos ou tribunais; não conduz licitação/chamada pública, pesquisa de preço, habilitação nem liquidação contábil. Conferência de fornecedor é declaração local referenciada e requer validação humana. |
| Agricultura familiar e limites | Indicador anual compara pagamentos registrados de agricultura familiar, categorias e pagamentos FNDE com repasses do exercício cadastrados. Saldo reprogramado aparece separado e não compõe esse denominador. CSV discrimina pagamentos e referências do fornecedor. | É somente uma conferência auxiliar. Deve ser conciliada com extratos, notas, estornos, fonte efetiva, documentos de habilitação, execução e regras do exercício. O cadastro não comprova CAF, chamada válida ou elegibilidade. |
| Estoque | Lotes, validade, locais central/escola, transferências, baixa por ficha aprovada e bloqueio de lotes vencidos, legado não conferido ou saldo insuficiente; preserva o histórico retroativo. | O banco legado é mantido e recebe lote “legado” sem validade; a prefeitura precisa conferir saldo físico, origem e validade antes de usá-lo. O aplicativo não é sistema de inspeção sanitária. |
| Governança | Contas locais com papéis, senha armazenada via Argon2, bloqueio de competência, estorno justificado, trilha encadeada por hash, hash de anexos, cópia e dossiê CSV/HTML local. Migração versionada gera cópia de segurança antes da atualização do banco existente. | Trilha/hash local não equivale a assinatura digital, carimbo de tempo confiável nem armazenamento imutável fora do computador. Administrador do Windows pode alterar os arquivos. O dossiê não é transmitido e não é formulário oficial. |

## Observações sobre indicadores

O relatório exibe as referências atuais de 45% para agricultura familiar, mínimo 85% para alimentos in natura/minimamente processados, máximo 10% para processados/ultraprocessados e máximo 5% para ingredientes culinários, conforme a regra anual aplicável. Percentuais do sistema usam pagamentos categorizados registrados divididos pelos repasses FNDE do exercício lançados no app; o saldo reprogramado é destacado separadamente. Esta simplificação operacional não implementa todas as regras de apuração, compensação, ressalvas, transições, devoluções, execução financeira e evidências oficiais. Indicador em verde não significa conformidade. Conferir a metodologia normativa do exercício com nutricionista, contabilidade e controle interno.

## Procedimentos necessários antes do uso oficial

1. O município deve designar formalmente RT(s), verificar CRN, responsabilidades, perfis de acesso, operadores, computadores e política de senha/backup.
2. O RT deve aprovar dados nutricionais e menus por faixa etária, etapa, tempo de permanência e necessidades específicas, publicar os cardápios pelos canais do município e manter os documentos oficiais.
3. Compras, chamada pública, pesquisa de preços, fornecedor, entrega, nota, pagamento, fonte de recursos, saldos e classificação de alimentos devem ser conferidos com documentos originais e a metodologia federal vigente.
4. Faça inventário físico inicial e reconcilie todos os cadastros/dados legados. Teste migração e restauração em cópia, mantenha backup externo controlado, registre responsável e preserve o banco em equipamento protegido.
5. Controle interno, contabilidade, jurídico, nutricionista, CAE e setor de compras devem revisar o fluxo, retenção, acesso e relatórios. Preencha a matriz de rastreabilidade do edital se houver contratação de software.
6. Para prestação de contas, use os sistemas e procedimentos oficiais atuais, inclusive BB Gestão Ágil/Sigecon ou suas transições oficiais. O aplicativo não envia ou substitui prestação oficial.

O armazenamento de nome de fornecedor, documento fiscal e eventos é local. A prefeitura deve aplicar seus controles de segurança, acesso, retenção e proteção de dados; não cadastre dados de estudantes ou informações de saúde individuais nesta versão.
