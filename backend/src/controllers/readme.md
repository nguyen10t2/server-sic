# Web Controllers API & WebSocket

Thư mục này chịu trách nhiệm phục vụ các Endpoint kết nối với Client ngoài (Frontend UI, Mobile...) qua giao thức HTTP (sử dụng thư viện `actix-web`) và kết nối trực tuyến qua **WebSocket (`actix-ws`)**. Mọi dữ liệu giao tiếp đều được chuẩn hóa dưới định dạng JSON (`serde`).

Tài liệu này được biên soạn để đội ngũ Frontend (React/Vue/v.v..) nắm rõ chuẩn kết nối và tích hợp.

---

## 🔌 Kênh Kết Nối WebSocket (Real-time Stream)
Kênh này thay thế cho việc gọi API HTTP liên tục (polling). Nó sẽ chủ động đẩy dữ liệu (Push) cho Frontend ngay khi các thiết bị nhúng (ESP32) phát tín hiệu mới.

* **Đường dẫn (URL):** `ws://<SERVER_IP>:<PORT>/ws` (Sử dụng `wss://` nếu đã cấu hình SSL).
* **Luồng hoạt động:** 
  - Khi một ESP32 gửi bản tin MQTT lên cloud, hệ thống Actix sẽ đẩy ngay bản tin đó đến mọi Frontend đang bắt tay (subscribe) qua WebSocket này.
* **Cơ chế Heartbeat (Ping/Pong):**
  - Tránh bị dính timeout từ trình duyệt, phía Frontend có thể gửi chuỗi `"ping"`. Server ngay lập tức phản hồi chuỗi `"pong"`.

**Ví dụ Message Dữ Liệu Cảm Biến Trả Về (Phía Client đón nhận qua `event.data`):**
```json
{
  "type": "SensorAndPathUpdate",
  "payload": {
    "timestamp": 1773421163390,
    "temperature": 25.15,
    "smoke": 0.0,
    "flame": false,
    "node_id": 20,
    "battery": 92,
    "status": 0
  },
  "evacuation_paths": null
}
```
*(Lưu ý: Nếu có cháy xảy ra, trường `evacuation_paths` sẽ trả về mảng chứa đường đi sơ tán thay vì `null`)*

**Đoạn Code Mẫu Dành Cho Frontend (JavaScript/TypeScript - Chạy với Bun/Node):**
```javascript
const ws = new WebSocket("ws://127.0.0.1:8080/ws");

ws.onopen = () => console.log("WebSocket connected!");

ws.onmessage = (event) => {
  // 1. Xử lý Ping/Pong
  if (event.data === "pong") return console.log("Heartbeat OK");

  // 2. Xử lý cập nhật thông số Node
  try {
    const data = JSON.parse(event.data);
    
    if (data.type === "SensorAndPathUpdate") {
      console.log(`\n[Cập Nhật] Node ${data.payload.node_id}: Temp=${data.payload.temperature}°C`);
      
      // Nếu có cháy, server sẽ đẩy luôn mảng đường sơ tán về
      if (data.evacuation_paths) {
        console.log(`CÓ CHÁY! Nhận được ${data.evacuation_paths.length} đường dẫn sơ tán:`);
        data.evacuation_paths.forEach(p => {
           console.log(`Node ${p.node_id} thoát ra cửa số ${p.exit_node} qua đường: [${p.path.join(" -> ")}]`);
        });
      } else {
        console.log("Toà nhà đang an toàn.");
      }
    }
  } catch (err) {
    console.error("Lỗi parse JSON: ", err);
  }
};

// Duy trì kết nối bằng cách ping mỗi 30 giây
const heartbeat = setInterval(() => {
  if (ws.readyState === WebSocket.OPEN) ws.send("ping");
}, 30000);

ws.onclose = () => clearInterval(heartbeat);
```

---

## 🌐 Danh Sách API REST (HTTP Endpoints)

Dưới đây là các Endpoints gọi một lần (One-time fetch), chủ yếu dùng lúc màn hình Frontend **vừa mới tải hoặc load lần đầu** để vẽ được nền tảng sơ đồ, sau đó WebSocket ở trên sẽ lo việc cập nhật thay đổi nhỏ.

### 1. Trạng Thái Cảm Biến Cuối Của Tất Cả Các Node
* **Tuyến Đường:** `GET /api/status`
* **Mục đích:** Khi người dùng F5 hoặc vừa vào Web, gọi cái này để vẽ tức thì ra thông số lần chót của tất cả các phòng/Node thay vì phải đợi tín hiệu WebSocket đến lắt nhắt.

**Phản hồi:**
```json
[
  {
    "timestamp": 1773373760916,
    "temperature": 25.15,
    "smoke": 0.0,
    "flame": false,
    "node_id": 20,
    "battery": 92,
    "status": 0
  }
]
```

### 2. Tình Hình Cháy Hiện Tại Toàn Cục
* **Tuyến Đường:** `GET /api/fire/status`
* **Mục đích:** Trả về tất cả các Node bị hệ thống AI phán đoán là Đang Cháy/Nguy Hiểm cùng với trọng số (risk_level).

**Phản hồi:**
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
         "temperature_score": 1.0,
         "smoke_score": 1.0,
         "anomaly_score": 0.15
      }
    }
  ]
}
```

### 3. Sơ Tán Tổng Thể Đại Trà Toàn Tòa Nhà
* **Tuyến Đường:** `GET /api/evacuate/all`
* **Mục đích:** Frontend gọi API này để tự động dựng các "Đường dẫn mũi tên chạy trốn" trên đồ họa React 2D/3D cho toàn bộ những phòng chưa cháy tới điểm thoát hiểm an toàn nhất (mỗi object là của 1 sinh mạng/node).

**Phản hồi:**
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

### 4. Tìm Đường Sơ Tán Của Một Node Riêng Lẻ
* **Tuyến Đường:** `GET /api/evacuate/{node_id}`
* **Ví Dụ Query:** `GET /api/evacuate/5` (Sơ tán từ nút số 5)
* **Mục đích:** Người dùng click vào một phòng bảo vệ trên bản đồ UI và yêu cầu "Tôi đang ở đây, chỉ tôi đường chui ra khỏi tòa nhà gần nhất nhạy bén nhất".

**Phản hồi:**
```json
{
  "node_id": 5,
  "path": [5, 4, 3, 2, 1],
  "total_weight": 16.0,
  "exit_node": 1,
  "has_fire": true
}
```

### 5. Khung Xương Bản Đồ (Nodes, Edges, Exits)
* **Tuyến Đường:** `GET /api/building/graph`
* **Mục đích:** Cung cấp thông số "hình học" để Frontend vẽ các hình tròn (Node), nét nối (Edge Cạnh) và cửa ra (Exit) lên file UI Vector. Đây là xương sống cấu trúc tĩnh.

**Phản hồi:**
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
