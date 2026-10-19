#!/usr/bin/env python3
"""Gera os diagramas do Horbyta (Classes, Casos de Uso e ER) em PlantUML.

Dois escopos:
  - artigo (padrão): somente o que o artigo do TCC descreve — módulos de clientes,
    projetos, tarefas, marcos, registros de tempo, faturamento, pagamentos,
    notificações e solicitações de mudança, com o fluxo de impacto (horas, custo,
    prazo) e aprovação ou rejeição pelo cliente.
  - completo: o sistema implementado inteiro (inclui revisões de escopo,
    conversa do projeto, renegociação etc.).

Fontes, para que os diagramas reflitam o sistema real:
  - ER: catálogo do PostgreSQL de uma base migrada (tabelas, colunas, PK e FK).
  - Classes: entidades SeaORM de src/entities (atributos e relações) e, no escopo
    completo, os serviços de src/services (métodos públicos).
  - Casos de uso: funcionalidades expostas pela API, agrupadas por ator.

Uso:
  python3 scripts/generate-diagrams.py                    # escopo do artigo
  python3 scripts/generate-diagrams.py --escopo completo  # sistema inteiro
  python3 scripts/generate-diagrams.py --no-render        # somente .puml

A conexão vem de --database-url, DATABASE_URL ou do .env do repositório; apenas
consultas de leitura são executadas. A renderização usa PlantUML com Graphviz
(local ou imagem Docker plantuml/plantuml) e, na falta dele, o motor interno.
"""
from __future__ import annotations

import argparse
import json
import os
import re
import shutil
import subprocess
import sys
import urllib.request
from collections import defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
PLANTUML_VERSION = "1.2024.7"
PLANTUML_URL = (
    "https://repo1.maven.org/maven2/net/sourceforge/plantuml/plantuml/"
    f"{PLANTUML_VERSION}/plantuml-{PLANTUML_VERSION}.jar"
)
DOCKER_IMAGE = os.environ.get("PLANTUML_DOCKER_IMAGE", "plantuml/plantuml")
# Infraestrutura interna: não faz parte do modelo de dados do domínio.
IGNORED_TABLES = {"horbyta_migrations"}
# user_id (o freelancer proprietário) existe em quase todas as tabelas; as linhas
# até users são omitidas para legibilidade e a legenda informa isso.
OWNER_TABLE = "users"
OWNER_LINKS_KEPT = {"clients", "projects", "notifications", "client_auth"}

# Tabelas do escopo do artigo: os nove módulos citados + o histórico imutável das
# solicitações (o artigo exige que cada pedido seja um registro imutável).
ARTICLE_TABLES = [
    "users", "clients", "client_auth", "projects", "milestones", "tasks", "time_entries",
    "project_billing", "invoices", "payments", "notifications", "change_requests",
    "scope_change_revisions",
]
# Colunas da solicitação de mudança que pertencem ao controle de escopo avançado
# (revisões de acordo), fora do que o artigo descreve.
ARTICLE_HIDDEN_COLUMNS = {
    "projects": {"enforce_scope"},
    "tasks": {"scope_preparation_reason"},
    "change_requests": {"scope_revision", "currency_type", "billing_terms", "target_milestone_id",
                        "schedule_before", "schedule_after", "legacy", "client_signature_ip_source",
                        "decline_comment"},
}

STYLE = """skinparam shadowing false
skinparam defaultFontName Arial
skinparam defaultFontSize 12
skinparam roundCorner 6
skinparam ArrowColor #4B5563
skinparam ArrowThickness 1.1
skinparam packageStyle rectangle
skinparam nodesep 70
skinparam ranksep 90
' Visual sóbrio, adequado a um trabalho acadêmico (independe do tema padrão do PlantUML).
skinparam BackgroundColor white
skinparam ClassBackgroundColor white
skinparam ClassBorderColor #374151
skinparam ClassHeaderBackgroundColor #F3F4F6
skinparam EntityBackgroundColor white
skinparam EntityBorderColor #374151
skinparam UsecaseBackgroundColor white
skinparam UsecaseBorderColor #374151
skinparam ActorBackgroundColor white
skinparam ActorBorderColor #374151
skinparam PackageBorderColor #6B7280
skinparam RectangleBorderColor #374151
skinparam LegendBackgroundColor #F9FAFB
skinparam LegendBorderColor #9CA3AF
skinparam TitleFontSize 16
skinparam TitleFontStyle bold
"""


