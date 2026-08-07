# Investment OS

AplicaÃ§Ã£o fullstack para gerenciamento de uma carteira de investimentos, desenvolvida em Rust como evoluÃ§Ã£o do projeto final do Bootcamp DIO.

O sistema permite criar uma conta, autenticar-se e administrar ativos em uma carteira individual. Cada usuÃ¡rio possui dados isolados, e nenhuma conta consegue visualizar, editar ou excluir ativos pertencentes a outra.


## Screenshots

### Login e autenticação

![Tela de login do Investment OS](docs/screenshots/login.png)

### Dashboard da carteira

![Dashboard do Investment OS](docs/screenshots/dashboard.png)

### Isolamento de carteiras por usuário

Cada conta acessa somente os ativos vinculados ao próprio usuário.

![Isolamento de carteiras no Investment OS](docs/screenshots/isolamento.png)

## Principais funcionalidades

- cadastro, login e logout;
- autenticaÃ§Ã£o com JWT armazenado em cookie `HttpOnly`;
- hash seguro de senhas;
- carteiras independentes por usuÃ¡rio;
- cadastro, listagem, ediÃ§Ã£o e exclusÃ£o de ativos;
- cÃ¡lculo automÃ¡tico do valor de cada posiÃ§Ã£o e do patrimÃ´nio total;
- validaÃ§Ã£o de nomes, valores, quantidades e duplicidades;
- API JSON protegida por autenticaÃ§Ã£o;
- dashboard responsivo com feedback visual e animaÃ§Ãµes;
- migrations SQL versionadas;
- testes automatizados de CRUD, resumo e isolamento entre usuÃ¡rios.

## Tecnologias

- Rust 2024
- Axum
- Askama
- SQLx
- PostgreSQL
- Tokio
- Docker Compose
- JWT
- Argon2 por meio da biblioteca `password-auth`
- HTML, CSS e JavaScript sem framework

## Arquitetura

```text
Navegador
   â”‚
   â”œâ”€â”€ pÃ¡ginas Askama e formulÃ¡rios
   â””â”€â”€ API JSON /api
            â”‚
            â–¼
       Rotas Axum
            â”‚
            â”œâ”€â”€ autenticaÃ§Ã£o JWT
            â”œâ”€â”€ validaÃ§Ãµes
            â””â”€â”€ Repository
                    â”‚
                    â–¼
               PostgreSQL
```

A coluna `user_id` relaciona cada ativo ao proprietÃ¡rio. As consultas de leitura, atualizaÃ§Ã£o e exclusÃ£o sempre incluem o usuÃ¡rio autenticado.

## SeguranÃ§a aplicada

- segredos carregados por variÃ¡veis de ambiente;
- `.env` ignorado pelo Git;
- senhas armazenadas apenas como hash;
- token com expiraÃ§Ã£o;
- cookie `HttpOnly` e `SameSite=Lax`;
- opÃ§Ã£o `Secure` para ambientes HTTPS;
- mensagens pÃºblicas sem detalhes internos do banco;
- proteÃ§Ã£o contra acesso cruzado entre carteiras;
- Ã­ndice Ãºnico de nome do ativo por usuÃ¡rio;
- validaÃ§Ã£o tambÃ©m na API, nÃ£o apenas no navegador.

> Este Ã© um projeto educacional e de portfÃ³lio. Para uma operaÃ§Ã£o financeira real ainda seriam necessÃ¡rios, entre outros itens, HTTPS obrigatÃ³rio, proteÃ§Ã£o CSRF dedicada, rate limiting, observabilidade, recuperaÃ§Ã£o de senha e auditoria de seguranÃ§a independente.

## PrÃ©-requisitos

- Git
- Rust e Cargo
- Docker Desktop
- `sqlx-cli`
- VS Code, opcional

InstalaÃ§Ã£o do SQLx CLI:

```powershell
cargo install sqlx-cli --no-default-features --features postgres
```

## ConfiguraÃ§Ã£o

Clone o repositÃ³rio e entre na pasta:

```powershell
git clone https://github.com/HanzSolo66/investment-os.git
cd investment-os
```

Crie o arquivo `.env` a partir do exemplo:

```powershell
Copy-Item .env.example .env
```

No `.env`, substitua os valores de exemplo por segredos longos e aleatÃ³rios.

Para gerar um segredo no PowerShell:

```powershell
$bytes = New-Object byte[] 48
$generator = [Security.Cryptography.RandomNumberGenerator]::Create()
$generator.GetBytes($bytes)
$generator.Dispose()
[Convert]::ToBase64String($bytes)
```

