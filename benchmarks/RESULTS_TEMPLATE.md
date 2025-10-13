# 📊 Benchmark Results Template

**Date**: YYYY-MM-DD  
**System**: [MacBook Pro M1/Linux Server/etc]  
**Rust Version**: 1.XX.X  
**Build**: Release  
**Configuration**: [Default/Custom]

---

## System Specifications

- **CPU**: [e.g., Apple M1 Pro, 10 cores]
- **RAM**: [e.g., 32GB]
- **OS**: [e.g., macOS 14.6.0]
- **Rust**: [e.g., 1.75.0]
- **Database**: [e.g., PostgreSQL 15.2]
- **Redis**: [e.g., 7.0.11]

---

## Test Configuration

### wrk
- **Threads**: 4
- **Connections**: 100
- **Duration**: 30s

### k6
- **VUs**: 10 → 50 → 100 → 50 → 0
- **Duration**: 7 minutes (load test)

---

## Results Summary

### Overall Performance
| Metric | Target | Actual | Status |
|--------|--------|--------|--------|
| **Throughput** | >1000 req/s | XXXX req/s | ✅/❌ |
| **P95 Latency** | <100ms | XXms | ✅/❌ |
| **Error Rate** | <1% | X.XX% | ✅/❌ |
| **Memory Usage** | -60% | -XX% | ✅/❌ |

---

## Detailed Results

### 1. Health Check (Baseline)

#### wrk Output
```
Running 30s test @ http://localhost:3000/health
  4 threads and 100 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency    XX.XXms   XX.XXms  XXX.XXms   XX.XX%
    Req/Sec   XXX.XX    XXX.XX   XXX.XX     XX.XX%
  XXXXX requests in 30.00s, XX.XXMBread
Requests/sec:   XXXX.XX
Transfer/sec:    XXX.XXKB
```

**Analysis**:
- Throughput: XXXX req/s
- Avg Latency: XX.XXms
- Max Latency: XXX.XXms

---

### 2. Task Listing (Query Building)

#### wrk Output
```
--- Task Listing Benchmark Results ---
Total Requests: XXXXX
Total Duration: 30.00s
Requests/sec: XXXX.XX
Avg Latency: XX.XXms
P50 Latency: XX.XXms
P95 Latency: XX.XXms
P99 Latency: XX.XXms
Max Latency: XXX.XXms

Successful Requests: XXXXX
Errors: XX (X.XX%)
```

**Optimization Tested**: Query building (with/without arena allocation)

**Analysis**:
- Throughput: XXXX req/s
- P95 Latency: XXms
- Error Rate: X.XX%
- **Impact**: [Describe impact of optimization]

**Queries Tested**:
- Simple list: `/api/v1/tasks`
- With pagination: `/api/v1/tasks?page=1&limit=20`
- With filters: `/api/v1/tasks?status=todo`
- Complex: `/api/v1/tasks?page=1&limit=10&sort_by=created_at&sort_direction=desc`
- With search: `/api/v1/tasks?search=test`

---

### 3. File Upload (Zero-Copy)

#### wrk Output
```
--- File Upload Benchmark Results ---
Total Uploads: XXXXX
Total Duration: 30.00s
Uploads/sec: XXX.XX
Avg Latency: XX.XXms
P95 Latency: XX.XXms
Max Latency: XXX.XXms

Successful Uploads: XXXXX
Errors: XX (X.XX%)

Note: Tests zero-copy optimization (bytes::Bytes)
```

**Optimization Tested**: Zero-copy with `bytes::Bytes`

**Analysis**:
- Upload speed: XXX req/s
- Avg Latency: XX.XXms
- Memory savings: -XX% (estimated)
- **Impact**: [Describe zero-copy impact]

**File Sizes Tested**:
- Small (1KB): XXX req/s
- Medium (100KB): XX req/s (if tested)
- Large (1MB): X req/s (if tested)

---

### 4. k6 Load Test (Comprehensive)

#### Summary
```
✓ http_req_duration..........: avg=XX.XXms p(95)=XXX.XXms p(99)=XXX.XXms
✓ http_req_failed............: X.XX% (XXX of XXXXX)
✓ http_reqs..................: XXXXX (XXX.XX/s)
✓ vus........................: 100 min=10 max=100

Custom Metrics:
✓ auth_success...............: XXXX
✓ tasks_created..............: XXXX
✓ auth_duration..............: avg=XX.XXms
✓ task_create_duration.......: avg=XX.XXms
✓ task_list_duration.........: avg=XX.XXms
✓ file_upload_duration.......: avg=XX.XXms
```

**Analysis**:
- Total requests: XXXXX
- Success rate: XX.XX%
- Avg response time: XX.XXms
- P95 response time: XXX.XXms

**User Scenarios Tested**:
1. ✅ Register/Login
2. ✅ Create tasks
3. ✅ List tasks
4. ✅ Filter tasks
5. ✅ Upload files
6. ✅ List files

