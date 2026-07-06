# Build stage
FROM node:lts-alpine as builder
# Set working directory
WORKDIR /app

# Skip husky git-hook installation (no .git in the build context)
ENV HUSKY=0

# Copy workspace config and sources
COPY ./package.json ./yarn.lock ./.yarnrc.yml ./tsconfig.json ./
COPY ./.yarn ./.yarn
COPY ./packages ./packages
COPY ./scripts/build.sh ./scripts/build.sh

# Install the workspace (shared layer across images)
RUN yarn install

# Compile this package + its workspace deps, then pack it
RUN ./scripts/build.sh packages/node

# Production stage
FROM node:lts-alpine

# Copy .tgz file from builder
COPY --from=builder /app/packages/node/app.tgz /app.tgz

# Install production dependencies
RUN apk add --no-cache tini curl git && \
  tar -xzvf /app.tgz --strip 1 && \
  rm /app.tgz && \
  yarn install --production && \
  yarn cache clean && \
  rm -rf /root/.npm /root/.cache

# Create ./.monitor directory and set permissions
RUN mkdir -p .monitor && \
  chown 1000:1000 .monitor

# Make the user not ROOT
USER 1000
# Fix timezone to UTC
ENV TZ=utc

# Set Entry point and command
ENTRYPOINT ["/sbin/tini", "--", "/bin/run"]
CMD ["-f","/app"]