# ---------------------------------------------------------------------------
# Utilidades
# ---------------------------------------------------------------------------
def database_url(explicit: str | None) -> str:
    if explicit:
        return explicit
    if os.environ.get("DATABASE_URL"):
        return os.environ["DATABASE_URL"]
    env_file = ROOT / ".env"
    if env_file.exists():
        for line in env_file.read_text().splitlines():
            match = re.match(r"\s*DATABASE_URL\s*=\s*(.+?)\s*$", line)
            if match:
                return match.group(1).strip().strip("'\"")
    sys.exit("Falta DATABASE_URL: use --database-url, exporte-a ou defina-a no .env.")


def query_json(url: str, sql: str):
    """Executa uma consulta somente leitura com psql e devolve o JSON resultante."""
    env = dict(os.environ, PGOPTIONS="-c default_transaction_read_only=on")
    result = subprocess.run(["psql", url, "-X", "-At", "-v", "ON_ERROR_STOP=1", "-c", sql],
                            capture_output=True, text=True, env=env)
    if result.returncode != 0:
        sys.exit(f"Erro ao consultar a base:\n{result.stderr}")
    return json.loads(result.stdout or "null")


def pascal(name: str) -> str:
    return "".join(part.capitalize() for part in name.split("_"))


def singular(table: str) -> str:
    irregular = {"seq_by_user": "SeqByUser", "user_ui_state": "UserUiState",
                 "conversation_history": "ConversationHistory", "task_tags": "TaskTag",
                 "time_entries": "TimeEntry", "activities": "Activity",
                 "categories": "Category", "client_auth": "ClientAuth",
                 "project_billing": "ProjectBilling"}
    if table in irregular:
        return irregular[table]
    return pascal(table[:-1] if table.endswith("s") else table)


def module_group(table: str) -> str:
    if table in {"change_requests", "scope_change_revisions"}:
        return "Solicitações de mudança"
    if table.startswith("scope_") or table in {"project_messages", "project_message_attachments",
                                               "project_conversation_reads",
                                               "conversation_change_sources"}:
        return "Escopo, negociação e conversa"
    if table in {"invoices", "payments", "project_billing", "time_entries", "activities"}:
        return "Tempo e faturamento"
    if table in {"users", "client_auth", "user_ui_state", "notifications", "seq_by_user",
                 "conversation_history"}:
        return "Usuários e acesso"
    return "Gestão de projetos"


# ---------------------------------------------------------------------------
# Diagrama ER (catálogo do PostgreSQL)
# ---------------------------------------------------------------------------
ER_SQL = r"""
SELECT json_build_object(
  'columns', (SELECT json_agg(json_build_object(
      'table', c.relname, 'name', a.attname, 'type', format_type(a.atttypid, a.atttypmod),
      'not_null', a.attnotnull) ORDER BY c.relname, a.attnum)
    FROM pg_attribute a JOIN pg_class c ON c.oid = a.attrelid
    JOIN pg_namespace n ON n.oid = c.relnamespace
    WHERE n.nspname = current_schema() AND c.relkind = 'r' AND a.attnum > 0 AND NOT a.attisdropped),
  'primary', (SELECT json_agg(json_build_object('table', c.relname, 'columns',
      (SELECT json_agg(a.attname) FROM unnest(k.conkey) col JOIN pg_attribute a
         ON a.attrelid = k.conrelid AND a.attnum = col)))
    FROM pg_constraint k JOIN pg_class c ON c.oid = k.conrelid
    JOIN pg_namespace n ON n.oid = c.relnamespace
    WHERE n.nspname = current_schema() AND k.contype = 'p'),
  'unique', (SELECT json_agg(json_build_object('table', c.relname, 'columns',
      (SELECT json_agg(a.attname) FROM unnest(k.conkey) col JOIN pg_attribute a
         ON a.attrelid = k.conrelid AND a.attnum = col)))
    FROM pg_constraint k JOIN pg_class c ON c.oid = k.conrelid
    JOIN pg_namespace n ON n.oid = c.relnamespace
    WHERE n.nspname = current_schema() AND k.contype = 'u'),
  'foreign', (SELECT json_agg(json_build_object('table', c.relname, 'target', t.relname,
      'columns', (SELECT json_agg(a.attname ORDER BY col.ord) FROM unnest(k.conkey) WITH ORDINALITY col(num, ord)
         JOIN pg_attribute a ON a.attrelid = k.conrelid AND a.attnum = col.num)))
    FROM pg_constraint k JOIN pg_class c ON c.oid = k.conrelid
    JOIN pg_class t ON t.oid = k.confrelid
    JOIN pg_namespace n ON n.oid = c.relnamespace
    WHERE n.nspname = current_schema() AND k.contype = 'f')
)
"""


