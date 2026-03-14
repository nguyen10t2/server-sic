# 🧠 Tài Liệu Cốt Lõi Hệ Thống (Core Logic & Architecture)

Tài liệu này tập trung giải thích các ý tưởng nền tảng, thuật toán và logic nghiệp vụ chính của Backend (không bao gồm các phần bao bọc bên ngoài như giao tiếp API, HTTP hay WebSocket).

---

## 1. Mô Hình Chẩn Đoán & Phát Hiện Cháy (Fire Detection Model)

Hệ thống không chỉ dựa vào 1 cảm biến đơn lẻ để báo cháy mà tiến hành tổng hợp theo "chỉ số rủi ro" (Danh giá đa biến) và theo dõi xu hướng (Time-series/Trend).

*   **Đầu vào (Cảm biến):** Nhiệt độ (Temperature), Độ ẩm (Humidity), Nồng độ khói/khí gas (Smoke) và Cảm biến lửa trực tiếp (Flame).
*   **Trọng số Nguy hiểm (Danger Scoring):**
    *   Mỗi chỉ số được chuẩn hoá theo điểm số. Ví dụ: Nhiệt độ chia cho ngưỡng `TEMP_MAX_SCORE`, Cảm biến khói chia cho ngưỡng `SMOKE_MAX_SCORE`.
    *   Nếu phát hiện `Flame == true`, hệ thống lập tức cộng dồn một điểm phạt (Penalty) khổng lồ, đưa thẳng vào trạng thái Báo động.
*   **Chẩn đoán theo Xu hướng (Trend Analysis):** 
    *   Hệ thống lưu lại lịch sử đo lường ngắn hạn của từng node. Nếu nhận thấy biên độ nhiệt có sự thay đổi đột ngột (ví dụ tăng 5-10 độ chỉ trong 1-2 vạch đo), hệ thống nhân thêm hệ số `trend_factor`, báo cáo nhạy bén hơn trước khi cháy lan rộng.
*   **Phân loại Rủi ro:** Điểm cuối cùng (`fire_probability`) sẽ quy định cấp độ từ `Safe` $\rightarrow$ `Low` $\rightarrow$ `Medium` $\rightarrow$ `High` $\rightarrow$ `Critical`. Khi đạt ngưỡng Critical, node mới bị tuyên bố là cháy (`is_fire = true`).

---

## 2. Hệ Thống "Watchdog" Giám Sát Sự Sống Kép (Dead Node Detection)

Trong thực tế thảm hoạ, các node vật lý có thể chết đột ngột mà không kịp gửi tín hiệu cháy (cháy đứt dây, nổ pin, sập mạng). 

*   **Logic Hoạt Động:** Backend vận hành một vòng lặp nhịp tim ngầm (chạy mỗi 5 giây). Hệ thống quét và trừ lùi `timestamp` gói tin tĩnh cuối cùng của tất cả 20 Node ESP32 so với đồng hồ gốc của Server.
*   **Bắt lỗi lâm sàng:** Nếu quá **15 giây** một mạch không nổ phát tín hiệu, Watchdog sẽ phán quyết là `NODEDEAD` (Trạng thái = 3).
*   **Hệ quả Tương tự Lửa:** Các node cấu trúc bị rớt mạng như thế này sẽ tự động bị loại khỏi hệ sinh thái an toàn, được gán trọng lượng vô cực, ép buộc quy trình Tìm Đường (Path Finding) tiếp theo phải đi vòng rẽ né tránh.

---

## 3. Bản Đồ Toà Nhà & Tính Toán Trọng Số Động (Dynamic Graph Weights)

Toà nhà được cấu trúc hoá bằng Đồ Thị (Graph Graph) dưới dạng các nút (node) và các cạnh (edge nối các phòng tương đương với cân nặng cơ bản mặc định).

*   **Kiến trúc Mảng Lưới 4x5:** Cấu trúc nhà được vạch định dưới dạng 4 Hàng $\times$ 5 Cột, chứa các lối ra (Exits) ở những vị trí chiến lược (Ví dụ: `5, 10, 15, 20`).
*   **Đường Đi Có Định Tính:** Trọng số (cost of travel) ở đây không phải là khoảng cách (m). Ngược lại là **Độ Nguy Hiểm**.
    *   `Weight = Distance * (1 + Danger_Avg)`: Đường đi giữa 2 vùng sẽ bị kéo giãn dài ra theo cấp số nhân nếu 1 trong 2 vùng đang nóng lên hoặc có khói, hướng mũi người di tản sang con đường khác bớt rủi ro hơn.
    *   **Vùng Tử Địa:** Nếu 1 Node rơi vào `NODEFIRE` (Đang Cháy) hoặc `NODEDEAD` (Chết Kết Nối), trọng lượng quy chụp thành $\infty$ (`f32::INFINITY`), chặt đứt mắt xích, tường thành rào vĩnh viễn với luồng sơ tán.

