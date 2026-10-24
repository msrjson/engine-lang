# Engine Forge (`.eng`)
> The First Laravel-Inspired Full-Stack Web Framework for Engine Lang

[![Standard](https://img.shields.io/badge/Standard-ENG--SPEC--03.1-blue.svg)](https://engine-lang.org)
[![Storage](https://img.shields.io/badge/Storage-ENG--SLS--01.1-purple.svg)](https://engine-lang.org)
[![SMT](https://img.shields.io/badge/Verification-Z3%20SMT-success.svg)](https://engine-lang.org)
[![License](https://img.shields.io/badge/License-Apache%202.0-green.svg)](https://www.apache.org/licenses/LICENSE-2.0)

**Engine Forge** is an expressive, full-stack web framework written natively in **Engine Lang (`.eng`)**. It combines the developer ergonomics, elegance, and conventions of **Laravel** with the zero-overhead architecture of **Single-Level Store (SLS)** and **Formal Verification via Z3 SMT**.

---

## 1. Paradigm Shift: Laravel vs. Engine Forge

| Architectural Layer | Legacy Laravel (PHP + SQL) | Engine Forge (Engine Lang) |
|---|---|---|
| **Persistence (ORM)** | Eloquent ORM + PDO + External SQL (MySQL/Postgres) | **Eloquent on Silicon**: `@persistent entity` mapped directly into `engine.db` with zero SQL parsing and zero object hydration. |
| **Validation** | FormRequests with runtime strings (`'email' => 'required\|email'`) | **Z3 SMT Proofs**: Refinement types proven mathematically at compile-time (`type Email = Text where len(value) >= 5 && is_alphanumeric(value)`). |
| **State Mutability** | Database transactions (`DB::transaction`) over TCP sockets | **Silicon Durability**: `atomic { ... }` blocks emitting hardware cache-line writeback (`CLWB`) and serializing fence (`SFENCE`). |
| **HTTP Boundary** | Web servers (Nginx/Apache) + PHP-FPM pool | Native `@boundary` HTTP gateway running directly on port `8090`. |
| **Authorization** | Sanctum bearer tokens + Gate/Policy classes | UUIDv7 cryptographic sessions + `@TenantGuard` compile-time tenant scoping. |
| **Queues & Jobs** | Redis / RabbitMQ / SQS workers with serialization | Native **Planetary Causal Mesh** with Delta-CRDT vector clocks. |
| **CLI Runner** | `php artisan` script interpreter | Native compiled console runner executed via `enginec run`. |

---

## 2. Directory Structure

```
framework/
├── Engine.toml                     # Framework manifest & compiler parameters
├── README.md                       # Architectural guide & documentation
├── src/
│   ├── mod.eng                     # Framework::Core (Kernel & initialization)
│   ├── http/
│   │   ├── request.eng             # Framework::Http::Request (HttpRequest struct & verbs)
│   │   ├── response.eng            # Framework::Http::Response (JSON & HTML envelopes)
│   │   ├── router.eng              # Framework::Http::Router (Route registration & matcher)
│   │   └── middleware.eng          # Framework::Http::Middleware (AuthGuard & TenantGuard)
│   ├── sls/
│   │   ├── model.eng               # Framework::Sls::Model (Persistent entity lifecycle)
│   │   ├── repository.eng          # Framework::Sls::Repository (Zero-copy in-memory query)
│   │   └── contracts.eng           # Framework::Sls::Contracts (SMT formal refinement types)
│   ├── auth/
│   │   ├── session.eng             # Framework::Auth::Session (UUIDv7 AuthSession entity)
│   │   ├── hasher.eng              # Framework::Auth::Hasher (SHA-256 password hashing)
│   │   └── gate.eng                # Framework::Auth::Gate (RBAC policies & permissions)
│   ├── events/
│   │   ├── dispatcher.eng          # Framework::Events::Dispatcher (Causal event bus)
│   │   └── listener.eng            # Framework::Events::Listener (Async pipeline handlers)
│   └── artisan/
│       └── console.eng             # Framework::Artisan::Console (Command runner & toolchain)
└── app/                            # Canonical Demonstration Web Application
    ├── bootstrap.eng               # Main entry point & simulation runner
    ├── models/
    │   ├── user.eng                # App::Models::User (Persistent User entity)
    │   └── post.eng                # App::Models::Post (Persistent Post with 1536-dim vector)
    ├── controllers/
    │   ├── auth_controller.eng     # App::Controllers::Auth (Login, Register, Logout)
    │   └── post_controller.eng     # App::Controllers::Post (CRUD & AI Vector Search)
    └── routes/
        └── web.eng                 # App::Routes::Web (@boundary HTTP ingress)
```

---

## 3. Quick Start & In-Container Verification

In accordance with Engine Lang Team Policy (Rule 0.1), all verification must be performed inside the `engine-genesis-node` Docker container:

### 3.1 Formally Verify All Framework Modules via Z3 SMT
```bash
docker exec engine-genesis-node sh -lc '
for f in $(find /app/framework -name "*.eng" | sort); do
    enginec check "$f"
done
'
```

### 3.2 Execute the Application Bootstrap
```bash
docker exec engine-genesis-node enginec run /app/framework/app/bootstrap.eng --db /app/data/engine.db
```

### 3.3 Inspect Single-Level Store Container
```bash
docker exec engine-genesis-node enginec inspect /app/data/engine.db
```

---

## 4. Code Examples

### 4.1 Eloquent on Silicon: SMT Invariants & Hardware Durability
```engine
module App::Models::Article;

type ArticleSlug = Text where len(value) >= 3 && is_alphanumeric(value);
type ArticleViews = Int where value >= 0;

@persistent
@TenantGuard(strict: true)
entity Article {
    id: UUIDv7,
    title: Text,
    slug: ArticleSlug,
    views: ArticleViews,

    invariant {
        self.views >= 0 &&
        len(self.slug) >= 3
    }
}

pub action increment_views(article: &mut Article) -> Result<ArticleViews, Text> {
    atomic {
        article.views += 1;
        // Hardware persistence flush emitted: CLWB [article] + SFENCE
    }
    return Ok(article.views);
}
```

### 4.2 Declarative HTTP `@boundary` Routing
```engine
module App::Routes::Web;

boundary WebGateway {
    @locality(public: true)
    route GET "/" -> handle_welcome;

    @locality(public: true)
    route GET "/api/v1/posts" -> handle_get_posts;

    @locality(public: true)
    route POST "/api/v1/posts" -> handle_create_post;

    @locality(public: true)
    route POST "/api/v1/auth/login" -> handle_login;
}
```

---

## 5. Production Benchmark: Engine Forge vs. Laravel 13

Verified in isolated Docker containers with 1,000 requests per scenario across 25 concurrent workers:

| Request Pattern | Engine Forge | Laravel 13 (PHP 8.3 OPcache) | Speedup Factor |
|---|:---:|:---:|:---:|
| **Server-Rendered HTML (SSR)** | **3,820.7 req/s** (5.79 ms P50) | 25.9 req/s (731.81 ms P50) | **147.3x** |
| **Simple Read API (JSON)** | **3,947.9 req/s** (5.57 ms P50) | 32.9 req/s (613.04 ms P50) | **120.0x** |
| **Complex Graph API (JSON)** | **3,465.2 req/s** (6.33 ms P50) | 27.8 req/s (533.78 ms P50) | **124.8x** |
| **Auth Session Gate (POST)** | **4,275.3 req/s** (5.26 ms P50) | 36.2 req/s (618.87 ms P50) | **118.0x** |
| **Transactional Mutation (POST)** | **5,918.1 req/s** (3.68 ms P50) | 27.2 req/s (720.40 ms P50) | **217.2x** |
| **Baseline RAM Footprint** | **840 KiB** | **31.9 MiB** | **38.9x lower** |

*For complete technical analysis and methodology, see [`docs/PRODUCTION_BENCHMARK.md`](docs/PRODUCTION_BENCHMARK.md).*

---

## 6. Authorship & Intellectual Sovereignty

Engine Forge is part of the decentralized **Engine Lang** ecosystem, developed under the literary pseudonym **Lysander Vance** and maintained by **The Vance Protocol Group / Engine Lang Core Working Group**.

Released freely under the **Apache License 2.0**.

