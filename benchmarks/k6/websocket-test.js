// k6 WebSocket benchmark - Tests arena allocation optimization
// Measures WebSocket connection performance and message broadcasting

import ws from 'k6/ws';
import { check } from 'k6';
import { Rate, Counter, Trend } from 'k6/metrics';

const errorRate = new Rate('ws_errors');
const messagesReceived = new Counter('messages_received');
const connectionDuration = new Trend('ws_connection_duration');
const messageDuration = new Trend('ws_message_duration');

export const options = {
    vus: 50,  // 50 concurrent WebSocket connections
    duration: '2m',
    thresholds: {
        ws_errors: ['rate<0.05'],
        ws_connection_duration: ['p(95)<1000'],
    },
};

const BASE_URL = __ENV.WS_URL || 'ws://localhost:3000';

export default function () {
    const url = `${BASE_URL}/ws`;
    const params = {
        tags: { name: 'WebSocketTest' },
    };
    
    const connectStart = Date.now();
    
    const res = ws.connect(url, params, function (socket) {
        connectionDuration.add(Date.now() - connectStart);
        
        socket.on('open', function () {
            console.log(`VU ${__VU}: Connected to WebSocket`);
            
            // Send a test message
            const messageStart = Date.now();
            socket.send(JSON.stringify({
                type: 'ping',
                data: { timestamp: Date.now() },
            }));
        });
        
        socket.on('message', function (data) {
            messageDuration.add(Date.now() - Date.parse(JSON.parse(data).timestamp || Date.now()));
            messagesReceived.add(1);
            
            check(data, {
                'message received': (d) => d.length > 0,
            });
        });
        
        socket.on('error', function (e) {
            console.error(`VU ${__VU}: WebSocket error: ${e.error()}`);
            errorRate.add(1);
        });
        
        socket.on('close', function () {
            console.log(`VU ${__VU}: Disconnected from WebSocket`);
        });
        
        // Keep connection open for 30 seconds
        socket.setTimeout(function () {
            socket.close();
        }, 30000);
    });
    
    check(res, {
        'WebSocket connected': (r) => r && r.status === 101,
    });
}

export function handleSummary(data) {
    return {
        'stdout': textSummary(data, { indent: ' ', enableColors: true }),
    };
}

function textSummary(data, options) {
    const indent = options.indent || '';
    const enableColors = options.enableColors || false;
    
    let summary = '\n' + indent + '=== WebSocket Benchmark Results ===\n';
    summary += indent + `Connections: ${data.metrics.vus_max.values.value}\n`;
    summary += indent + `Duration: ${(data.state.testRunDurationMs / 1000).toFixed(2)}s\n`;
    summary += indent + `Messages Received: ${data.metrics.messages_received.values.count}\n`;
    summary += indent + `Errors: ${(data.metrics.ws_errors.values.rate * 100).toFixed(2)}%\n`;
    summary += indent + `\nConnection Time:\n`;
    summary += indent + `  Avg: ${data.metrics.ws_connection_duration.values.avg.toFixed(2)}ms\n`;
    summary += indent + `  P95: ${data.metrics.ws_connection_duration.values['p(95)'].toFixed(2)}ms\n`;
    summary += indent + `\nNote: Tests arena allocation optimization in broadcast()\n`;
    summary += indent + '=====================================\n';
    
    return summary;
}

