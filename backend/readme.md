# Tài Liệu Backend (Actix-Web)

Đây là mã nguồn backend cho dự án, được xây dựng bằng framework **Actix-web** trong Rust. Dự án hỗ trợ kết nối WebSocket theo thời gian thực và cung cấp các RESTful API cho ứng dụng.

## Cấu Trúc Dự Án

Cấu trúc thư mục hiện tại của backend:

```text
backend/
├── src/
│   ├── lib.rs       # Điểm vào chính của thư viện, xuất các module
│   └── main.rs      # Điểm vào của ứng dụng (chạy server)
├── docs/            # Tất cả các tài liệu luồng nghiệp vụ nằm riêng biệt
├── tests/
│   ├── api_tests.rs # Các bài kiểm thử tích hợp cho API
│   └── common_tests.rs # Các bài kiểm thử dùng chung / tiện ích
├── building_graph.json # Tệp tin định nghĩa sơ đồ toà nhà (Nodes, Edges, Exits)
├── Cargo.toml       # Quản lý dependencies và cấu hình Rust
└── rustfmt.toml     # Lựa chọn định dạng mã nguồn bộ định dạng Rust
```

* **`building_graph.json`**: Cấu hình bản đồ không gian của tòa nhà. File này giúp loại bỏ hoàn toàn các giá trị hardcode trong mã nguồn. Bạn có thể định nghĩa tự do các vị trí phòng (nodes), khoảng cách kết nối giữa chúng (edges) và nơi nào là cửa thoát hiểm an toàn (exits).
* **`src/main.rs`**: Khởi tạo cấu hình và chạy HTTP server với Actix-web. Thiết lập các worker chạy ngầm bằng `tokio::spawn` để xử lý hàng đợi Database và Watchdog.
* **`src/lib.rs`**: Khai báo và cấu hình các module chức năng chính (controllers, database, services, websocket, v.v.).

## Kiến Trúc Xử Lý Database & Watchdog
Hệ thống sử dụng luồng thiết kế **Asynchronous queues (hàng đợi bất đồng bộ)** bằng framework `tokio`:
1. **Batch Insert Database:** Thay vì mở kết nối DB (pool) liên tục ứng với mỗi thông điệp MQTT, server sử dụng `tokio::sync::mpsc::channel` để gom thông điệp. Cứ sau **500ms** hoặc thu thập đủ **50 Payload**, dữ liệu sẽ được đẩy vào Database bằng 1 câu truy vấn `INSERT` chung (`sqlx::QueryBuilder`), khắc phục tình trạng "timeout ngẽn pool kết nối".
2. **Server-side Timestamp Watchdog:** Để nhận diện chính xác Node chết thay vì bị nhiễu do xung đột đồng hồ trên phần cứng (ESP32 `millis()`), backend tự động gán nhãn thời gian Epoch hệ thống (`SystemTime::now`) vào gói tin ngay lập tức khi MQTT tiếp nhận. Logic này xử lý chuẩn xác độ ổn định của Watchdog 15 giây.

## Kết Nối WebSocket Thời Gian Thực (`/ws`)

Hệ thống hỗ trợ giao tiếp hai chiều theo thời gian thực (real-time) thông qua WebSocket. 
- Tính năng này được triển khai sử dụng thư viện **`actix-ws`** thay vì thư viện cũ `actix-web-actors`.
- Để quản lý và phân phối tin nhắn đến nhiều clients cùng một lúc một cách an toàn và tối ưu, hệ thống sử dụng **`tokio::sync::broadcast`**. Kênh truyền thông (channel) này cho phép phát (broadcast) tín hiệu trạng thái và các sự kiện từ server đến tất cả các kết nối WebSocket đang mở.

## Kiểm Thử (Testing)

Dự án được tổ chức và kiểm thử một cách bài bản nhằm đảm bảo chất lượng và độ ổn định của hệ thống:

- **Unit tests (Kiểm thử đơn vị)**: Cùng nằm trong thư mục `src/`, thường cấu hình bằng `#![cfg(test)]` cạnh mã nguồn gốc.
- **Integration & Concurrent tests (Kiểm thử tích hợp & Tương tác đồng thời)**: Được đặt trong thư mục `tests/`. Nó giúp kiểm định quy trình hoạt động giữa các modules với nhau:
  - `tests/api_tests.rs`: Kiểm thử trực tiếp lên các route REST API.
  - `tests/common_tests.rs`: Chứa các kiểm thử liên quan đến logic hoặc các tiện ích dùng chung xuyên suốt backend.
  - `tests/deadlock_test.rs`: Mô phỏng tải lưu lượng mạng (stress test) đa nguồn (hàng trăm Async Worker thread gửi thông điệp kẹt cùng lúc) để phòng ngừa xung đột bộ nhớ và kiểm thử hệ khả năng không bị lỗi Deadlock của luồng `DashMap` và luồng thuật toán Dijkstra. Môi trường này sử dụng mô hình lập lịch ảo chặn luồng.

### Cách Chạy Kiểm Thử

Để chạy toàn bộ các bài unit tests và integration tests, bạn chỉ cần mở terminal, di chuyển vào thư mục `backend/` và chạy lệnh sau:

```bash
cargo test
```

Lệnh này sẽ tự động thu thập và kiểm tra tất cả các tệp có đuôi `.rs` trong môi trường kiểm thử (cả bên trong `src/` và `tests/`). Mặc định thông tin log khi pass sẽ bị ẩn đi, bạn có thể chạy `cargo test -- --nocapture` nếu muốn in kết quả log chi tiết.
