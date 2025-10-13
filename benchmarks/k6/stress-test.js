// k6 stress test - Find breaking point
// Progressively increases load to identify system limits

import http from 'k6/http';
import { check, sleep } from 'k6';
import { Rate } from 'k6/metrics';

const errorRate = new Rate('errors');

export const options = {
    stages: [
        { duration: '2m', target: 100 },    // Below normal load
        { duration: '5m', target: 100 },    // Stay at normal load
        { duration: '2m', target: 200 },    // Around breaking point
        { duration: '5m', target: 200 },    // Stay at breaking point
        { duration: '2m', target: 300 },    // Beyond breaking point
        { duration: '5m', target: 300 },    // Stay beyond breaking point
        { duration: '2m', target: 400 },    // Extreme load
        { duration: '5m', target: 400 },    // Stay at extreme load
        { duration: '10m', target: 0 },     // Recovery
    ],
    thresholds: {
        http_req_duration: ['p(99)<1000'], // 99% under 1s (will likely fail at high load)
    },
};

const BASE_URL = __ENV.BASE_URL || 'http://localhost:3000';

export default function () {
    const res = http.get(`${BASE_URL}/health`);
    
    check(res, {
        'status is 200': (r) => r.status === 200,
    });
    
    errorRate.add(res.status >= 400);
    
    sleep(1);
}

