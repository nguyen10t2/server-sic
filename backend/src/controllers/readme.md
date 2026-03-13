# Web Controllers API (REST Endpoints)

Thư mục này chịu trách nhiệm phục vụ các Endpoint giao tiếp với Client bên ngoài qua giao thức HTTP (sử dụng thư viện `actix-web`). Mọi dữ liệu trả về đều dưới dạng JSON (dùng `serde`).

## Danh Sách Rest API

Dưới đây là định nghĩa và các ví dụ về cách gọi các Endpoints.

### 1. Trạng Thái Cảm Biến Của Các Node
**Tuyến Đường:** `GET /api/status`
**Chức Năng:** Lấy các thiết lập đo lường/cảm biến cuối cùng được gửi từ *tất cả* các Node ESP32 có mặt ở hệ thống (trong cache memory).

**Ví Dụ Response:**
```json
[
  {
    "timestamp": 1773373760916,
    "temperature": 25.15,
    "humidity": 55.0,
    "smoke": 0.0,
    "flame": 0.0,
    "node_id": 20,
    "battery": 92,
    "status": 0
  }
]
```

### 2. Tình Hình Cháy Hiện Tại Toàn Cục
**Tuyến Đường:** `GET /api/fire/status`
**Chức Năng:** Trả về một bảng phân tích sự biến đổi của tất cả các Node cảm biến và độ tin cậy rủi ro theo "Chỉ Số Cháy" đã được tính toán bởi thuật toán mô hình AI cảnh báo.

**Ví Dụ Response:**
```json
{
  "has_fire": true,
  "fire_nodes": [
    {
      "node_id": 8,
      "fire_probability": 1.0,
      "is_fire": true,
      "risk_level": "Critical",
      "details": {
         "temperature_score": 1,
         "smoke_score": 1,
         "humidity_score": 0.1,
         "flame_factor": 1,
         "trend_factor": 0.95,
         "anomaly_score": 0.15
      }
    }
  ]
}
```

### 3. Lấy Lộ Trình Sơ Tán Của Một Node (Căn Phòng)
**Tuyến Đường:** `GET /api/evacuate/{node_id}`
**Ví Dụ Gọi:** `GET /api/evacuate/5` (Sơ tán từ nút số 5 ra ngoài điểm gần nhất).
**Chức Năng:** Giao cho AI tìm đường ngắn nhất, an toàn nhất (tránh những node đang có trạng thái Critical) dựa trên thuật toán Dijkstra kèm theo "Danger Weight".

**Ví Dụ Response:**
```json
{
  "node_id": 5,
  "path": [5, 4, 3, 2, 1],
  "total_weight": 16.0,
  "exit_node": 1,
  "has_fire": true
}
```

### 4. Lấy Lộ Trình Của Toàn Bộ Các Nút
**Tuyến Đường:** `GET /api/evacuate/all`
**Chức Năng:** Trả về danh sách chi tiết của mảng `paths` đối với tất cả các nút cùng lúc (trừ những nút đã bị thiêu rụi). Tối ưu cho Frontend render bản đồ tổng.

**Ví dụ Response:**
```json
{
  "has_fire": false,
  "paths": [
    {
      "node_id": 20,
      "path": [20, 19, 15],
      "total_weight": 8.0,
      "exit_node": 15
    },
    {
      "node_id": 2,
      "path": [2, 1],
      "total_weight": 4.0,
      "exit_node": 1
    }
  ]
}
```

### 5. Thông Tin Bản Đồ Của Toà Nhà
**Tuyến Đường:** `GET /api/building/graph`
**Chức Năng:** Cho phép truy vấn tất cả các đỉnh (nodes), các cạnh khoảng cách mặc định (edges) và các lối thoát (exits nodes) được thiết lập tại Server. (Phục vụ việc vẽ Node Graph ở Frontend React).

**Ví Dụ Phản Hồi:**
```json
{
  "nodes": [1, 2, 3, 4, 5],
  "edges": [
    [1, 2, 4.0],
    [2, 3, 4.0]
  ],
  "exits": [5, 10, 15, 20]
}
```