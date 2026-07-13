# Stage 1: Build
FROM rust:1.80-slim AS builder
WORKDIR /usr/src/app

# Cài đặt các thư viện phụ thuộc C cần thiết để build (như OpenSSL)
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

COPY . .
RUN cargo build --release

# Stage 2: Run
FROM debian:bookworm-slim
WORKDIR /app

# Cài đặt OpenSSL runtime
RUN apt-get update && apt-get install -y libssl3 ca-certificates && rm -rf /var/lib/apt/lists/*

# Copy file thực thi từ builder sang (Tên project trong Cargo.toml là esp32)
COPY --from=builder /usr/src/app/target/release/esp32 /app/rust_engine

# Copy các file cấu hình phụ (nếu có)
COPY --from=builder /usr/src/app/.env* ./
COPY --from=builder /usr/src/app/building_graph.json ./

# Khởi chạy
CMD ["./rust_engine"]