def er_diagram(url: str, scope: str) -> str:
    data = query_json(url, ER_SQL)
    allowed = set(ARTICLE_TABLES) if scope == "artigo" else None

    def included(table: str) -> bool:
        return table not in IGNORED_TABLES and (allowed is None or table in allowed)

    hidden = ARTICLE_HIDDEN_COLUMNS if scope == "artigo" else {}
    columns = defaultdict(list)
    for column in data["columns"] or []:
        if included(column["table"]) and column["name"] not in hidden.get(column["table"], set()):
            columns[column["table"]].append(column)
    primary = {p["table"]: set(p["columns"]) for p in data["primary"] or []}
    unique = defaultdict(list)
    for u in data["unique"] or []:
        unique[u["table"]].append(set(u["columns"]))
    foreign = [f for f in data["foreign"] or [] if included(f["table"]) and included(f["target"])
               and not (set(f["columns"]) & hidden.get(f["table"], set()))]
    fk_columns = defaultdict(set)
    for f in foreign:
        fk_columns[f["table"]].update(f["columns"])

    title = "Diagrama Entidade-Relacionamento" + (" (escopo do artigo)" if scope == "artigo" else " (sistema completo)")
    lines = [f"@startuml diagrama-er", STYLE, f"title Horbyta — {title}",
             "hide circle", "skinparam linetype ortho", ""]
    groups = defaultdict(list)
    for table in sorted(columns):
        groups[module_group(table)].append(table)
    for group, tables in groups.items():
        lines.append(f'package "{group}" {{')
        for table in tables:
            lines.append(f'  entity "{table}" as {table} {{')
            keys = [c for c in columns[table] if c["name"] in primary.get(table, set())]
            rest = [c for c in columns[table] if c["name"] not in primary.get(table, set())]
            for column in keys:
                fk = " <<FK>>" if column["name"] in fk_columns[table] else ""
                lines.append(f'    * {column["name"]} : {column["type"]} <<PK>>{fk}')
            lines.append("    --")
            for column in rest:
                tags = []
                if column["name"] in fk_columns[table]:
                    tags.append("<<FK>>")
                if any(u == {column["name"]} for u in unique[table]):
                    tags.append("<<UQ>>")
                mark = "*" if column["not_null"] else " "
                lines.append(f'    {mark} {column["name"]} : {column["type"]}{" " + " ".join(tags) if tags else ""}')
            lines.append("  }")
        lines.append("}")
        lines.append("")

    seen = set()
    for f in foreign:
        child, parent, cols = f["table"], f["target"], set(f["columns"])
        if parent == OWNER_TABLE and cols == {"user_id"} and child not in OWNER_LINKS_KEPT:
            continue
        key = (child, parent, tuple(sorted(cols)))
        if key in seen:
            continue
        seen.add(key)
        required = all(c["not_null"] for c in columns[child] if c["name"] in cols)
        one_to_one = cols == primary.get(child, set()) or cols in unique[child]
        lines.append(f"{parent} {'||' if required else '|o'}--{'o|' if one_to_one else 'o{'} {child}")
    lines += ["", "legend right",
              "  * obrigatório (NOT NULL)   <<PK>> chave primária   <<FK>> chave estrangeira   <<UQ>> única",
              "  user_id (freelancer proprietário) existe em quase todas as tabelas; essas ligações a users",
              "  são omitidas para legibilidade, exceto em clients, projects, notifications e client_auth.",
              "  Gerado do catálogo do PostgreSQL por scripts/generate-diagrams.py",
              "endlegend", "@enduml"]
    return "\n".join(lines) + "\n"


# ---------------------------------------------------------------------------
# Diagrama de classes (código Rust)
# ---------------------------------------------------------------------------
def rust_type(raw: str) -> str:
    raw = raw.strip()
    optional = raw.startswith("Option<")
    inner = raw[len("Option<"):-1] if optional else raw
    inner = {"DateTimeWithTimeZone": "DateTime"}.get(inner, inner)
    return f"{inner}?" if optional else inner


