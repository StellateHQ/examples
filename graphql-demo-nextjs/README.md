# GraphQL Demo (Next.js)

A stateful GraphQL endpoint generator.

This project is a Next.js port of the original Cloudflare Worker “graphql-demo,” offering both a Redis-backed server mode and a client-only localStorage mode.

---

## Features

- **Admin endpoint** at `/api/admin` to create new slug-based GraphQL endpoints.
- **Token generator** at `/api/token` to produce long-lived JWTs.
- **Per-slug endpoint** at `/api/[slug]` serving GraphQL Playground (HTML) and JSON queries.
- **Home page** lists all existing endpoints (Redis mode) and explains how to use the demo.

---

## Getting Started

1. **Clone** this repo and install dependencies:

   ```bash
   git clone <your-repo-url>
   cd graphql-demo-nextjs
   npm install
   ```

2. **Environment**

   Copy `.env.local.example` → `.env.local` and fill in:

   ```dotenv
   UPSTASH_REDIS_REST_URL=https://<your-upstash-id>.upstash.io
   UPSTASH_REDIS_REST_TOKEN=<your-upstash-rest-token>
   TOKEN_SECRET=<your-jwt-secret>
   ALLOWED_ORIGINS=http://localhost:3000
   ```

3. **Run**

   ```bash
   npm run dev
   ```

   - Open `http://localhost:3000` for home page
   - `/api/token`, `/api/admin`, `/api/<slug>` are live

4. **Deploy**

   Push to GitHub, import into Vercel, set the same env vars, and deploy.

---

## Usage Overview

1. **Generate Token**

   ```bash
   curl http://localhost:3000/api/token
   # → { "token": "<your-jwt-here>" }
   ```

2. **Create Endpoint**

   - Open GraphQL Playground at `http://localhost:3000/api/admin`
   - Click **HTTP HEADERS** and add:

     ```json
     { "Authorization": "bearer <your-jwt-here>" }
     ```

   - Run:

     ```graphql
     mutation {
       createEndpoint(slug: "my-first-slug")
     }
     ```

3. **Query Your Slug**

   - Playground: `http://localhost:3000/api/my-first-slug`
   - Or via `curl`:

     ```bash
     curl -X POST http://localhost:3000/api/my-first-slug \
       -H "Content-Type: application/json" \
       -d '{"query":"query { todos { id title } }"}'
     ```

---
