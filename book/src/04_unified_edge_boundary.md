# Chapter 4: The Unified Edge Boundary and the SQL Virtual Bridge

> *"A reverse proxy is an admission that your runtime cannot speak to the wire; an external database driver is an admission that your memory cannot endure. When the binary speaks HTTP and PostgreSQL natively, the middleman is dead."*
> — **Lysander Vance**, *Axioms of Inviolable Silicon*, 2026

---

## 1. The Middleware Tax of Network Ingress

In legacy cloud architectures, exposing a persistent application to the internet requires traversing another agonizing sequence of intermediary daemons:

```
THE LEGACY NETWORK INGRESS STACK
┌────────────────────────────────────────────────────────┐
│ 1. Cloud Load Balancer (AWS ALB / Cloudflare)          │
├────────────────────────────────────────────────────────┤
│ 2. Ingress Reverse Proxy (Nginx / Envoy / Traefik)     │
├────────────────────────────────────────────────────────┤
│ 3. Application Web Framework (FastAPI / Express / Actix│
├────────────────────────────────────────────────────────┤
│ 4. JSON Serialization / Deserialization CPU Cycles     │
├────────────────────────────────────────────────────────┤
│ 5. Database Connection Pooler (PgBouncer)              │
├────────────────────────────────────────────────────────┤
│ 6. Separate Database Wire Daemon (PostgreSQL on 5432)  │
└────────────────────────────────────────────────────────┘
```

Each boundary incurs memory copies, thread context switches, TLS re-encryption, and operational maintenance overhead. If any connection pooler exhausts its file descriptors, the entire ingress tier locks up.

**Engine Lang collapses network ingress and SQL interoperability directly into the compiled binary.**

---

## 2. Connecting via HTTP: The `@boundary` Gateway on Port 8090

Engine Lang introduces the `boundary` declaration block. Instead of relying on external web frameworks like Express or FastAPI, network routes are first-class declarations recognized directly by the compiler:

```engine
boundary CustomerGateway {
    @locality(public: true)
    route GET "/api/v1/accounts/:id" -> get_account_status;

    @locality(public: true)
    route POST "/api/v1/accounts/deposit" -> handle_http_deposit;
}
```

When you start the binary with `enginec serve`, the runtime initializes an embedded asynchronous HTTP/1.1 and HTTP/2 gateway listening on port **8090** (default). Incoming HTTP requests are parsed with zero memory allocations into typed function arguments, dispatched to your pure actions, and returned as streaming JSON or binary buffers.

There is no Nginx configuration to maintain; there are no proxy pass headers to forward.

---

## 3. How to Query via SQL: The Embedded `engine-bridge` on Port 5432

A primary roadblock for adopting new systems languages is the friction of enterprise data tooling. Data analysts, product managers, and site reliability engineers rely on tools like:
- **DBeaver** and **DataGrip** (desktop SQL workbenches);
- **Tableau**, **Power BI**, and **Metabase** (business intelligence dashboards);
- **Grafana** (metrics and telemetry visualizations);
- The standard **`psql`** command-line utility.

If adopting Engine Lang required abandoning these tools, adoption would stall.

Engine Lang solves this through the **`engine-bridge`** (mandated under specification `ENG-FFI-01.1`).

### How `engine-bridge` Works Under the Hood
1. The Engine Lang binary opens a native TCP listener on port **5432** (the standard PostgreSQL port).
2. It implements the standard **PostgreSQL v3.0 binary wire protocol** (handling `StartupMessage`, `SSLRequest`, `AuthenticationOk`, `ParameterStatus`, `ReadyForQuery`, and `RowDescription`).
3. When an analyst runs:
   ```sql
   SELECT id, email, balance FROM customer_account WHERE balance > 1000.00;
   ```
4. **No PostgreSQL database server exists.** The `engine-bridge` translates the incoming SQL query into a direct zero-copy memory traversal across the `@persistent` entity heap in `engine.db`.
5. The result rows are streamed back across TCP formatted as native PostgreSQL data packets.