def parse_entities():
    entities = {}
    for path in sorted((ROOT / "src" / "entities").glob("*.rs")):
        if path.stem in {"mod", "prelude"}:
            continue
        text = path.read_text()
        table = re.search(r'table_name\s*=\s*"([^"]+)"', text)
        model = re.search(r"pub struct Model\s*\{(.*?)\n\}", text, re.S)
        if not table or not model:
            continue
        fields, pending_pk = [], False
        for line in model.group(1).splitlines():
            line = line.strip()
            if line.startswith("#[") and "primary_key" in line:
                pending_pk = True
            match = re.match(r"pub (\w+): (.+),$", line)
            if match:
                fields.append((match.group(1), rust_type(match.group(2)), pending_pk))
                pending_pk = False
        belongs = re.findall(r'belongs_to\s*=\s*"super::(\w+)::Entity",\s*from\s*=\s*"Column::(\w+)"', text)
        entities[path.stem] = {"table": table.group(1), "fields": fields, "belongs": belongs}
    return entities


def split_params(text: str) -> list[str]:
    params, depth, current = [], 0, ""
    for char in text:
        if char in "<([":
            depth += 1
        elif char in ">)]":
            depth -= 1
        if char == "," and depth == 0:
            params.append(current)
            current = ""
        else:
            current += char
    if current.strip():
        params.append(current)
    names = []
    for param in params:
        param = param.strip()
        if not param or param.startswith("&self") or param == "self":
            continue
        name = param.split(":", 1)[0].strip().removeprefix("mut ")
        if name not in {"db", "tx", "state", "storage"}:
            names.append(name)
    return names


def parse_services(entity_modules: set[str]):
    services = {}
    for path in sorted((ROOT / "src" / "services").glob("*.rs")):
        if path.stem == "mod":
            continue
        text = path.read_text()
        for impl in re.finditer(r"\nimpl (\w+Service) \{", text):
            start, depth = impl.end(), 1
            end = start
            while depth and end < len(text):
                depth += {"{": 1, "}": -1}.get(text[end], 0)
                end += 1
            body = text[start:end]
            methods = []
            for method in re.finditer(r"\n    pub (?:async )?fn (\w+)(?:<[^>]*>)?\(", body):
                depth, cursor = 1, method.end()
                while depth and cursor < len(body):
                    depth += {"(": 1, ")": -1}.get(body[cursor], 0)
                    cursor += 1
                methods.append(f"+ {method.group(1)}({', '.join(split_params(body[method.end():cursor - 1]))})")
            used = {m for m in entity_modules
                    if re.search(rf"\b(entities::{m}\b|{m}::(Entity|Model|ActiveModel|Column))", text)
                    or re.search(rf"entities::\{{[^}}]*\b{m}\b", text)}
            entry = services.setdefault(impl.group(1), {"methods": [], "uses": set()})
            entry["methods"].extend(methods)
            entry["uses"].update(used)
    return services


# Classes do escopo do artigo: nome do domínio em português, atributos relevantes
# (tipos lidos do código) e as operações que o serviço correspondente implementa.
ARTICLE_CLASSES = {
    "users": ("Freelancer", ["id", "name", "email", "role", "status"],
              ["+ autenticar(email, senha)"]),
    "clients": ("Cliente", ["id", "name", "email", "phone", "company", "status"],
                ["+ acessarPortal(email, senha)"]),
    "projects": ("Projeto", ["id", "name", "description", "status", "billable"],
                 ["+ alterarStatus(status)", "+ configurarFaturamento(tipo, valor)"]),
    "milestones": ("Marco", ["id", "name", "status", "due_date"], ["+ atualizar(dados)"]),
    "tasks": ("Tarefa", ["id", "title", "status", "priority", "due_date"], ["+ mover(status)"]),
    "time_entries": ("RegistroDeTempo", ["id", "start_time", "end_time", "duration_minutes", "billable"],
                     ["+ iniciarCronometro()", "+ pararCronometro()"]),
    "invoices": ("Fatura", ["id", "amount", "currency_type", "status", "due_date"],
                 ["+ alterarStatus(status)"]),
    "payments": ("Pagamento", ["id", "amount", "payment_date", "transaction_type"],
                 ["+ registrar(valor)", "+ estornar(valor)"]),
    "notifications": ("Notificacao", ["id", "title", "body", "is_read"], ["+ marcarComoLida()"]),
    "change_requests": ("SolicitacaoDeMudanca",
                        ["id", "title", "client_request_description", "technical_assessment",
                         "impact_hours", "impact_cost", "schedule_impact_days", "status",
                         "decline_reason", "version"],
                        ["+ submeter()", "+ avaliar(horas, custo, dias)", "+ aprovar()",
                         "+ rejeitar(motivo)", "+ marcarComoImplementada()"]),
}
CHANGE_STATUS = ["rascunho (draft)", "enviada (submitted)", "avaliada (assessed)",
                 "aprovada (approved)", "rejeitada (declined)", "implementada (implemented)",
                 "cancelada (cancelled)"]


