# ──────────────────────────────────────────────────────────────────────────────
# Stage 1 – builder
# Installs all deps (including devDependencies) and compiles TypeScript.
# ──────────────────────────────────────────────────────────────────────────────
FROM node:20-alpine AS builder

WORKDIR /app

# Copy manifests first for better layer caching.
COPY package.json package-lock.json ./
RUN npm ci

# Copy source and prisma schema.
COPY . .

# Generate Prisma client and compile.
RUN npm run prisma:generate
RUN npm run build

# ──────────────────────────────────────────────────────────────────────────────
# Stage 2 – runner
# Installs production dependencies only and copies the compiled output.
# ──────────────────────────────────────────────────────────────────────────────
FROM node:20-alpine AS runner

# Install dumb-init for proper signal handling in Docker.
RUN apk add --no-cache dumb-init

WORKDIR /app

ENV NODE_ENV=production

# Copy manifests and install production deps only.
COPY package.json package-lock.json ./
RUN npm ci --omit=dev

# Copy compiled output from builder.
COPY --from=builder /app/dist ./dist

# Copy prisma schema so Prisma Client can resolve at runtime.
COPY --from=builder /app/prisma ./prisma
COPY --from=builder /app/node_modules/.prisma ./node_modules/.prisma

# Non-root user for security.
RUN addgroup -S appgroup && adduser -S appuser -G appgroup
USER appuser

EXPOSE 3001

# Use dumb-init so that Node handles SIGTERM correctly.
ENTRYPOINT ["dumb-init", "--"]
CMD ["node", "dist/main"]