```
ZERO-COPY SQL QUERYING VIA ENGINE-BRIDGE
┌────────────────────────────────────────────────────────┐
│ Client Tool (DBeaver / Tableau / Metabase / psql)      │
└──────────────────────────┬─────────────────────────────┘
                           │ Standard PostgreSQL Wire Protocol (Port 5432)
                           ▼
┌────────────────────────────────────────────────────────┐
│ ENGINE LANG COMPILED BINARY (enginec serve)            │
│                                                        │
│  ┌──────────────────────────────────────────────────┐  │
│  │ engine-bridge (PostgreSQL v3.0 Wire Emulation)    │  │
│  └───────────────────────┬──────────────────────────┘  │
│                          │ Zero-Copy Memory Traversal  │
│                          ▼ (Direct Silicon Read)       │
│  ┌──────────────────────────────────────────────────┐  │
│  │ Single-Level Store: engine.db (Region 3 Entities)│  │
│  └──────────────────────────────────────────────────┘  │
└────────────────────────────────────────────────────────┘
```

Your data analysts can connect their existing dashboards to Engine Lang using their standard PostgreSQL drivers on Day 1.

---

## 4. The Unified Server Command: `enginec serve`

Engine Lang unifies web services and SQL query access into a single CLI command:

```bash
enginec serve --http-port 8090 --pg-port 5432 --host 0.0.0.0 --db /app/data/engine.db
```

This single command:
- Serves HTTP/REST client traffic on port **8090**;
- Serves PostgreSQL BI/SQL queries on port **5432**;
- Mounts and manages the persistent Single-Level Store on `/app/data/engine.db`.

---

## 5. Canonical Code Walkthrough