def class_diagram(entities: dict, scope: str) -> str:
    article = scope == "artigo"
    title = "Diagrama de Classes" + (" (escopo do artigo)" if article else " (modelo de domínio completo)")
    lines = ["@startuml diagrama-classes", STYLE, f"title Horbyta — {title}",
             "skinparam classAttributeIconSize 0", "skinparam linetype ortho", "hide empty methods", ""]
    names = {}
    for module, entity in sorted(entities.items()):
        table = entity["table"]
        if article and table not in ARTICLE_CLASSES:
            continue
        if article:
            name, keep, operations = ARTICLE_CLASSES[table]
            fields = [f for f in entity["fields"] if f[0] in keep]
        else:
            name, operations = singular(table), []
            fields = entity["fields"]
        names[module] = name
        lines.append(f"class {name} {{")
        for field, kind, pk in fields:
            lines.append(f"  - {field}: {kind}{'  «PK»' if pk else ''}")
        if operations:
            lines.append("  --")
            lines += [f"  {op}" for op in operations]
        lines.append("}")
    if article:
        lines.append("enum StatusDaSolicitacao {")
        lines += [f"  {status}" for status in CHANGE_STATUS]
        lines.append("}")
        lines.append('SolicitacaoDeMudanca --> StatusDaSolicitacao : status')
    lines.append("")
    for module, entity in sorted(entities.items()):
        if module not in names:
            continue
        for target, column in entity["belongs"]:
            if target not in names:
                continue
            field = re.sub(r"(?<!^)(?=[A-Z])", "_", column).lower()
            if entities[target]["table"] == OWNER_TABLE and field == "user_id" \
                    and entity["table"] not in OWNER_LINKS_KEPT:
                continue
            optional = any(n == field and k.endswith("?") for n, k, _ in entity["fields"])
            lines.append(f'{names[target]} "{"0..1" if optional else "1"}" -- "0..*" {names[module]} : {field}')
    lines += ["", "legend right",
              "  Gerado de src/entities (SeaORM) por scripts/generate-diagrams.py. Tipo com ? = opcional.",
              "  O vínculo user_id com o Freelancer existe em quase todas as classes; desenhado apenas",
              "  em Cliente, Projeto e Notificacao para legibilidade." if article else
              "  O vínculo user_id com User existe em quase todas as classes; desenhado apenas onde é central.",
              "endlegend", "@enduml"]
    return "\n".join(lines) + "\n"


def services_diagram(entities: dict) -> str:
    services = parse_services(set(entities))
    lines = ["@startuml diagrama-classes-servicos", STYLE,
             "title Horbyta — Diagrama de Classes (serviços de aplicação)",
             "skinparam classAttributeIconSize 0", "hide empty members", "left to right direction", ""]
    lines.append('package "Serviços (src/services)" {')
    for name, service in sorted(services.items()):
        if service["methods"]:
            lines.append(f"  class {name} <<service>> {{")
            lines += [f"    {method}" for method in service["methods"]]
            lines.append("  }")
    lines.append("}")
    used = sorted({m for s in services.values() if s["methods"] for m in s["uses"]})
    lines.append('package "Entidades (src/entities)" {')
    lines += [f"  class {singular(entities[m]['table'])} <<entity>>" for m in used]
    lines.append("}")
    for name, service in sorted(services.items()):
        if service["methods"]:
            lines += [f"{name} ..> {singular(entities[m]['table'])}" for m in sorted(service["uses"])]
    lines += ["", "legend right",
              "  Gerado de src/services. O parâmetro de conexão (db/tx), comum a todas as operações, é omitido.",
              "endlegend", "@enduml"]
    return "\n".join(lines) + "\n"


