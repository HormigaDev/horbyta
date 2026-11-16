#!/usr/bin/env python3
"""Gera os diagramas do Relatório de TCC II (Casos de Uso, Classes e ER).

Conforme orientação, os diagramas contemplam somente a regra de negócio principal
do artefato: o controle formal de mudanças de escopo (linha de base aceita,
solicitação de mudança, avaliação de impacto, decisão do cliente e efeitos da
aprovação). Os demais módulos (pagamentos, etiquetas, anexos, conversa etc.)
foram omitidos de propósito para manter os diagramas simples e legíveis.

Os elementos foram extraídos do código-fonte:
  - ER: database/schema.sql e database/migrations/002, 006, 009 (nomes reais
    de tabelas e colunas; apenas as colunas relevantes para a regra).
  - Classes: src/entities/*.rs, src/dtos/change_request.rs (enumeração de
    estados) e src/services/change_requests.rs (operações do serviço).
  - Casos de uso: rotas de src/controllers/change_requests.rs e
    src/controllers/scope.rs.

Uso:
    python3 scripts/gerar_diagramas_tcc2.py              # gera .puml, .png e .svg
    python3 scripts/gerar_diagramas_tcc2.py --no-render  # somente .puml
    python3 scripts/gerar_diagramas_tcc2.py --saida /outro/diretorio

A renderização usa o comando `plantuml` (requer Java e Graphviz). Na falta dele,
os arquivos .puml podem ser abertos em https://www.plantuml.com/plantuml.
"""
from __future__ import annotations

import argparse
import os
import shutil
import subprocess
import sys
from pathlib import Path

RAIZ = Path(__file__).resolve().parent.parent
SAIDA_PADRAO = RAIZ / "docs" / "diagrams" / "tcc2"

# Estilo comum: preto e branco, fonte sem serifa, adequado à impressão (ABNT).
ESTILO = """
skinparam dpi 150
skinparam shadowing false
skinparam monochrome true
skinparam defaultFontName Arial
skinparam defaultFontSize 13
skinparam roundCorner 4
skinparam padding 4
skinparam ArrowColor #000000
skinparam BackgroundColor #FFFFFF
"""

CASOS_DE_USO = f"""@startuml diagrama-casos-de-uso
{ESTILO}
left to right direction
skinparam packageStyle rectangle
skinparam usecase {{
  BorderColor #000000
  BackgroundColor #FFFFFF
}}

actor "Freelancer" as F
actor "Cliente" as C

rectangle "Controle formal de mudanças de escopo" {{
  usecase "Publicar revisão do escopo" as UC01
  usecase "Aceitar escopo\\n(linha de base)" as UC02
  usecase "Registrar solicitação\\nde mudança" as UC03
  usecase "Avaliar impacto\\n(horas, custo e prazo)" as UC04
  usecase "Consultar solicitação\\ne impactos" as UC05
  usecase "Aprovar mudança" as UC06
  usecase "Recusar mudança\\n(com motivo)" as UC07
  usecase "Aplicar efeitos da aprovação\\n(fatura suplementar e prazo do marco)" as UC08
  usecase "Reabrir mudança recusada" as UC09
  usecase "Vincular tarefas à mudança" as UC10
  usecase "Marcar mudança\\ncomo implementada" as UC11
  usecase "Cancelar solicitação" as UC12
}}

F -- UC01
F -- UC03
F -- UC04
F -- UC05
F -- UC09
F -- UC10
F -- UC11
F -- UC12

UC02 -- C
UC03 -- C
UC05 -- C
UC06 -- C
UC07 -- C
UC12 -- C

UC06 .[norank].> UC08 : <<include>>
UC04 .[norank].> UC02 : <<requer>>
UC11 .[norank].> UC10 : <<requer>>
@enduml
"""