---

## 4. Thuật Toán Định Định Tuyến Sơ Tán Cứu Nạn (Evacuation Routing)

Khi toà nhà ở trạng thái "Có Lửa", module Path Finding sẽ kích hoạt cho **những node dân cư chưa bị cháy**. 

*   **Thuật toán Dijkstra Biến Thể:** Dijkstra được sử dụng để quét ra toàn bộ danh sách `paths` - lộ trình đường thoát thân nhỏ nhất về mặt trọng số "NGUY HIỂM" (tính ở bước số 3), dẫn tới Exit an toàn tối ưu. Con đường này không cam kết là đường *nhanh nhất*, mà là đường **bọc thép nhất**.
*   **Tính toán Phương hướng ESP32 (LED Array Dịch La Bàn):**
    *   Do các thiết bị mạch như ESP32 cần phải hiện hình ảnh siêu tối giản. Backend nhận trách nhiệm biên dịch lộ tuyến mảng `[Current, Next_X, Next_Y...]` ra thành mũi tên nháy.
    *   Sử dụng toán học tọa độ trên mặt lưới: 
        *   Tịnh tiến cùng hàng: Sang Đông `+1`, Sang Tây `-1`.
        *   Tịnh tiến khác cột: Đi xuống Nam `+5`, Đi Lên Bắc `-5`.
    *   Backend bọc thành gói lệnh gửi thẳng trực tiếp vào luồng vi điều khiển của từng ESP riêng biêt, đồng bộ còi rú (`Buzzer`) nếu khu vực nguy hiểm.

---

## 5. Cơ Chế Báo Động & Phản Hồi Trực Tiếp (Alert & Feedback Dispatching)

Sau khi có một tín hiệu hoặc chẩn đoán cháy chính thức (điểm rủi ro vượt ngưỡng `Critical`) và hệ thống vừa hoàn tất việc tính toán trọng số động lẫn quy trình Tìm Đường (Dijkstra), logic xử lý tiếp nối là phân phối cảnh báo nhằm "đóng băng" vùng nguy hiểm và "mở luồng thoát" cho vùng an toàn:

*   **Phân Trách Nhiệm Thiết Bị Đích (Target Classification):**
    *   **Vùng Đỏ (Node Phát Hiện Cháy):** Các thiết bị có trạng thái `NODEFIRE` lặp tức nhận lệnh chuyển sang chế độ khẩn cấp (Buzzer = ON, chu kỳ rền SOS liên tục, đèn rực đỏ nếu có). Nhiệm vụ của node lúc này là răn đe, xua đuổi để không có ai bước lầm vào vùng tập trung lửa.
    *   **Vùng Xanh (Thành Phần Dẫn Đường - Node An Toàn):** Thu nhận gói tin định hướng mà thuật toán Path Finding đã vạch ra. Từ đó chuyển hóa thành mũi tên (LED Matrix/Array) hiển thị trên hành lang để chỉ lối sống sót tới điểm Exit tối ưu.
*   **Đồng Bộ Lệnh Hàng Loạt (Mass Command Pipeline):**
    *   Trong thảm hoạ, từng giây đều giá trị. Backend sẽ không gửi yêu cầu phản hồi riêng lẻ mà nén các mã lệnh chỉ đạo (mũi tên hướng nào, còi rú hay không) cho toàn bộ các node vào cấu trúc dữ liệu gọn nhẹ nhất (Payload batching).
    *   Sự kiện này được đưa đi qua các luồng truyền tải real-time bảo đảm rằng MỌI thiết bị trong tòa nhà đều đánh thức tính năng báo nguy/cập nhật lộ trình đúng ở cùng một "tic tắc" thời gian (Synchronous Action).
*   **Phát Sóng Phương Tiện Ngoại Lớp (External Broadcasters):**
    *   Song song với phần Cứng (Embedded), luồng Cảnh báo gieo rắc trực tiếp các gói trạng thái (Event Hooks) cho cổng WebSocket truyền lên sơ đồ Dashboard của Nhân viên Trực tổng đài (Frontend).
    *   Đồng thời kích hoạt các dịch vụ cảnh báo vòng ngoài, ví dụ Push Notification, SMS, hoặc gọi VoIP tới hệ thống cơ sở dữ liệu PCCC bằng việc cung cấp chính xác tọa độ node đang gặp thảm hoạ.