# ---------------------------------------------------------------------------
# Diagrama de casos de uso
# ---------------------------------------------------------------------------
# Escopo do artigo: os módulos citados e o fluxo da Figura 1 (solicitação → impacto
# em horas, custo e prazo → aprovação ou rejeição do cliente → liberação).
ARTICLE_USE_CASES = {
    "UC_LOGIN": ("Autenticar-se", None),
    "UC_CLIENTS": ("Gerenciar clientes", "Gestão de projetos"),
    "UC_PROJECTS": ("Gerenciar projetos", "Gestão de projetos"),
    "UC_TASKS": ("Gerenciar tarefas", "Gestão de projetos"),
    "UC_MILESTONES": ("Gerenciar marcos", "Gestão de projetos"),
    "UC_TIME": ("Registrar tempo", "Tempo e faturamento"),
    "UC_INVOICES": ("Gerenciar faturamento", "Tempo e faturamento"),
    "UC_PAYMENTS": ("Registrar pagamentos", "Tempo e faturamento"),
    "UC_CR_CREATE": ("Registrar solicitação\nde mudança", "Solicitações de mudança"),
    "UC_CR_ASSESS": ("Avaliar solicitação\nde mudança", "Solicitações de mudança"),
    "UC_CR_IMPACT": ("Estimar impacto\n(horas, custo e prazo)", "Solicitações de mudança"),
    "UC_CR_DECIDE": ("Aprovar ou rejeitar\nsolicitação de mudança", "Solicitações de mudança"),
    "UC_CR_RELEASE": ("Liberar mudança para\no desenvolvimento", "Solicitações de mudança"),
    "UC_FOLLOW": ("Acompanhar projetos\ne marcos", "Portal do cliente"),
    "UC_NOTIFY": ("Consultar notificações", None),
}
ARTICLE_ACTORS = {
    "Freelancer": ["UC_LOGIN", "UC_CLIENTS", "UC_PROJECTS", "UC_TASKS", "UC_MILESTONES", "UC_TIME",
                   "UC_INVOICES", "UC_PAYMENTS", "UC_CR_CREATE", "UC_CR_ASSESS", "UC_NOTIFY"],
    "Cliente": ["UC_LOGIN", "UC_FOLLOW", "UC_CR_CREATE", "UC_CR_DECIDE", "UC_NOTIFY"],
}
ARTICLE_RELATIONS = [
    ("UC_CR_ASSESS", "include", "UC_CR_IMPACT"),
    ("UC_CR_RELEASE", "extend", "UC_CR_DECIDE"),
]

