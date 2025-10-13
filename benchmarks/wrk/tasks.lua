-- wrk benchmark script for task endpoints
-- Tests task listing with pagination and filters

local counter = 0
local threads = {}
local token = os.getenv("AUTH_TOKEN") or "your-test-token-here"

function setup(thread)
    thread:set("id", counter)
    table.insert(threads, thread)
    counter = counter + 1
end

function init(args)
    requests = 0
    responses = 0
    
    -- Various query patterns to test
    paths = {
        "/api/v1/tasks",                                    -- Simple list
        "/api/v1/tasks?page=1&limit=20",                   -- Pagination
        "/api/v1/tasks?status=todo",                       -- Filter by status
        "/api/v1/tasks?page=1&limit=10&sort_by=created_at&sort_direction=desc",  -- Complex query
        "/api/v1/tasks?search=test",                       -- Search
    }
    
    path_index = 1
end

function request()
    requests = requests + 1
    
    -- Rotate through different query patterns
    local path = paths[path_index]
    path_index = (path_index % #paths) + 1
    
    -- Add authentication header
    wrk.headers["Authorization"] = "Bearer " .. token
    wrk.headers["Content-Type"] = "application/json"
    
    return wrk.format("GET", path)
end

function response(status, headers, body)
    responses = responses + 1
    
    if status >= 400 then
        wrk.thread:set("errors", (wrk.thread:get("errors") or 0) + 1)
    elseif status == 200 then
        wrk.thread:set("success", (wrk.thread:get("success") or 0) + 1)
    end
end

function done(summary, latency, requests)
    io.write("\n--- Task Listing Benchmark Results ---\n")
    io.write(string.format("Total Requests: %d\n", summary.requests))
    io.write(string.format("Total Duration: %.2fs\n", summary.duration / 1000000))
    io.write(string.format("Requests/sec: %.2f\n", summary.requests / (summary.duration / 1000000)))
    io.write(string.format("Avg Latency: %.2fms\n", latency.mean / 1000))
    io.write(string.format("P50 Latency: %.2fms\n", latency:percentile(50) / 1000))
    io.write(string.format("P95 Latency: %.2fms\n", latency:percentile(95) / 1000))
    io.write(string.format("P99 Latency: %.2fms\n", latency:percentile(99) / 1000))
    io.write(string.format("Max Latency: %.2fms\n", latency.max / 1000))
    
    local errors = 0
    local success = 0
    for i, thread in ipairs(threads) do
        errors = errors + (thread:get("errors") or 0)
        success = success + (thread:get("success") or 0)
    end
    
    io.write(string.format("\nSuccessful Requests: %d\n", success))
    io.write(string.format("Errors: %d (%.2f%%)\n", errors, (errors / summary.requests) * 100))
    io.write("---------------------------------------\n")
end

