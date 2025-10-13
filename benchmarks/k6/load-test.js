// k6 load test script - Comprehensive API testing
// Tests multiple endpoints with realistic user scenarios

import http from 'k6/http';
import { check, group, sleep } from 'k6';
import { Rate, Trend, Counter } from 'k6/metrics';

// Custom metrics
const errorRate = new Rate('errors');
const authDuration = new Trend('auth_duration');
const taskListDuration = new Trend('task_list_duration');
const taskCreateDuration = new Trend('task_create_duration');
const fileUploadDuration = new Trend('file_upload_duration');
const authSuccessCounter = new Counter('auth_success');
const taskCreatedCounter = new Counter('tasks_created');

// Test configuration
export const options = {
    stages: [
        { duration: '30s', target: 10 },   // Ramp up to 10 users
        { duration: '1m', target: 50 },    // Ramp up to 50 users
        { duration: '2m', target: 100 },   // Ramp up to 100 users
        { duration: '2m', target: 100 },   // Stay at 100 users
        { duration: '1m', target: 50 },    // Ramp down to 50 users
        { duration: '30s', target: 0 },    // Ramp down to 0 users
    ],
    thresholds: {
        http_req_duration: ['p(95)<500'],  // 95% of requests should be below 500ms
        http_req_failed: ['rate<0.05'],    // Error rate should be below 5%
        errors: ['rate<0.05'],
    },
};

const BASE_URL = __ENV.BASE_URL || 'http://localhost:3000';

// Test data
const testUser = {
    name: 'Benchmark User',
    email: `bench-${Date.now()}-${__VU}@example.com`,
    password: 'SecurePass123!',
};

export function setup() {
    console.log('Setting up load test...');
    console.log(`Base URL: ${BASE_URL}`);
    console.log(`VUs: ${__ENV.K6_VUS || 'auto'}`);
    return { startTime: Date.now() };
}