FULL_USE_CASES = {
    "UC_LOGIN": ("Autenticar-se", "Acesso"),
    "UC_2FA": ("Configurar autenticação\nde dois fatores", "Acesso"),
    "UC_PROFILE": ("Gerenciar perfil\ne preferências", "Acesso"),
    "UC_USERS": ("Administrar usuários", "Acesso"),
    "UC_CLIENTS": ("Gerenciar clientes", "Gestão de projetos"),
    "UC_PORTAL": ("Habilitar acesso do\ncliente ao portal", "Gestão de projetos"),
    "UC_PROJECTS": ("Gerenciar projetos", "Gestão de projetos"),
    "UC_BILLING_CFG": ("Configurar faturamento\ndo projeto", "Gestão de projetos"),
    "UC_MILESTONES": ("Gerenciar marcos", "Gestão de projetos"),
    "UC_TASKS": ("Gerenciar tarefas\n(quadro Kanban)", "Gestão de projetos"),
    "UC_CATALOGS": ("Gerenciar categorias,\netiquetas e atividades", "Gestão de projetos"),
    "UC_FILES": ("Anexar arquivos", "Gestão de projetos"),
    "UC_TIME": ("Registrar tempo\n(cronômetro ou manual)", "Tempo e faturamento"),
    "UC_INVOICES": ("Gerenciar faturas", "Tempo e faturamento"),
    "UC_PAYMENTS": ("Registrar pagamentos\ne estornos", "Tempo e faturamento"),
    "UC_DASHBOARD": ("Consultar painel", "Tempo e faturamento"),
    "UC_PUBLISH": ("Publicar revisão\nde escopo", "Controle de escopo"),
    "UC_AMEND": ("Corrigir o acordo\nvigente", "Controle de escopo"),
    "UC_REVIEW": ("Revisar proposta\nde escopo", "Controle de escopo"),
    "UC_ACCEPT": ("Aceitar escopo", "Controle de escopo"),
    "UC_EVIDENCE": ("Registrar evidência\nde aceite (data, IP)", "Controle de escopo"),
    "UC_ADJUST": ("Solicitar ajustes\nou rejeitar proposta", "Controle de escopo"),
    "UC_REASON": ("Informar motivo", "Controle de escopo"),
    "UC_CLASSIFY": ("Classificar tarefas no\nescopo ou como extra", "Controle de escopo"),
    "UC_CR_CREATE": ("Registrar solicitação\nde mudança", "Solicitações de mudança"),
    "UC_CR_FROM_MSG": ("Criar solicitação a\npartir da conversa", "Solicitações de mudança"),
    "UC_CR_ASSESS": ("Avaliar solicitação\n(publicar oferta)", "Solicitações de mudança"),
    "UC_CR_APPROVE": ("Aprovar oferta\nde mudança", "Solicitações de mudança"),
    "UC_CR_DECLINE": ("Rejeitar oferta\nde mudança", "Solicitações de mudança"),
    "UC_CR_REOPEN": ("Reabrir negociação de\numa mudança rejeitada", "Solicitações de mudança"),
    "UC_CONSOLIDATE": ("Consolidar a mudança\nno acordo", "Solicitações de mudança"),
    "UC_DRAFT_INVOICE": ("Gerar fatura em rascunho\ndo suplemento", "Solicitações de mudança"),
    "UC_CHAT": ("Conversar no projeto", "Comunicação"),
    "UC_PENDING": ("Consultar ações pendentes\ne notificações", "Comunicação"),
    "UC_NOTIFY": ("Notificar em tempo real", "Comunicação"),
    "UC_CLIENT_VIEW": ("Acompanhar projetos,\nmarcos e painel", "Comunicação"),
}
FULL_ACTORS = {
    "Freelancer": ["UC_LOGIN", "UC_2FA", "UC_PROFILE", "UC_CLIENTS", "UC_PROJECTS", "UC_MILESTONES",
                   "UC_TASKS", "UC_CATALOGS", "UC_FILES", "UC_TIME", "UC_INVOICES", "UC_PAYMENTS",
                   "UC_DASHBOARD", "UC_PUBLISH", "UC_CLASSIFY", "UC_CR_CREATE", "UC_CR_FROM_MSG",
                   "UC_CR_ASSESS", "UC_CR_REOPEN", "UC_CHAT"],
    "Administrador": ["UC_USERS"],
    "Cliente": ["UC_LOGIN", "UC_CLIENT_VIEW", "UC_REVIEW", "UC_ACCEPT", "UC_ADJUST", "UC_CR_CREATE",
                "UC_CR_APPROVE", "UC_CR_DECLINE", "UC_CHAT", "UC_PENDING"],
    "Sistema": ["UC_NOTIFY", "UC_CONSOLIDATE", "UC_DRAFT_INVOICE"],
}
FULL_RELATIONS = [
    ("UC_PORTAL", "extend", "UC_CLIENTS"), ("UC_BILLING_CFG", "extend", "UC_PROJECTS"),
    ("UC_AMEND", "extend", "UC_PUBLISH"), ("UC_ACCEPT", "include", "UC_EVIDENCE"),
    ("UC_ADJUST", "include", "UC_REASON"), ("UC_CR_DECLINE", "include", "UC_REASON"),
    ("UC_CR_APPROVE", "include", "UC_CONSOLIDATE"), ("UC_CR_APPROVE", "include", "UC_DRAFT_INVOICE"),
    ("UC_CR_REOPEN", "include", "UC_CR_ASSESS"), ("UC_CR_FROM_MSG", "extend", "UC_CR_CREATE"),
    ("UC_CHAT", "include", "UC_NOTIFY"),
]


def use_case_diagram(scope: str) -> str:
    article = scope == "artigo"
    cases, actors, relations = ((ARTICLE_USE_CASES, ARTICLE_ACTORS, ARTICLE_RELATIONS) if article
                                else (FULL_USE_CASES, FULL_ACTORS, FULL_RELATIONS))
    title = "Diagrama de Casos de Uso" + (" (escopo do artigo)" if article else " (sistema completo)")
    lines = ["@startuml diagrama-casos-de-uso", STYLE, f"title Horbyta — {title}",
             "left to right direction", ""]
    lines += ['actor "Freelancer" as Freelancer', 'actor "Cliente" as Cliente']
    if not article:
        lines += ['actor "Administrador" as Administrador',
                  'actor "Sistema" as Sistema <<sistema>>', "Administrador -|> Freelancer"]
    lines += ["", 'rectangle "Horbyta" {']
    packages = defaultdict(list)
    for key, (name, package) in cases.items():
        packages[package].append((key, name.replace("\n", "\\n")))
    for package, items in packages.items():
        indent = "  "
        if package:
            lines.append(f'  package "{package}" {{')
            indent = "    "
        lines += [f'{indent}usecase "{label}" as {key}' for key, label in items]
        if package:
            lines.append("  }")
    lines += ["}", ""]
    # Associações sem seta; a ordem dos extremos fixa o lado: freelancer à esquerda,
    # cliente e sistema à direita.
    for actor, links in actors.items():
        for case in links:
            lines.append(f"{actor} -- {case}" if actor in {"Freelancer", "Administrador"}
                         else f"{case} -- {actor}")
    lines += [f"{a} ..> {b} : <<{kind}>>" for a, kind, b in relations]
    lines.append("@enduml")
    return "\n".join(lines) + "\n"


