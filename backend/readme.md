# Tài Liệu Backend (Actix-Web)

Đây là mã nguồn backend cho dự án, được xây dựng bằng framework **Actix-web** trong Rust. Dự án hỗ trợ kết nối WebSocket theo thời gian thực và cung cấp các RESTful API cho ứng dụng.

## Cấu Trúc Dự Án

Cấu trúc thư mục hiện tại của backend:

```text
backend/
├── src/
│   ├── lib.rs       # Điểm vào chính của thư viện, xuất các module
│   └── main.rs      # Điểm vào của ứng dụng (chạy server)
├── tests/
│   ├── api_tests.rs # Các bài kiểm thử tích hợp cho API
│   └── common_tests.rs # Các bài kiểm thử dùng chung / tiện ích
├── Cargo.toml       # Quản lý dependencies và cấu hình Rust
└── rustfmt.toml     # Lựa chọn định dạng mã nguồn bộ định dạng Rust
```

* **`src/main.rs`**: Khởi tạo cấu hình và chạy HTTP server với Actix-web.
* **`src/lib.rs`**: Khai báo và cấu hình các module chức năng chính (controllers, database, services, websocket, v.v.).

## Kết Nối WebSocket Thời Gian Thực (`/ws`)

Hệ thống hỗ trợ giao tiếp hai chiều theo thời gian thực (real-time) thông qua WebSocket. 
- Tính năng này được triển khai sử dụng thư viện **`actix-ws`** thay vì thư viện cũ `actix-web-actors`.
- Để quản lý và phân phối tin nhắn đến nhiều clients cùng một lúc một cách an toàn và tối ưu, hệ thống sử dụng **`tokio::sync::broadcast`**. Kênh truyền thông (channel) này cho phép phát (broadcast) tín hiệu trạng thái và các sự kiện từ server đến tất cả các kết nối WebSocket đang mở.

## Kiểm Thử (Testing)

Dự án được tổ chức và kiểm thử một cách bài bản nhằm đảm bảo chất lượng và độ ổn định của hệ thống:

- **Unit tests (Kiểm thử đơn vị)**: Cùng nằm trong thư mục `src/`, thường cấu hình bằng `#![cfg(test)]` cạnh mã nguồn gốc.
- **Integration tests (Kiểm thử tích hợp)**: Được đặt gọn gàng trong thư mục `tests/`. Nó giúp kiểm định quy trình hoạt động giữa các modules với nhau một cách độc lập:
  - `tests/api_tests.rs`: Kiểm thử trực tiếp lên các route REST API và các logic yêu cầu/phản hồi (Request/Response).
  - `tests/common_tests.rs`: Chứa các kiểm thử liên quan đến logic hoặc các tiện ích dùng chung xuyên suốt backend.

### Cách Chạy Kiểm Thử

Để chạy toàn bộ các bài unit tests và integration tests, bạn chỉ cần mở terminal, di chuyển vào thư mục `backend/` và chạy lệnh sau:

```bash
cargo test
```

Lệnh này sẽ tự động thu thập và kiểm tra tất cả các tệp có đuôi `.rs` trong môi trường kiểm thử (cả bên trong `src/` và `tests/`). Mặc định thông tin log khi pass sẽ bị ẩn đi, bạn có thể chạy `cargo test -- --nocapture` nếu muốn in kết quả log chi tiết.
