# Horbyta

Aplicação web para gestão de projetos freelancers com controle formal de mudanças de escopo, desenvolvida como artefato do Trabalho de Conclusão de Curso (TCC II).

O sistema reúne clientes, projetos, tarefas, marcos, registro de tempo, faturas e pagamentos. Seu diferencial é o módulo de **solicitações de mudança** (change requests): uma alteração só pode ser aprovada pelo cliente depois que o freelancer registra o impacto em **horas, custo e prazo**, e a aprovação gera automaticamente a fatura suplementar e a nova data do marco afetado.

- **Backend:** Rust, Axum e SeaORM
- **Banco de dados:** PostgreSQL 16 ou superior
- **Frontend:** Vue 3, TypeScript e Quasar

Este documento explica como instalar e executar o sistema localmente para avaliação. A documentação técnica completa (arquitetura, implantação em servidor, testes e migrações) está em [docs/README-tecnico.md](docs/README-tecnico.md).

---

## 1. Requisitos

O sistema foi desenvolvido e testado em **Linux (Ubuntu/Debian)**. No Windows, recomenda-se usar o **WSL2 com Ubuntu** e seguir os mesmos passos dentro do terminal do Ubuntu.

| Ferramenta | Versão | Observação |
|---|---|---|
| Git | qualquer recente | para clonar o repositório |
| Rust | 1.97.1 | instalada automaticamente pelo `rustup` (arquivo `rust-toolchain.toml`) |
| Node.js | 22.12 ou superior | para o frontend |
| PostgreSQL | 16 ou superior | banco de dados |
| Compilador C, `pkg-config`, `curl` | — | necessários para compilar o backend |

### 1.1. Instalação das dependências no Ubuntu

```bash
sudo apt update
sudo apt install -y git build-essential pkg-config curl ca-certificates postgresql
```

Rust (aceite a instalação padrão e, ao final, abra um novo terminal):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Node.js 22 (por meio do `nvm`; abra um novo terminal após o primeiro comando):

```bash
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.40.1/install.sh | bash
nvm install 22
```

Confira as versões:

```bash
cargo --version
node --version
psql --version
```

---

## 2. Obter o código

```bash
git clone https://github.com/HormigaDev/horbyta.git
cd horbyta
```

---

## 3. Criar o banco de dados

Crie um usuário e um banco no PostgreSQL. O nome do banco **deve ser `horbyta_development`**, pois os dados de demonstração só podem ser carregados nesse banco.

```bash
sudo -u postgres psql -c "CREATE USER horbyta WITH PASSWORD 'horbyta123';"
sudo -u postgres createdb -O horbyta horbyta_development
```

---

## 4. Configurar o ambiente

Copie o arquivo de exemplo:

```bash
cp .env.template .env
```

Gere duas chaves aleatórias para as sessões:

```bash
openssl rand -hex 32
openssl rand -hex 32
```

Abra o arquivo `.env` e ajuste **somente** estas linhas (cole uma chave diferente em cada `JWT_SECRET`):

```env
DATABASE_URL=postgresql://horbyta:horbyta123@localhost:5432/horbyta_development
APP_ENV=development
FRONTEND_ORIGIN=http://localhost:9000
JWT_SECRET=cole-aqui-a-primeira-chave
JWT_SECRET_CLIENT=cole-aqui-a-segunda-chave
```

As demais variáveis podem permanecer como estão. Os anexos são gravados localmente em `data/storage/`.

---

## 5. Carregar o banco com dados de demonstração

O comando abaixo cria todas as tabelas e insere dados fictícios (projetos, clientes, tarefas, faturas e solicitações de mudança em todos os estados). A primeira execução compila o backend e pode levar alguns minutos.

```bash
cargo run --locked --bin seed
```

Para apagar tudo e recriar os dados de demonstração em outro momento:

```bash
cargo run --locked --bin seed -- --reset
```

> Se preferir começar com o sistema vazio, use `cargo run --locked --bin migrate` no lugar do `seed` e crie o administrador com `cargo run --locked --bin create-user`.

---

## 6. Executar o sistema

São necessários **dois terminais**, ambos na pasta do projeto.

**Terminal 1 – backend:**

```bash
cargo run --locked
```

Aguarde a mensagem `Server running at http(s)://127.0.0.1:8890`.

**Terminal 2 – frontend:**

