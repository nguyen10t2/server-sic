const ws = new WebSocket('ws://f5.soict.io:8080/ws');

ws.onopen = () => {
  console.log('✓ WebSocket đã kết nối');
  ws.send('ping'); // Test ping
};

ws.onmessage = (event) => {
  console.log('Nhận dữ liệu:', event.data);
};

ws.onerror = (err) => console.error('Lỗi:', err);
ws.onclose = () => console.log('Đã đóng');