# Frontend

This directory contains the read-only frontend for World ID Rootcast.

The frontend currently includes:

- a dark landing page that explains the trust flow
- an operations dashboard for the latest root update and replication targets
- a small typed API client for the backend read model
- reusable status components for target cards, stage banners, and recent updates

The frontend stays intentionally small. It uses Next.js app routes, server-side
data fetching, lightweight auto-refresh, and a custom dark theme rather than a
large component library or client-state framework.