```bash
cd frontend
npm ci
npm run dev
```

O navegador abrirá em **http://localhost:9000**. Use exatamente esse endereço (com `localhost`, e não `127.0.0.1`), pois o backend só aceita requisições dessa origem.

Para encerrar, pressione `Ctrl+C` em cada terminal.

---

## 7. Acessar o sistema

### 7.1. Área do freelancer

Acesse http://localhost:9000/login

- **E-mail:** `admin@horbyta.test`
- **Senha:** `Admin@123`

O idioma pode ser escolhido na tela de login, mas, depois de entrar, a conta de demonstração volta para o espanhol, que é a preferência salva nela. Para mudar para português, acesse no menu lateral **Configuraciones → Preferencias → Idioma → Português** e salve. A partir daí a interface fica em português.

### 7.2. Portal do cliente

O portal do cliente tem um endereço próprio, que inclui o identificador do freelancer. Para obtê-lo, execute:

```bash
psql postgresql://horbyta:horbyta123@localhost:5432/horbyta_development -Atc \
  "SELECT 'http://localhost:9000/client/u/' || id || '/login' FROM users WHERE email = 'admin@horbyta.test';"
```

Abra o endereço exibido e entre com uma das contas de cliente:

- **E-mail:** `cliente1@horbyta.test` até `cliente8@horbyta.test`
- **Senha:** `Cliente@123`

> Use uma janela anônima (ou outro navegador) para manter o freelancer e o cliente conectados ao mesmo tempo.

---

## 8. Roteiro sugerido para avaliar o fluxo de mudanças

Este roteiro percorre a principal regra de negócio do sistema: uma mudança de escopo só é liberada depois de avaliada pelo freelancer e aprovada pelo cliente.

1. **Freelancer:** acesse **Solicitações de mudança** e abra a solicitação **#00002 – Añadir buscador de contenidos** (status *Enviada*, projeto *Web de Norte Café*).
2. Clique em **Avaliar**. Informe as horas, o custo e os dias de impacto, escolha um marco e escreva a avaliação técnica. Salve.
   - Observe que, antes dessa etapa, o cliente não tem a opção de aprovar.
   - Um impacto em dias maior que zero exige a escolha de um marco com data.
3. **Cliente:** no portal, entre com `cliente2@horbyta.test` / `Cliente@123`. O painel mostra a decisão pendente. Abra a solicitação, confira as condições e clique em **Aprovar** (ou **Recusar**, informando o motivo).
4. **Freelancer:** abra de novo a solicitação #00002. Após a aprovação, aparecem:
   - a **fatura suplementar** gerada em rascunho (menu **Faturas**);
   - a **nova data do marco** escolhido;
   - o histórico imutável da negociação (aba **Histórico**).

Outros exemplos já prontos nos dados de demonstração:

- #00003, avaliada e aguardando decisão de `cliente3`;
- #00004, aprovada, com fatura e prazo aplicados;
- #00005, recusada e passível de reabertura pelo freelancer;
- #00006, implementada.

---

## 9. Documentação da API

Com o backend em execução, a documentação interativa (Swagger) fica disponível em http://localhost:8890/docs

---

## 10. Problemas comuns

| Sintoma | Causa provável e solução |
|---|---|
| `Could not connect to Postgres` | O PostgreSQL não está em execução ou a `DATABASE_URL` está incorreta. Verifique com `sudo systemctl status postgresql`. |
| `Ejecute cargo run --locked --bin migrate antes de iniciar` | O banco não foi preparado. Execute o passo 5. |
| O `seed` recusa a execução | O banco precisa se chamar `horbyta_development` e o `.env` deve ter `APP_ENV=development`. Se o banco já tiver dados, use `-- --reset`. |
| Login retorna erro 403 ou a tela não carrega dados | Acesse por `http://localhost:9000` (não `127.0.0.1`) e confira `FRONTEND_ORIGIN=http://localhost:9000` no `.env`. |
| Erro de compilação mencionando `cc` ou `linker` | Faltam ferramentas de compilação: `sudo apt install build-essential pkg-config`. |
| `npm ci` falha por versão do Node | Use o Node.js 22 ou superior (`nvm install 22`). |

---

## Licença

Distribuído sob a licença AGPL-3.0. Consulte o arquivo [LICENSE](LICENSE).
