const ws = new WebSocket('ws://localhost:8080/ws');

ws.onopen = () => {
  console.log('WebSocket đã kết nối');
  ws.send('ping'); // Test ping
};

ws.onmessage = (event) => {
  if (event.data === 'pong') {
    return console.log('Heartbeat OK');
  }

  try {
    const data = JSON.parse(event.data);
    
    if (data.type === "SensorAndPathUpdate") {
      console.log(`\n[Cảm Biến Mới] Node: ${data.payload.node_id} | Nhiệt: ${data.payload.temperature}`);
      
      if (data.evacuation_paths) {
        console.log(`CÓ CHÁY! Nhận được ${data.evacuation_paths.length} đường dẫn sơ tán:`);
        data.evacuation_paths.forEach(p => {
           console.log(`Node ${p.node_id} thoát ra cửa số ${p.exit_node} qua đường: [${p.path.join(" -> ")}]`);
        });
      } else {
        console.log("Toà nhà đang an toàn.");
      }
    }
  } catch (e) {
    console.error('Lỗi parse JSON:', e);
  }
};

ws.onerror = (err) => console.error('Lỗi:', err);
ws.onclose = () => console.log('Đã đóng');