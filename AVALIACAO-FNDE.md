# Avaliação técnica de aderência ao PNAE/FNDE

Data da consulta: 20/10/2026. Versão analisada: código deste repositório.

## Resultado

**A versão atual é uma ferramenta auxiliar de estoque e consumo. Não está demonstrada a conformidade integral para gestão municipal do PNAE nem o atendimento a um pregão específico.** Nenhum edital ou termo de referência foi fornecido. A pesquisa nas fontes gerais do FNDE não identifica, por si só, qual contratação se aplica a Pau Brasil. Não há certificação ou homologação atribuída a este aplicativo.

As normas regulam a execução do programa. A aceitação de um software contratado depende também do edital, seus anexos, critérios de demonstração/aceite e validação pela prefeitura.

## Referências oficiais e implicações

- [Resolução CD/FNDE nº 4/2026](https://www.gov.br/fnde/pt-br/acesso-a-informacao/legislacao/resolucoes/2026/resolucao-cd_fnde-no-4-de-26-de-fevereiro-de-2026.pdf/@@download/file): arts. 15–18 tratam da responsabilidade técnica e do planejamento nutricional; art. 17, § 9º, de fichas técnicas; arts. 27–39, de aquisições; arts. 58–60, da prestação de contas. O art. 60 prevê BB Gestão Ágil e parecer do CAE no Sigecon. O aplicativo não substitui esses sistemas.
- [Orientações oficiais sobre agricultura familiar no PNAE](https://www.gov.br/fnde/pt-br/acesso-a-informacao/acoes-e-programas/programas/pnae/agricultura-familiar-no-pnae): a aquisição da agricultura familiar ocorre por chamada pública e o mínimo indicado para os recursos federais é de 45%. As orientações distinguem despesas com alimentos das despesas operacionais. Não é correto tratar todas as aquisições como um único pregão.
- [Legislação do FNDE em 2026](https://www.gov.br/fnde/pt-br/acesso-a-informacao/legislacao/resolucoes/2026): inclui as Resoluções 6/2026 e 11/2026, relativas a chamadas públicas em contextos específicos, e a 15/2026, complementar à prestação de contas. A análise de um caso concreto deve considerar os atos aplicáveis e suas atualizações.

## O que foi verificado no código

| Área | Situação atual | Lacuna observada |
|---|---|---|
| Estoque e consumo | Atende ao escopo básico | Estoque central, pesos em gramas, movimentos e saldos por data, baixas por porção e escola. Não há separação por almoxarifado/unidade. |
| Retroatividade | Implementada com proteção de saldo | Saída retroativa é rejeitada se gerar saldo diário negativo; não existe aprovação por responsável ou fechamento de competência. |
| Receitas | Parcial | Há ingredientes e peso por porção. Faltam valor nutricional, custo, modo de preparo, apresentação e ficha técnica completa. |
| Cardápios | Não implementado | Faltam calendário, etapa/faixa etária, dietas específicas, identificação/assinatura do nutricionista e documentos de divulgação. |
| Aquisições | Não implementado | Não há processos, editais, contratos, pesquisa de preços, fornecedores ou notas fiscais estruturadas. A observação em texto livre não equivale a esses registros. |
| Agricultura familiar | Não implementado | Não há identificação de origem, vínculo a CAF, compras e valores que permitam apurar o percentual da execução financeira. |
| Rastreabilidade e recebimento | Não implementado | Faltam lote, validade, fornecedor, recebimento/aceite e vínculo documental. |
| Auditoria | Insuficiente para responsabilização individual | Há data de criação, mas não usuário responsável, perfis de acesso, estorno justificado nem trilha protegida contra alteração direta do SQLite. |
| Relatórios | Auxiliares | CSV de saldo/entradas/saídas e refeições. Não há conciliação financeira ou prestação de contas oficial. |
| Backup | Implementado localmente | Existe cópia SQLite. A rotina de guarda externa e recuperação depende do operador. |

As lacunas acima são constatações do código e recomendações de evolução. Não significam que cada uma seja, isoladamente, uma obrigação legal de todo software de estoque: os critérios funcionais de uma contratação devem ser extraídos do respectivo termo de referência.

## Prioridades recomendadas

1. Obter o edital/termo de referência e construir uma matriz de requisito, evidência, teste de aceite e responsável.
2. Validar o fluxo com o nutricionista responsável técnico, setor de alimentação, compras, contabilidade e controle interno.
3. Implementar usuários, trilha de auditoria, estornos e fechamento/reabertura de períodos antes de usar retroatividade em produção oficial.
4. Ampliar recebimentos com fornecedores, documentos, lotes/validade e movimentação entre unidades.
5. Implementar fichas técnicas e cardápios com dados aprovados pelo nutricionista. Não calcular adequação nutricional sem tabelas e parâmetros validados.
6. Acrescentar valores, origem dos recursos e classificação das compras para relatórios financeiros e agricultura familiar.
7. Fazer um piloto com dados de teste e validar recuperação de backup e os relatórios de conferência.

Manter o aplicativo offline é compatível com seu papel de apoio local. A publicação e transmissão exigidas em processos externos continuam dependentes dos canais oficiais e dos procedimentos municipais. Esta revisão é técnica e não constitui parecer jurídico, nutricional ou aceite de contratação.
