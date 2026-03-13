use esp32::database::schema::Payload;
use esp32::state::app_state::AppState;
use futures_util::future::join_all;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::timeout;

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_process_payload_deadlock() {
    // 1. Tạo dummy channel để giả lập phần DB Worker luôn xử lý thành công
    let (db_tx, mut db_rx) = mpsc::channel(1000);

    // Luồng Consumer (DB Worker mạo danh): Rút cạn channel để không bị đầy queue (full channel)
    tokio::spawn(async move {
        while let Some(_payload) = db_rx.recv().await {
            // Giả lập lưu thành công vào DB, không làm gì cả
        }
    });

    // 2. Khởi tạo AppState chung giống với cấu hình thật (không có mqtt_client)
    let state = Arc::new(AppState::new(None, db_tx));

    // 3. Tạo một mảng chứa các task mô phỏng nhiều luồng đẩy dữ liệu vào cùng lúc
    let mut tasks = vec![];
    let num_concurrent_tasks = 200; // Số lượng task cùng gọi hàm process_payload đồng thời
    let iterations_per_task = 50; // Số vòng lặp mỗi task sẽ push data liên tục

    for task_id in 0..num_concurrent_tasks {
        let state_clone = state.clone();

        let t = tokio::spawn(async move {
            for i in 0..iterations_per_task {
                let mut payload = Payload::default();
                // Phân bổ ngẫu nhiên node 1-20
                payload.node_id = (task_id % 20) as u16 + 1;
                payload.temperature = 30.0 + (i as f32);

                // Cố ý tạo kịch bản cháy chập chờn (khiến thuật toán Dijkstra chạy lại liên tục)
                // Cứ vòng lặp thứ 5 (i % 5 == 0) hoặc ngẫu nhiên thì node đó báo cháy!
                payload.flame = i % 5 == 0;
                payload.status = if payload.flame { 2 } else { 0 };

                payload.timestamp = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as i64;

                // THỰC THI CHÍNH: Gọi vào process_payload
                // Nơi xảy ra read/write locks, DashMap, và chạy Dijktsra
                state_clone.process_payload(&payload);

                // Delay nhẹ giả lập tốc độ xử lý mạng (giảm áp lực tokio scheduler)
                tokio::time::sleep(Duration::from_millis(1)).await;
            }
        });
        tasks.push(t);
    }

    // 4. Kiểm tra Test có bị Deadlock hay không!
    // join_all đợi tất cả các task hoàn thành
    // Nếu AppState bị deadlock, hệ thống sẽ bị treo mãi ở lệnh wait
    // Do đó ta wrap trong 1 timeout 15 giây.
    let result = timeout(Duration::from_secs(15), join_all(tasks)).await;

    // 5. Kết quả
    assert!(
        result.is_ok(),
        "TEST TIMES OUT! Phát hiện Deadlock bên trong hàm AppState::process_payload hoặc hệ thống bị quá tải nặng nề."
    );

    println!(
        "Đã test thành công với {} payloads đồng thời. Hệ thống không gặp deadlock.",
        num_concurrent_tasks * iterations_per_task
    );

    // Xác minh DashMap vẫn toàn vẹn dữ liệu ở cuối
    assert!(state.latest_data.len() > 0);
}
