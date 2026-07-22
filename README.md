# 🔍 Search Engine — Planning Document

> A reference document covering all project requirements and the agreed-upon roadmap.

---

## 📌 Overview

A search engine built from scratch using **Rust + React + Elasticsearch**, covering all the fundamental stages of a real search engine: crawling, indexing, searching, and displaying results.

---

## 🛠️ Tech Stack

| Component | Technology |
|---|---|
| Crawler | Rust |
| Backend API | Rust (Axum) |
| Database / Index | Elasticsearch |
| Frontend | React (Vite + TypeScript) |
| Local Infrastructure | Docker / docker-compose |

---

## 🗂️ Project Structure

```
search-engine/
│
├── README.md
├── docker-compose.yml
├── Cargo.toml                   ← Workspace combining crawler & backend
│
├── crawler/                     ← Rust
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs
│       ├── fetcher.rs           ← Fetch web pages
│       ├── parser.rs            ← Extract links and text content
│       ├── queue.rs             ← Manage the crawl queue
│       └── indexer.rs           ← Send data to Elasticsearch
│
├── backend/                     ← Rust (Axum)
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs
│       ├── routes/
│       │   └── search.rs        ← GET /search?q=...
│       ├── models/
│       │   └── document.rs
│       └── elasticsearch/
│           └── client.rs
│
├── frontend/                    ← React (Vite)
│   ├── package.json
│   ├── index.html
│   └── src/
│       ├── main.tsx
│       ├── App.tsx
│       ├── components/
│       │   ├── SearchBar.tsx
│       │   ├── SearchResults.tsx
│       │   └── ResultCard.tsx
│       ├── hooks/
│       │   └── useSearch.ts
│       └── types/
│           └── index.ts
│
└── elasticsearch/
    └── mappings/
        └── pages.json
```

---

## 🔗 Component Relationships

```
crawler  ──────────→  Elasticsearch (port 9200)
                              ↑
backend  ──────────→  Elasticsearch (port 9200)
   ↑
frontend ──────────→  backend (port 3000)
```

> - The crawler and backend never communicate directly with each other
> - The frontend never talks to Elasticsearch directly — only through the backend

---

## 📦 Required Dependencies

### Rust — Crawler
```toml
[dependencies]
reqwest  = { version = "0.11", features = ["json"] }
scraper  = "0.17"
tokio    = { version = "1", features = ["full"] }
```

### Rust — Backend
```toml
[dependencies]
axum        = "0.7"
tokio       = { version = "1", features = ["full"] }
serde       = { version = "1", features = ["derive"] }
serde_json  = "1"
reqwest     = { version = "0.11", features = ["json"] }
tower-http  = { version = "0.5", features = ["cors"] }
thiserror   = "1"
anyhow      = "1"
```

### React — Frontend
```
axios
```

---

## 🗺️ Build Phases

### Phase 1 — Crawler 🕷️

| # | Step | Tools |
|---|---|---|
| 1 | Set up Rust project | `cargo new crawler` |
| 2 | Fetch pages via HTTP | `reqwest`, `tokio` |
| 3 | Parse HTML and extract links & text | `scraper` |
| 4 | Manage crawl queue and visited URLs | `VecDeque`, `HashSet` |
| 5 | Respect `robots.txt` | `texting` crate |
| 6 | Rate limiting to avoid bans | `tokio::time::sleep` |
| 7 | Concurrent crawling | `tokio`, `Arc`, `Mutex` |

---

### Phase 2 — Indexing with Elasticsearch 🗄️

| # | Step | Tools |
|---|---|---|
| 1 | Run Elasticsearch locally | `Docker` |
| 2 | Design the index schema | ES Mapping |
| 3 | Send page content from the crawler | REST API / `elasticsearch` crate |
| 4 | Text analysis for partial search | ES Analyzers |
| 5 | Periodic index updates | `tokio` scheduled tasks |

---

### Phase 3 — Backend API with Axum ⚙️

| # | Step | Tools |
|---|---|---|
| 1 | Set up Rust project | `cargo new backend` |
| 2 | Create search endpoint `GET /search?q=` | `axum` routing |
| 3 | Connect to Elasticsearch and forward queries | `reqwest` |
| 4 | Format results as clean JSON | `serde`, `serde_json` |
| 5 | Configure CORS | `tower-http` |
| 6 | Error handling | `thiserror`, `anyhow` |

---

### Phase 4 — Frontend with React 🖥️

| # | Step | Tools |
|---|---|---|
| 1 | Create React project | `Vite` |
| 2 | Build search page (input + button) | React, Tailwind |
| 3 | Call the backend API | `axios` |
| 4 | Display results (title + URL + description) | React components |
| 5 | Pagination | React state |
| 6 | Loading and error states | React state |

---

### Phase 5 — Ranking 📊

| # | Step | Tools |
|---|---|---|
| 1 | TF-IDF — rank by keyword frequency | Elasticsearch built-in |
| 2 | Page Score — weight pages by inbound links | Custom logic in Rust |
| 3 | Freshness — prefer more recently crawled pages | timestamp field in index |

---

## ⏱️ Suggested Timeline

```
Week 1-2  →  Basic crawler (fetch + extract)
Week 3    →  Connect crawler to Elasticsearch
Week 4    →  Build backend API with Axum
Week 5    →  Build React frontend
Week 6+   →  Improve ranking and performance
```

---

## ✅ Prerequisites

```bash
rustc --version      # Rust installed
cargo --version      # Cargo installed
node --version       # Node.js installed
docker --version     # Docker installed
```

---

## 🚀 Running Elasticsearch Locally

```bash
docker-compose up -d

# Verify it's running
curl http://localhost:9200
```

---

## 📋 Important Notes

- This is a **portfolio-scale** project — a real search engine in concept, but on a small scale
- A production search engine (like Google) requires thousands of engineers and billions in infrastructure
- The crawler must respect `robots.txt` to avoid being banned
- Never hammer a single site with too many requests at once (Rate Limiting)
