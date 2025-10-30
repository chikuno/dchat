// k6 load testing script for dchat relay network
// Run with: k6 run --vus 100 --duration 30s relay_stress_test.js

import http from 'k6/http';
import { check, sleep } from 'k6';
import { Rate, Trend } from 'k6/metrics';

// Custom metrics
const errorRate = new Rate('errors');
const messageLatency = new Trend('message_latency');

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
        http_req_duration: ['p(95)<500'], // 95% of requests should be below 500ms
        http_req_failed: ['rate<0.01'],   // Error rate should be below 1%
        errors: ['rate<0.05'],            // Custom error rate below 5%
    },
};

// Base URL for relay nodes
const BASE_URL = __ENV.RELAY_URL || 'http://localhost:7071';

// Generate random message
function generateMessage() {
    return JSON.stringify({
        id: `msg-${Date.now()}-${Math.random()}`,
        sender: `user-${__VU}`,
        content: `Test message from VU ${__VU} at ${new Date().toISOString()}`,
        timestamp: Date.now(),
    });
}

// Test scenario: Send and receive messages
export default function() {
    const message = generateMessage();
    
    // Send message to relay
    const sendStart = Date.now();
    const sendResponse = http.post(`${BASE_URL}/api/v1/messages`, message, {
        headers: { 'Content-Type': 'application/json' },
        timeout: '10s',
    });
    
    const sendSuccess = check(sendResponse, {
        'send status is 200': (r) => r.status === 200,
        'send has message ID': (r) => r.json('message_id') !== undefined,
    });
    
    errorRate.add(!sendSuccess);
    messageLatency.add(Date.now() - sendStart);
    
    if (sendSuccess) {
        const messageId = sendResponse.json('message_id');
        
        // Poll for message delivery
        sleep(0.5);
        
        const receiveResponse = http.get(`${BASE_URL}/api/v1/messages/${messageId}`, {
            timeout: '5s',
        });
        
        const receiveSuccess = check(receiveResponse, {
            'receive status is 200': (r) => r.status === 200,
            'message delivered': (r) => r.json('status') === 'delivered',
        });
        
        errorRate.add(!receiveSuccess);
    }
    
    // Random sleep between 1-3 seconds
    sleep(Math.random() * 2 + 1);
}

// Setup function - runs once per VU before default function
export function setup() {
    // Health check
    const healthResponse = http.get(`${BASE_URL}/health`);
    check(healthResponse, {
        'relay is healthy': (r) => r.status === 200,
    });
    
    return { startTime: Date.now() };
}

// Teardown function - runs once after all iterations
export function teardown(data) {
    const duration = (Date.now() - data.startTime) / 1000;
    console.log(`Test completed in ${duration} seconds`);
}
