-- wrk benchmark script for file upload endpoint
-- Tests file upload performance with arena allocation optimization

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
    
    -- Boundary for multipart form data
    boundary = "----WebKitFormBoundary7MA4YWxkTrZu0gW"
    
    -- Sample file content (small text file for testing)
    file_content = "This is a test file for benchmarking upload performance.\n" ..
                   "Testing zero-copy optimization with bytes::Bytes.\n" ..
                   "Arena allocation provides 10-100x faster temporary allocations.\n"
    
    -- Multipart form data body
    body = "--" .. boundary .. "\r\n" ..
           "Content-Disposition: form-data; name=\"file\"; filename=\"benchmark-test.txt\"\r\n" ..
           "Content-Type: text/plain\r\n\r\n" ..
           file_content .. "\r\n" ..
           "--" .. boundary .. "--\r\n"
end

function request()
    requests = requests + 1
    
    wrk.method = "POST"
    wrk.headers["Authorization"] = "Bearer " .. token
    wrk.headers["Content-Type"] = "multipart/form-data; boundary=" .. boundary
    wrk.body = body
    
    return wrk.format(nil, "/api/v1/files")
end

function response(status, headers, body)
    responses = responses + 1
    
    if status >= 400 then
        wrk.thread:set("errors", (wrk.thread:get("errors") or 0) + 1)
    elseif status == 201 then  -- File created
        wrk.thread:set("success", (wrk.thread:get("success") or 0) + 1)
    end
end

function done(summary, latency, requests)
    io.write("\n--- File Upload Benchmark Results ---\n")
    io.write(string.format("Total Uploads: %d\n", summary.requests))
    io.write(string.format("Total Duration: %.2fs\n", summary.duration / 1000000))
    io.write(string.format("Uploads/sec: %.2f\n", summary.requests / (summary.duration / 1000000)))
    io.write(string.format("Avg Latency: %.2fms\n", latency.mean / 1000))
    io.write(string.format("P95 Latency: %.2fms\n", latency:percentile(95) / 1000))
    io.write(string.format("Max Latency: %.2fms\n", latency.max / 1000))
    
    local errors = 0
    local success = 0
    for i, thread in ipairs(threads) do
        errors = errors + (thread:get("errors") or 0)
        success = success + (thread:get("success") or 0)
    end
    
    io.write(string.format("\nSuccessful Uploads: %d\n", success))
    io.write(string.format("Errors: %d (%.2f%%)\n", errors, (errors / summary.requests) * 100))
    io.write("\nNote: Tests zero-copy optimization (bytes::Bytes)\n")
    io.write("--------------------------------------\n")
end