Em desenvolvimento local:

```env
COOKIE_SECURE=false
```

Em produÃ§Ã£o com HTTPS:

```env
COOKIE_SECURE=true
```

## Executar no Windows

O script abaixo abre o Docker Desktop quando necessÃ¡rio, inicia o PostgreSQL, espera o banco ficar pronto, aplica as migrations e inicia a aplicaÃ§Ã£o:

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\scripts\start-dev.ps1
```

Acesse:

```text
http://localhost:3000
```

## ExecuÃ§Ã£o manual

```powershell
docker compose up -d
sqlx migrate run
cargo run
```

## VerificaÃ§Ã£o de qualidade

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\scripts\quality-check.ps1
```

O script executa:

```text
sqlx migrate run
cargo fmt --check
cargo check
cargo test
```

## Testes cobertos

- criaÃ§Ã£o de ativo;
- listagem da carteira;
- atualizaÃ§Ã£o;
- exclusÃ£o;
- resumo do patrimÃ´nio;
- isolamento de leitura entre usuÃ¡rios;
- bloqueio de ediÃ§Ã£o e exclusÃ£o por outro usuÃ¡rio;
- rejeiÃ§Ã£o de valores invÃ¡lidos;
- rejeiÃ§Ã£o de nomes duplicados na mesma carteira.

## API

Todas as rotas abaixo exigem uma sessÃ£o vÃ¡lida.

| MÃ©todo | Rota | Finalidade |
|---|---|---|
| `GET` | `/api/assets` | Lista os ativos do usuÃ¡rio |
| `POST` | `/api/assets` | Cria um ativo |
| `PATCH` | `/api/assets` | Atualiza um ativo |
| `DELETE` | `/api/assets/{id}` | Exclui um ativo |
| `GET` | `/api/portfolio/summary` | Retorna o resumo da carteira |

Exemplo de criaÃ§Ã£o:

```json
{
  "name": "Bitcoin",
  "unit_value": 350000.0,
  "quantity": 0.01
}
```

## Estrutura principal

```text
migrations/              alteraÃ§Ãµes versionadas do banco
scripts/                 automaÃ§Ãµes de execuÃ§Ã£o e qualidade
src/auth/                autenticaÃ§Ã£o e sessÃ£o
src/routes/api.rs        API JSON e testes
src/routes/frontend.rs   pÃ¡ginas, formulÃ¡rios e dashboard
src/repository.rs        acesso ao PostgreSQL
templates/               interfaces Askama
compose.yml              PostgreSQL local
```

## DecisÃµes tÃ©cnicas

- O projeto mantÃ©m backend e frontend na mesma aplicaÃ§Ã£o Rust.
- SQLx faz validaÃ§Ã£o das consultas em tempo de compilaÃ§Ã£o.
- O banco impÃµe a unicidade do nome do ativo dentro de cada carteira.
- A autorizaÃ§Ã£o Ã© aplicada no acesso aos dados, nÃ£o apenas na interface.
- O dashboard nÃ£o depende de frameworks JavaScript externos.

## EvoluÃ§Ãµes futuras

- usar `NUMERIC/DECIMAL` em vez de `DOUBLE PRECISION` para valores monetÃ¡rios;
- adicionar categorias e tipos de ativos;
- histÃ³rico de movimentaÃ§Ãµes;
- rentabilidade e grÃ¡ficos;
- integraÃ§Ã£o com cotaÃ§Ãµes;
- recuperaÃ§Ã£o de senha;
- paginaÃ§Ã£o e filtros;
- testes end-to-end com Playwright;
- deploy com HTTPS e pipeline de CI.

## ApresentaÃ§Ã£o em entrevista

> Desenvolvi uma aplicaÃ§Ã£o fullstack de carteira de investimentos em Rust. Usei Axum no backend, Askama na interface, SQLx com PostgreSQL e Docker para o ambiente. Implementei autenticaÃ§Ã£o, CRUD completo, cÃ¡lculo do patrimÃ´nio e isolamento de dados entre usuÃ¡rios. TambÃ©m criei migrations, validaÃ§Ãµes e testes automatizados para impedir que uma conta visualize ou altere ativos de outra.

## Origem

Projeto desenvolvido a partir do repositÃ³rio-base do desafio da Digital Innovation One e expandido com interface prÃ³pria, autenticaÃ§Ã£o reforÃ§ada, isolamento de dados, validaÃ§Ãµes e automaÃ§Ãµes de desenvolvimento.