# ---------------------------------------------------------------------------
# Renderização
# ---------------------------------------------------------------------------
def docker_ready() -> bool:
    if not shutil.which("docker"):
        return False
    if subprocess.run(["docker", "image", "inspect", DOCKER_IMAGE], capture_output=True).returncode == 0:
        return True
    print(f"Baixando a imagem {DOCKER_IMAGE} (PlantUML com Graphviz) ...")
    return subprocess.run(["docker", "pull", "-q", DOCKER_IMAGE], capture_output=True).returncode == 0


def render(files: list[Path]) -> None:
    """Renderiza com Graphviz quando possível; senão, com o motor interno (smetana)."""
    output = files[0].parent
    names = [f.name for f in files]
    if shutil.which("plantuml") and shutil.which("dot"):
        command = ["plantuml"]
        print("Renderização: PlantUML local + Graphviz")
    elif docker_ready():
        command = ["docker", "run", "--rm", "-u", f"{os.getuid()}:{os.getgid()}",
                   "-e", "PLANTUML_LIMIT_SIZE=16384", "-v", f"{output}:/data", "-w", "/data", DOCKER_IMAGE]
        print(f"Renderização: Docker ({DOCKER_IMAGE}) com Graphviz")
    elif shutil.which("java"):
        jar = Path(os.environ.get("PLANTUML_JAR", ROOT / "target" / "tools" / f"plantuml-{PLANTUML_VERSION}.jar"))
        if not jar.exists():
            jar.parent.mkdir(parents=True, exist_ok=True)
            print(f"Baixando PlantUML {PLANTUML_VERSION} em {jar} ...")
            urllib.request.urlretrieve(PLANTUML_URL, jar)
        command = ["java", "-DPLANTUML_LIMIT_SIZE=16384", "-jar", str(jar), "-Playout=smetana"]
        print("Renderização: PlantUML (smetana, sem Graphviz)")
    else:
        print("Sem Java, Docker ou PlantUML: foram gerados apenas os .puml.")
        return
    env = dict(os.environ, PLANTUML_LIMIT_SIZE="16384")
    for fmt in ("svg", "png"):
        result = subprocess.run([*command, f"-t{fmt}", "-charset", "UTF-8", *names],
                                capture_output=True, text=True, cwd=output, env=env)
        if result.returncode != 0:
            sys.exit(f"PlantUML falhou ({fmt}):\n{result.stdout}\n{result.stderr}")


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--escopo", choices=["artigo", "completo"], default="artigo",
                        help="artigo: somente o que o artigo descreve (padrão); completo: sistema inteiro.")
    parser.add_argument("--database-url", help="Base migrada para o diagrama ER (somente leitura).")
    parser.add_argument("--no-render", action="store_true", help="Gerar somente os .puml.")
    parser.add_argument("--output", type=Path, help="Pasta de saída (padrão: docs/diagrams/<escopo>).")
    args = parser.parse_args()

    output = args.output or ROOT / "docs" / "diagrams" / args.escopo
    output.mkdir(parents=True, exist_ok=True)
    entities = parse_entities()
    diagrams = {
        "diagrama-casos-de-uso.puml": use_case_diagram(args.escopo),
        "diagrama-classes.puml": class_diagram(entities, args.escopo),
        "diagrama-er.puml": er_diagram(database_url(args.database_url), args.escopo),
    }
    if args.escopo == "completo":
        diagrams["diagrama-classes-servicos.puml"] = services_diagram(entities)
    files = []
    for name, content in diagrams.items():
        path = output / name
        path.write_text(content, encoding="utf-8")
        files.append(path)
    if not args.no_render:
        render(files)
    for path in files:
        produced = [path.name] + [path.with_suffix(f".{e}").name for e in ("svg", "png")
                                  if path.with_suffix(f".{e}").exists() and not args.no_render]
        print(f"{output.relative_to(ROOT) if output.is_relative_to(ROOT) else output}/: {', '.join(produced)}")


if __name__ == "__main__":
    main()
