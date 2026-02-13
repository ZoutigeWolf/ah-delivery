# --- Builder Stage ---
# Explicitly use Bookworm to match the runtime GLIBC version
FROM rust:1-bookworm AS builder

# Install Tesseract and Leptonica development headers
RUN apt-get update && apt-get install -y \
    libtesseract-dev \
    libleptonica-dev \
    clang \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY . .

# Help bindgen find headers on this specific architecture
ENV BINDGEN_EXTRA_CLANG_ARGS="-I/usr/include"

RUN cargo build --release

# --- Final Runtime Stage ---
FROM debian:bookworm-slim

# Install the matching Tesseract runtime and language data
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libtesseract5 \
    tesseract-ocr \
    tesseract-ocr-nld \
    tesseract-ocr-eng \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binary from the builder
COPY --from=builder /app/target/release/ah-delivery .

# Standard Tesseract data path for Debian Bookworm
ENV TESSDATA_PREFIX=/usr/share/tesseract-ocr/5/tessdata/

EXPOSE 3069

ENTRYPOINT ["./ah-delivery"]