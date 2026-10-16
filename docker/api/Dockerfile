# syntax=docker/dockerfile:1@sha256:4edf897a3ffa55b89f906fc8cc78afdb3f1834cc9c7083565e611a8a7d5fe99e
# Stage 1: Install dependencies
# The Bun that package.json's packageManager pins, which CI installs too
FROM oven/bun:1.4.3@sha256:ec06c3b6cea04192ae6770c434f668ca41d343ad19fa6472216c7b48be39c598 AS builder

WORKDIR /app

# Copy workspace root files
COPY package.json bun.lock ./

# Copy every workspace's package.json, at its own path, for dependency resolution, and the patches
# the install applies (none is fine: a wildcard may match nothing when another source matches)
COPY --parents apps/*/package.json packages/*/package.json patches/* ./

RUN bun install --frozen-lockfile

# template:start
# Copy shared config package and the runtime helpers the API imports, which are workspaces here and
# installed packages in an app
COPY packages/config/ packages/config/
COPY packages/drizzle-utils/ packages/drizzle-utils/
# template:end

# Copy API source and migration files
COPY apps/api/src/ apps/api/src/
COPY apps/api/drizzle/ apps/api/drizzle/
COPY apps/api/drizzle.config.ts apps/api/drizzle.config.ts
COPY apps/api/tsconfig.json apps/api/tsconfig.json

# Stage 2: Runtime
FROM oven/bun:1.4.3-slim@sha256:14a223fa35747ec728bce300ef85888bc5f7958164d15bbba6a54b22b52a3288

WORKDIR /app

# Copy installed dependencies
COPY --from=builder /app/node_modules node_modules
COPY --from=builder /app/package.json package.json

# template:start
COPY --from=builder /app/packages/config packages/config
COPY --from=builder /app/packages/drizzle-utils packages/drizzle-utils
# template:end

# Copy API application
COPY --from=builder /app/apps/api/src apps/api/src
COPY --from=builder /app/apps/api/package.json apps/api/package.json
COPY --from=builder /app/apps/api/drizzle apps/api/drizzle

WORKDIR /app/apps/api

EXPOSE 3000

HEALTHCHECK --interval=10s --timeout=5s --retries=3 \
    CMD bun -e "fetch('http://localhost:3000/health').then(r => { if (!r.ok) process.exit(1) }).catch(() => process.exit(1))"

# Copy entrypoint script
COPY docker/api/entrypoint.sh /app/entrypoint.sh
RUN chmod +x /app/entrypoint.sh

CMD ["/app/entrypoint.sh"]
