# Hệ Thống Cảnh Báo Cháy & Tìm Đường Sơ Tán (ESP32 Backend)

Dự án này là Backend viết bằng **Rust**, hoạt động đóng vai trò như một bộ não trung tâm để nhận và phân tích dữ liệu trực tiếp từ các nút cảm biến (ESP32) qua giao thức MQTT. Nó có chức năng phát hiện hoả hoạn dựa trên thuật toán/mô hình phân tích nguy cơ cháy, đồng thời cung cấp REST API cho phép truy vấn trạng thái, cảnh báo và chỉ dẫn lộ trình sơ tán an toàn nhất theo thời gian thực (Dijkstra Algorithm).

## Tổng Quan Kiến Trúc

* **Rust Framework**: `actix-web` (Cung cấp REST API hiệu năng cao), `tokio` (Quản lý các tác vụ bất đồng bộ - Async runtime).
* **Kết Nối Điểm Cuối**: `rumqttc` (Lắng nghe tín hiệu MQTT), `sqlx` (Tích hợp cùng CSDL PostgreSQL để lưu vết Payload lịch sử).
* **Phân Tích AI & Giải thuật**:
  * Phát Hiện Cháy (Fire Detection): Theo dõi độ dốc gia tăng nhiệt độ/khói, phát hiện bất thường và flame sensor.
  * Tìm Lộ Trình (Path Finding): Quét đồ thị toà nhà và sử dụng Dijkstra kết hợp trọng số động (dựa vào ngưỡng nguy hiểm của phòng) để tránh chạy vào nơi có đám cháy.

## Cách Hoạt Động

1. Các Node ESP32 truyền dữ liệu đo lường liên tục tới MQTT Broker (topic: `fire/#`).
2. Backend (ở tiến trình ngầm) nhận Payload, chạy tiền xử lý (Parse JSON) rồi lưu vào bộ nhớ tạm thời (`app_state`), đồng thời insert vào CSDL.
3. Hệ thống chạy `FireDetectionModel` nhận diện rủi ro tại mỗi phòng (có ngọn lửa, khói cao đột biến, hoặc nhiệt độ cảnh báo).
4. Nếu phát hiện rủi ro cháy (`has_fire = true`), hệ thống chạy thuật toán Dijkstra để tính toán lại lộ trình thoát hiểm cho toàn bộ toà nhà tránh xa khu vực nguy hiểm.
5. Người ứng cứu/Frontend có thể gọi tới các **[REST API / Controllers](backend/src/controllers/readme.md)** để lấy được mọi dữ liệu mới nhất.

## Hướng Dẫn Cài Đặt & Chạy

### Yêu Cầu Cấu Hình
* **Rust**: Bản cài đặt stable mới nhất.
* **MQTT Broker**: Đang chạy trên máy chủ (Ví dụ: `Mosquitto`).
* **PostgreSQL**: Nếu bật kết nối CSDL trong cấu hình logic.

### File Môi Trường (`.env`)
Tạo một file `.env` ở gốc hoặc trong thư mục `backend/` tương tự như sau:
```env
ip=localhost
port=8080
mqtt_broker=localhost
mqtt_port=1883
database_url=postgres://user:password@localhost/db_name
```

### Chạy Dự Án Chế Độ Phát Triển

```bash
cd backend
cargo check     # Kiểm tra lỗi biên dịch
cargo test      # Chạy kiểm thử các hàm tính toán
cargo run       # Bắt đầu khởi chạy server web & service mqtt
```

## Các Module Chính
* `common/`: Lưu trữ các thuật toán lõi (Mô hình phát hiện cháy, tìm đường đồ thị thuật toán Dijkstra).
* `configs/`: Tải và parse biến môi trường.
* `constants/`: Nơi cài đặt hệ số cứng quan trọng (ngưỡng giới hạn nhiệt/khói, trọng số Dijkstra).
* `controllers/`: Nơi quản lý các Entry point (APIs endpoint) giao tiếp với người dùng ([Xem chi tiết API Docs tại đây](backend/src/controllers/readme.md)).
* `database/`: Các kết nối `sqlx` với dữ liệu.
* `state/`: Chứa `AppState`, là mạch máu kết nối bộ đệm dữ liệu (cached memory) giữa MQTT - AI Models - APIs.
