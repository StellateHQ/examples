# Partial Query Caching

This repo serves as a playground environment for us to hack on partial query caching. It contains the following parts:

1. A GraphQL API deployed to Vercel using a Next.js route handler and GraphQL yoga (inside the `app` directory).
2. A lambda function written in Rust that is also deployed to Vercel.

The lambda function (2) is the interesting part. Conceptually this should be the Stellate CDN layer. It takes the request it receives and proxies them through to the GraphQL API we set up in (1).

There even exists a very simple caching API in `api/cache.rs` powered by Vercel Edge Config that we can make use of to simulate caching.

The splitting logic resides in `api/split`.

## Open issues and questions

See the issues on this repository.