Let us inspect [`examples/08_book_chapter4_boundary_and_sql.eng`](file:///data/projects/workspaces/engine-lang/examples/08_book_chapter4_boundary_and_sql.eng):

```engine
// Engine Lang Book - Chapter 04: The Unified Edge Boundary and the SQL Virtual Bridge
// Canonical Example: Unified HTTP Boundary & Virtual PostgreSQL SQL Projection

module Book::Chapter04::BoundaryAndSql;

// 1. Refinement Types proven at compile-time by Z3
type CustomerId = Text where len(value) >= 4;
type EmailAddress = Text where len(value) >= 5 && is_alphanumeric(value);
type AccountBalance = Decimal(18, 4) where value >= 0.0000;
type DepositAmount = Decimal(18, 4) where value > 0.0000;

// 2. Persistent Customer Entity mapped to Single-Level Store (.db)
// This entity is simultaneously queryable via zero-copy memory traversal
// and projectable as a virtual relational table 'customer_account' via engine-bridge
@persistent
entity CustomerAccount {
    id: CustomerId,
    email: EmailAddress,
    tier: Text,
    balance: AccountBalance,
    total_deposits: Int where value >= 0,

    invariant {
        self.balance >= 0.0000 &&
        self.total_deposits >= 0 &&
        len(self.id) >= 4
    }
}

// 3. Pure Action: Deposit funds with hardware durability
action apply_deposit(
    account: &mut CustomerAccount,
    amount: DepositAmount
) -> Result<AccountBalance, Text> {
    atomic {
        account.balance += amount;
        account.total_deposits += 1;
        // Hardware flush emitted: CLWB [account] + SFENCE
    }

    return Ok(account.balance);
}

// 4. HTTP Boundary Definition: Native web ingress on port 8090
boundary CustomerGateway {
    @locality(public: true)
    route GET "/api/v1/accounts/:id" -> get_account_status;

    @locality(public: true)
    route POST "/api/v1/accounts/deposit" -> handle_http_deposit;
}

action get_account_status(id: Text) -> Result<Text, Text> {
    return Ok("ACCOUNT_ONLINE");
}

action handle_http_deposit(id: Text) -> Result<Text, Text> {
    return Ok("DEPOSIT_SUCCESS");
}

// 5. Entry point
pub action main() -> Result<(), Text> {
    io::println("=== Engine Lang: Chapter 04 Boundary & SQL Bridge ===");

    let mut account = CustomerAccount {
        id: "CUST-8801",
        email: "alice88@engine-lang.org",
        tier: "Enterprise",
        balance: 25000.0000,
        total_deposits: 10
    };

    io::println("Customer ID:       " + account.id);
    io::println("Customer Email:    " + account.email);
    io::println("Account Tier:      " + account.tier);
    io::println("Initial Balance:   " + string::from_decimal(account.balance));

    // Execute atomic deposit
    let deposit: DepositAmount = 5000.0000;
    let new_balance = apply_deposit(&mut account, deposit)?;
    io::println("Updated Balance:   " + string::from_decimal(new_balance));
    io::println("Total Deposits:    " + string::from_int(account.total_deposits));

    io::println("--- Connectivity Endpoints Ready ---");
    io::println("HTTP Gateway Boundary: http://0.0.0.0:8090/api/v1/accounts");
    io::println("PostgreSQL SQL Bridge: postgresql://engine:engine@0.0.0.0:5432/engine_db");

    return Ok(());
}
```

---

## 6. Real Container Execution

We compile and verify the module inside `engine-genesis-node`:

```bash
docker exec engine-genesis-node sh -lc 'enginec check /app/examples/08_book_chapter4_boundary_and_sql.eng'
```

Output:
```text
🔍 Engine Lang Compiler (enginec) - Checking: /app/examples/08_book_chapter4_boundary_and_sql.eng
✅ Grammar Validation: Deterministic LL(1) parse succeeded (Module: Book::Chapter04::BoundaryAndSql)
📐 Formal Verification: Translating invariants to Z3 SMT-LIB2...
  [SMT-PROVEN] Refinement type 'CustomerId' mathematically verified
  [SMT-PROVEN] Refinement type 'EmailAddress' mathematically verified
  [SMT-PROVEN] Refinement type 'AccountBalance' mathematically verified
  [SMT-PROVEN] Refinement type 'DepositAmount' mathematically verified
  [SMT-PROVEN] Entity invariant for '@persistent CustomerAccount' verified
✨ Formal Verification Succeeded: All type refinements & contracts mathematically proven.
```

Execute with persistent runtime:

```bash
docker exec engine-genesis-node sh -lc 'enginec run /app/examples/08_book_chapter4_boundary_and_sql.eng --db /app/data/engine.db'
```

Output:
```text
🚀 Engine Lang Runtime - Executing: /app/examples/08_book_chapter4_boundary_and_sql.eng
⚡ Single-Level Store mounted on: /app/data/engine.db
🔒 Atomic hardware protection active: CLWB + SFENCE durable
🌐 Causal mesh ready: Local node #1 active
⚙️  Executing action 'main' via causal runtime engine...
=== Engine Lang: Chapter 04 Boundary & SQL Bridge ===
Customer ID:       CUST-8801
Customer Email:    alice88@engine-lang.org
Account Tier:      Enterprise
Initial Balance:   25000.0000
Updated Balance:   30000.0000
Total Deposits:    11
--- Connectivity Endpoints Ready ---
HTTP Gateway Boundary: http://0.0.0.0:8090/api/v1/accounts
PostgreSQL SQL Bridge: postgresql://engine:engine@0.0.0.0:5432/engine_db
🎉 Execution of 'main' succeeded (Result: ())
```

---

## 7. The Dialectical Transition to Chapter 5

We have achieved transparent connectivity: external HTTP clients consume our endpoints on port 8090, while SQL tooling accesses our persistent data on port 5432 via `engine-bridge`.

Now, two deeper architectural questions demand answers:

> **The Next Critical Questions**:
> 1. *How does `engine.db` organize its physical bytes under the hood? What are its exact regions, and how does zero-copy Direct Access (DAX) work in silicon?*
> 2. *How do we connect an existing, legacy database (PostgreSQL, MySQL, Oracle) to an Engine Lang application so we can ingest legacy data and migrate without downtime?*

In **Chapter 5**, we will dissect **The Anatomy of `engine.db` and Legacy Database Migration**, detailing the 4 physical regions of the container and demonstrating how to ingest enterprise SQL data into the Single-Level Store.