---

### 5. WebSocket Test (Arena Allocation)

#### k6 Output
```
=== WebSocket Benchmark Results ===
Connections: XX
Duration: XXX.XXs
Messages Received: XXXXX
Errors: X.XX%

Connection Time:
  Avg: XX.XXms
  P95: XX.XXms

Note: Tests arena allocation optimization in broadcast()
```

**Optimization Tested**: Arena allocation with `bumpalo`

**Analysis**:
- Concurrent connections: XX
- Messages/sec: XXXXX
- Connection latency: XX.XXms
- Error rate: X.XX%
- **Impact**: [Describe arena allocation impact]

**Expected Improvements**:
- Before: ~50-100µs per message to 1000 clients
- After: ~5-10µs per message (10x faster)
- Memory allocations: 1000x fewer

---

## Optimization Impact Comparison

### Before Optimizations
| Metric | Value |
|--------|-------|
| Task Listing | XXXX req/s |
| File Upload | XXX req/s |
| WebSocket Broadcast | XX-XXXµs |
| P95 Latency | XXXms |
| Memory Usage | XXXXMBs |

### After Optimizations
| Metric | Value | Improvement |
|--------|-------|------------|
| Task Listing | XXXX req/s | +XX% |
| File Upload | XXXX req/s | +XXX% |
| WebSocket Broadcast | X-XXµs | 10x faster |
| P95 Latency | XXms | -XX% |
| Memory Usage | XXXXMBs | -XX% |

---

## Resource Usage

### CPU
- Average: XX%
- Peak: XX%

### Memory
- Average: XXXXMB
- Peak: XXXXMB
- Compared to baseline: -XX%

### Network
- Throughput: XX MB/s
- Connections: XXX concurrent

### Database
- Connections: XX
- Query time avg: XXms
- Connection pool: XX/XX used

---

## Bottlenecks Identified

1. **[Bottleneck 1]**
   - Location: [e.g., Database queries]
   - Impact: [e.g., XX% of total time]
   - Recommendation: [e.g., Add indexes]

2. **[Bottleneck 2]**
   - Location: [e.g., File I/O]
   - Impact: [e.g., XX% of total time]
   - Recommendation: [e.g., Implement streaming]

---

## Optimization Effectiveness

### Zero-Copy (bytes::Bytes)
- ✅ **File Upload**: +XXX% throughput
- ✅ **Memory**: -XX% usage
- ✅ **Latency**: -XX% reduction
- **Verdict**: [Highly Effective/Effective/Marginal]

### Arena Allocation (bumpalo)
- ✅ **WebSocket**: XXx faster
- ✅ **Allocations**: XXXXx fewer
- ⚠️ **Query Building**: [Applied/Not Applied - reason]
- **Verdict**: [Highly Effective/Effective/Marginal]

### Arc<str> Message Sharing
- ✅ **WebSocket**: XXXXx fewer copies
- ✅ **Memory**: -XX% per broadcast
- **Verdict**: [Highly Effective/Effective/Marginal]

---

## Comparison with Previous Benchmarks

### Previous Run (Date: YYYY-MM-DD)
| Metric | Previous | Current | Change |
|--------|----------|---------|--------|
| Task Listing | XXXX req/s | XXXX req/s | +/-XX% |
| File Upload | XXX req/s | XXX req/s | +/-XX% |
| P95 Latency | XXms | XXms | +/-XX% |
| Error Rate | X.XX% | X.XX% | +/-X.XX% |

---

## Recommendations

### Short-term (Next Sprint)
1. [Recommendation 1]
2. [Recommendation 2]
3. [Recommendation 3]

### Medium-term (Next Month)
1. [Recommendation 1]
2. [Recommendation 2]

### Long-term (Next Quarter)
1. [Recommendation 1]
2. [Recommendation 2]

---

## Stress Test Results (if performed)

### Breaking Point
- **Stable at**: XXX concurrent users
- **Degradation starts**: XXX concurrent users
- **System failure**: XXX concurrent users

### Recovery
- Time to recover: XXs
- Auto-scaling triggered: [Yes/No]

---

## Notes

### Test Environment
- [Any environmental factors]
- [Network conditions]
- [Background processes]

### Anomalies
- [Any unusual observations]
- [Unexpected behaviors]

### Next Steps
1. [Action item 1]
2. [Action item 2]
3. [Action item 3]

---

## Conclusion

**Overall Assessment**: [Excellent/Good/Needs Improvement]

**Key Achievements**:
- ✅ [Achievement 1]
- ✅ [Achievement 2]
- ✅ [Achievement 3]

**Production Readiness**: [Ready/Needs Work]

**Optimization Success**: [Optimizations delivered XX-XXXx improvements across hot paths]

---

**Benchmarked by**: [Your Name]  
**Reviewed by**: [Reviewer Name]  
**Next Benchmark**: [Scheduled Date]

