# ContextKeeper build container
FROM docker.io/rustlang/rust:nightly-bookworm

# Install useful tools for development
RUN apt-get update && apt-get install -y \
    git \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Default command
CMD ["bash"]