CLASSES = f"""@startuml diagrama-classes
{ESTILO}
hide circle
hide empty members
skinparam linetype ortho
skinparam nodesep 60
skinparam ranksep 60
skinparam class {{
  BorderColor #000000
  BackgroundColor #FFFFFF
}}

class Client {{
  id: Uuid
  name: String
  email: String
  status: String
}}

class Project {{
  id: Uuid
  name: String
  status: String
  billable: bool
}}

class ScopeRevision {{
  revision: i32
  document: Json
  published: bool
  revision_kind: String
  accepted_at: DateTime
  accepted_by: Uuid
}}

class ChangeRequest {{
  id: Uuid
  origin: String
  title: String
  client_request_description: String
  technical_assessment: String
  impact_hours: Decimal
  impact_cost: Decimal
  schedule_impact_days: i32
  status: ChangeRequestStatus
  decline_reason: String
  client_approved_at: DateTime
  version: i32
}}

enum ChangeRequestStatus <<enumeration>> {{
  Draft
  Submitted
  Assessed
  Approved
  Declined
  Implemented
  Cancelled
}}

class ChangeRequestService <<service>> {{
  + create(dto)
  + create_by_client(dto)
  + submit(id, version)
  + assess(id, dto)
  + client_approve(id, dto)
  + client_decline(id, dto)
  + reopen(id, version)
  + mark_implemented(id, version)
  + cancel(id, version)
}}

class ScopeChangeRevision {{
  version: i32
  snapshot: Json
  actor_kind: String
  recorded_at: DateTime
}}

class ScopeEffect {{
  offer_version: i32
  amount: Decimal
  schedule_before: DateTime
  schedule_after: DateTime
}}

class Milestone {{
  id: Uuid
  name: String
  status: String
  due_date: DateTime
}}

class Invoice {{
  id: Uuid
  amount: Decimal
  currency_type: String
  status: String
}}

class Task {{
  id: Uuid
  title: String
  status: String
}}

Client "1" -- "0..*" Project
Project "1" *-- "0..*" ScopeRevision : linha de base
Project "1" -- "0..*" ChangeRequest
Project "1" *-- "0..*" Milestone
Project "1" -- "0..*" Task
ScopeRevision "1" -- "0..*" ChangeRequest : avaliada sobre
ChangeRequest "1" *-- "1..*" ScopeChangeRevision : histórico
ChangeRequest "1" -- "0..1" ScopeEffect : gera ao aprovar
ScopeEffect "0..1" -- "0..1" Invoice
ChangeRequest "0..*" -- "0..1" Milestone : marco afetado
ChangeRequest "0..1" -- "0..*" Task : tarefas extras
ChangeRequest ..> ChangeRequestStatus
ChangeRequestService ..> ChangeRequest : altera
@enduml
"""