export default function () {
    let authToken = null;
    
    // Group 1: Authentication
    group('Authentication', function () {
        // Register new user
        const registerPayload = JSON.stringify(testUser);
        const registerRes = http.post(
            `${BASE_URL}/api/v1/auth/register`,
            registerPayload,
            {
                headers: { 'Content-Type': 'application/json' },
                tags: { name: 'Register' },
            }
        );
        
        const registerSuccess = check(registerRes, {
            'register status is 201': (r) => r.status === 201,
            'register has token': (r) => {
                try {
                    const body = JSON.parse(r.body);
                    return body && body.data && body.data.token !== undefined;
                } catch (e) {
                    return false;
                }
            },
        });
        
        errorRate.add(!registerSuccess);
        
        if (registerSuccess) {
            try {
                authToken = JSON.parse(registerRes.body).data.token;
                authSuccessCounter.add(1);
            } catch (e) {
                console.error('Failed to parse register response:', registerRes.body);
            }
        } else {
            // Try logging in with existing user
            const loginPayload = JSON.stringify({
                email: testUser.email,
                password: testUser.password,
            });
            
            const loginRes = http.post(
                `${BASE_URL}/api/v1/auth/login`,
                loginPayload,
                {
                    headers: { 'Content-Type': 'application/json' },
                    tags: { name: 'Login' },
                }
            );
            
            authDuration.add(loginRes.timings.duration);
            
            const loginSuccess = check(loginRes, {
                'login status is 200': (r) => r.status === 200,
                'login has token': (r) => {
                    try {
                        const body = JSON.parse(r.body);
                        return body && body.data && body.data.token !== undefined;
                    } catch (e) {
                        return false;
                    }
                },
            });
            
            errorRate.add(!loginSuccess);
            
            if (loginSuccess) {
                try {
                    authToken = JSON.parse(loginRes.body).data.token;
                    authSuccessCounter.add(1);
                } catch (e) {
                    console.error('Failed to parse login response:', loginRes.body);
                }
            }
        }
    });
    
    if (!authToken) {
        console.error('Failed to authenticate');
        return;
    }
    
    const headers = {
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${authToken}`,
    };
    
    sleep(1); // Simulate user think time
    
    // Group 2: Task Operations
    group('Task Operations', function () {
        // Create a task
        const taskPayload = JSON.stringify({
            title: `Benchmark Task ${Date.now()}-${__VU}`,
            description: 'Created by k6 load test to measure performance',
        });
        
        const createRes = http.post(
            `${BASE_URL}/api/v1/tasks`,
            taskPayload,
            {
                headers: headers,
                tags: { name: 'CreateTask' },
            }
        );
        
        taskCreateDuration.add(createRes.timings.duration);
        
        const createSuccess = check(createRes, {
            'create task status is 201': (r) => r.status === 201,
            'create task has id': (r) => JSON.parse(r.body).data.id !== undefined,
        });
        
        errorRate.add(!createSuccess);
        
        if (createSuccess) {
            taskCreatedCounter.add(1);
        }
        
        sleep(0.5);
        
        // List tasks (tests arena allocation optimization)
        const listRes = http.get(
            `${BASE_URL}/api/v1/tasks?page=1&limit=20&sort_by=created_at&sort_direction=desc`,
            {
                headers: headers,
                tags: { name: 'ListTasks' },
            }
        );
        
        taskListDuration.add(listRes.timings.duration);
        
        check(listRes, {
            'list tasks status is 200': (r) => r.status === 200,
            'list tasks has data': (r) => JSON.parse(r.body).data !== undefined,
        });
        
        errorRate.add(listRes.status >= 400);
        
        sleep(0.5);
        
        // List with filters (tests complex query building)
        const filterRes = http.get(
            `${BASE_URL}/api/v1/tasks?status=todo&page=1&limit=10`,
            {
                headers: headers,
                tags: { name: 'FilterTasks' },
            }
        );
        
        check(filterRes, {
            'filter tasks status is 200': (r) => r.status === 200,
        });
        
        errorRate.add(filterRes.status >= 400);
    });
    
    sleep(1);
    
    // Group 3: File Operations (tests zero-copy optimization)
    group('File Operations', function () {
        // Create a small test file
        const boundary = '----WebKitFormBoundary7MA4YWxkTrZu0gW';
        const fileContent = 'k6 benchmark test file content\nTesting zero-copy optimization\n';
        
        const body = `--${boundary}\r\n` +
            `Content-Disposition: form-data; name="file"; filename="k6-test-${__VU}.txt"\r\n` +
            `Content-Type: text/plain\r\n\r\n` +
            `${fileContent}\r\n` +
            `--${boundary}--\r\n`;
        
        const uploadRes = http.post(
            `${BASE_URL}/api/v1/files`,
            body,
            {
                headers: {
                    'Content-Type': `multipart/form-data; boundary=${boundary}`,
                    'Authorization': `Bearer ${authToken}`,
                },
                tags: { name: 'UploadFile' },
            }
        );
        
        fileUploadDuration.add(uploadRes.timings.duration);
        
        check(uploadRes, {
            'upload file status is 201': (r) => r.status === 201,
        });
        
        errorRate.add(uploadRes.status >= 400);
        
        sleep(0.5);
        
        // List files
        const listFilesRes = http.get(
            `${BASE_URL}/api/v1/files`,
            {
                headers: headers,
                tags: { name: 'ListFiles' },
            }
        );
        
        check(listFilesRes, {
            'list files status is 200': (r) => r.status === 200,
        });
        
        errorRate.add(listFilesRes.status >= 400);
    });
    
    sleep(1);
}

export function teardown(data) {
    const duration = (Date.now() - data.startTime) / 1000;
    console.log(`\n=== Load Test Complete ===`);
    console.log(`Total Duration: ${duration.toFixed(2)}s`);
}

