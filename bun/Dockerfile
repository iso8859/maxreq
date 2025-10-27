# Use the official Bun image (Alpine-based for smallest size)
FROM oven/bun:1-alpine

# Set working directory
WORKDIR /app

# Copy package files
COPY package.json bun.lockb* ./

# Install dependencies
RUN bun install --frozen-lockfile

# Copy application files
COPY . .

# Expose port
EXPOSE 8084

# Set environment variables
ENV PORT=8084
ENV DB_PATH=users.db

# Run the application
CMD ["bun", "run", "index.ts"]
