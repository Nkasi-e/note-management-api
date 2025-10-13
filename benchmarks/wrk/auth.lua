-- wrk benchmark script for authentication endpoints
-- Tests login performance

-- Global counter for tracking requests
local counter = 0
local threads = {}

-- Setup function - called once per thread
function setup(thread)
    thread:set("id", counter)
    table.insert(threads, thread)
    counter = counter + 1
end

-- Initialize function - called once per thread before starting
function init(args)
    -- Credentials for testing
    requests = 0
    responses = 0
    
    -- JSON payload for login
    body = [[{
        "email": "test@example.com",
        "password": "SecurePass123!"
    }]]
end

-- Request function - called for each request
function request()
    requests = requests + 1
    
    -- Add headers
    wrk.method = "POST"
    wrk.headers["Content-Type"] = "application/json"
    wrk.body = body
    
    return wrk.format(nil, "/api/v1/auth/login")
end

-- Response function - called for each response
function response(status, headers, body)
    responses = responses + 1
    
    -- Track errors
    if status >= 400 then
        wrk.thread:set("errors", (wrk.thread:get("errors") or 0) + 1)
    end
    
    -- Track successful logins
    if status == 200 then
        wrk.thread:set("success", (wrk.thread:get("success") or 0) + 1)
    end
end

-- Done function - called when all requests complete
function done(summary, latency, requests)
    io.write("\n--- Authentication Benchmark Results ---\n")
    io.write(string.format("Total Requests: %d\n", summary.requests))
    io.write(string.format("Total Duration: %.2fs\n", summary.duration / 1000000))
    io.write(string.format("Requests/sec: %.2f\n", summary.requests / (summary.duration / 1000000)))
    io.write(string.format("Avg Latency: %.2fms\n", latency.mean / 1000))
    io.write(string.format("Max Latency: %.2fms\n", latency.max / 1000))
    io.write(string.format("Stdev Latency: %.2fms\n", latency.stdev / 1000))
    
    -- Aggregate error counts
    local errors = 0
    local success = 0
    for i, thread in ipairs(threads) do
        errors = errors + (thread:get("errors") or 0)
        success = success + (thread:get("success") or 0)
    end
    
    io.write(string.format("\nSuccessful Logins: %d\n", success))
    io.write(string.format("Errors: %d (%.2f%%)\n", errors, (errors / summary.requests) * 100))
    io.write("----------------------------------------\n")
end