ER = f"""@startuml diagrama-er
{ESTILO}
hide circle
hide empty members
skinparam linetype ortho
skinparam nodesep 50
skinparam ranksep 70
skinparam class {{
  BorderColor #000000
  BackgroundColor #FFFFFF
}}

entity "clients" as clients {{
  * id : uuid <<PK>>
  --
  name : varchar(255)
  email : varchar(255)
  status : varchar(50)
}}

entity "projects" as projects {{
  * id : uuid <<PK>>
  --
  client_id : uuid <<FK>>
  name : varchar(255)
  status : varchar(20)
  billable : boolean
}}

entity "scope_projects" as scope_projects {{
  * project_id : uuid <<PK, FK>>
  --
  client_id : uuid <<FK>>
  last_revision : integer
  accepted_revision : integer
  activated_at : timestamptz
}}

entity "scope_revisions" as scope_revisions {{
  * project_id : uuid <<PK, FK>>
  * revision : integer <<PK>>
  --
  document : jsonb
  published : boolean
  revision_kind : text
  accepted_at : timestamptz
  accepted_by : uuid
}}

entity "change_requests" as change_requests {{
  * id : uuid <<PK>>
  --
  project_id : uuid <<FK>>
  client_id : uuid <<FK>>
  scope_revision : integer <<FK>>
  target_milestone_id : uuid <<FK>>
  origin : varchar(30)
  title : varchar(255)
  client_request_description : text
  technical_assessment : text
  impact_hours : numeric(6,2)
  impact_cost : numeric(15,4)
  schedule_impact_days : integer
  status : varchar(30)
  decline_reason : varchar(100)
  client_approved_at : timestamptz
  version : integer
}}

entity "scope_change_revisions" as scope_change_revisions {{
  * change_request_id : uuid <<PK, FK>>
  * version : integer <<PK>>
  --
  snapshot : jsonb
  actor_kind : text
  recorded_at : timestamptz
}}

entity "scope_effects" as scope_effects {{
  * change_request_id : uuid <<PK, FK>>
  --
  invoice_id : uuid <<FK>>
  milestone_id : uuid <<FK>>
  offer_version : integer
  amount : numeric(15,4)
  schedule_before : timestamptz
  schedule_after : timestamptz
}}

entity "milestones" as milestones {{
  * id : uuid <<PK>>
  --
  project_id : uuid <<FK>>
  name : varchar(255)
  status : varchar(20)
  due_date : timestamptz
}}

entity "invoices" as invoices {{
  * id : uuid <<PK>>
  --
  project_id : uuid <<FK>>
  amount : numeric(15,4)
  currency_type : varchar(3)
  status : varchar(30)
}}

entity "tasks" as tasks {{
  * id : uuid <<PK>>
  --
  project_id : uuid <<FK>>
  title : varchar(255)
  status : varchar(50)
}}

entity "scope_task_links" as scope_task_links {{
  * task_id : uuid <<PK, FK>>
  --
  project_id : uuid <<FK>>
  baseline_revision : integer <<FK>>
  change_request_id : uuid <<FK>>
  reason : text
}}

clients ||--o{{ projects
projects ||--o| scope_projects
scope_projects ||--|{{ scope_revisions
projects ||--o{{ change_requests
scope_revisions ||--o{{ change_requests
change_requests ||--|{{ scope_change_revisions
change_requests ||--o| scope_effects
scope_effects |o--o| invoices
projects ||--o{{ milestones
milestones |o--o{{ change_requests
projects ||--o{{ invoices
projects ||--o{{ tasks
tasks ||--o| scope_task_links
change_requests |o--o{{ scope_task_links
@enduml
"""

DIAGRAMAS = {
    "diagrama-casos-de-uso": CASOS_DE_USO,
    "diagrama-classes": CLASSES,
    "diagrama-er": ER,
}


def renderizar(arquivos: list[Path], saida: Path) -> bool:
    plantuml = shutil.which("plantuml")
    if not plantuml:
        print("Aviso: comando 'plantuml' não encontrado; foram gerados apenas os .puml.")
        print("Instale com: sudo apt install plantuml graphviz")
        return False
    for formato in ("png", "svg"):
        resultado = subprocess.run(
            [plantuml, f"-t{formato}", "-charset", "UTF-8", "-o", str(saida), *map(str, arquivos)],
            capture_output=True,
            text=True,
            env={**os.environ, "PLANTUML_LIMIT_SIZE": "16384"},
        )
        if resultado.returncode != 0:
            print(resultado.stdout, resultado.stderr, file=sys.stderr)
            return False
    return True


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--saida", type=Path, default=SAIDA_PADRAO, help="diretório de saída")
    parser.add_argument("--no-render", action="store_true", help="gera somente os arquivos .puml")
    args = parser.parse_args()

    saida: Path = args.saida.resolve()
    saida.mkdir(parents=True, exist_ok=True)

    arquivos = []
    for nome, conteudo in DIAGRAMAS.items():
        caminho = saida / f"{nome}.puml"
        caminho.write_text(conteudo.strip() + "\n", encoding="utf-8")
        arquivos.append(caminho)
        print(f"Gerado: {caminho.relative_to(RAIZ) if caminho.is_relative_to(RAIZ) else caminho}")

    if not args.no_render and renderizar(arquivos, saida):
        print(f"Imagens PNG e SVG geradas em {saida}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
