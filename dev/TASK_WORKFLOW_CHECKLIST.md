# ✅ CHECKLIST QUY TRÌNH THỰC THI TASK

**Quick Reference Card** - In ra hoặc bookmark để dùng cho mỗi task

---

## 📋 5 BƯỚC BẮT BUỘC

### **BƯỚC 1: ĐỌC & HIỂU** ⏱️ 10-15%

```markdown
## Bước 1 Checklist

### 1.1. Phân Tích Yêu Cầu
- [ ] Đọc kỹ mô tả task từ đầu đến cuối
- [ ] Highlight keywords (must-have, constraints)
- [ ] Xác định deliverables cụ thể
- [ ] Note ambiguities để hỏi lại

### 1.2. Xác Định Files Liên Quan
- [ ] Search codebase tìm files liên quan
- [ ] Đọc documentation modules
- [ ] Xác định dependencies
- [ ] List test files cần update

### 1.3. Đánh Giá Rủi Ro
- [ ] Phân loại risk (Low/Medium/High)
- [ ] Xác định breaking changes potential
- [ ] Note side effects
- [ ] Lên rollback plan

## Output Bước 1
- [ ] Task Understanding document
- [ ] Affected Files list
- [ ] Risk Assessment
```

---

### **BƯỚC 2: TRIỂN KHAI** ⏱️ 40-50%

```markdown
## Bước 2 Checklist

### 2.1. Tạo Todo List
- [ ] Break task thành atomic sub-tasks
- [ ] Estimate thời gian mỗi sub-task
- [ ] Xác định dependencies
- [ ] Tạo todo list với todowrite tool

### 2.2. Implement
- [ ] Đánh dấu in_progress trước khi làm
- [ ] Commit nhỏ, thường xuyên
- [ ] Chạy tests sau mỗi thay đổi lớn
- [ ] Đánh dấu completed ngay khi xong

### 2.3. Verify
- [ ] Code compile/build thành công
- [ ] Tests pass (existing + new)
- [ ] Linting clean
- [ ] Type checking pass
- [ ] Manual testing (nếu cần)

## Output Bước 2
- [ ] Todo list với status
- [ ] Implementation log
- [ ] Test results
```

---

### **BƯỚC 3: AUDIT** ⏱️ 15-20%

```markdown
## Bước 3 Checklist

### 3.1. Chuẩn Bị Audit
- [ ] Thu thập files đã thay đổi
- [ ] List changes cần verify
- [ ] Xác định audit criteria

### 3.2. Gọi Subagent Audit
- [ ] Viết audit request với đầy đủ context
- [ ] List success criteria rõ ràng
- [ ] Specify required output format

### 3.3. Thu Thập Bằng Chứng
- [ ] Command outputs (grep, find, etc.)
- [ ] Test results (pass/fail counts)
- [ ] Build output (success/failure)
- [ ] File listings (before/after)
- [ ] Screenshots (UI changes)
- [ ] Logs (runtime behavior)

### 3.4. Báo Cáo Audit
- [ ] Summary (PASS/FAIL)
- [ ] Findings by severity
- [ ] Verification results table
- [ ] Conclusion & recommendations

## Output Bước 3
- [ ] Audit Report
- [ ] Evidence Index
- [ ] Issues List (nếu có)
```

---

### **BƯỚC 4: KHẮC PHỤC** ⏱️ 15-20%

```markdown
## Bước 4 Checklist

### 4.1. Phân Loại Issues
- [ ] Priority P0 (Critical)
- [ ] Priority P1 (High)
- [ ] Priority P2 (Medium)
- [ ] Priority P3 (Low)

### 4.2. Fix Issues
- [ ] Tạo todo cho từng issue
- [ ] Fix theo priority (P0 → P3)
- [ ] Verify fix với tests
- [ ] Update implementation log

### 4.3. Re-Audit
- [ ] Fix xong TẤT CẢ issues
- [ ] Verify locally các fixes
- [ ] Gọi lại subagent audit
- [ ] Get PASS confirmation

## Output Bước 4
- [ ] Issue Tracking log
- [ ] Fix verification evidence
- [ ] Re-audit report
```

---

### **BƯỚC 5: TỔNG HỢP** ⏱️ 10-15%

```markdown
## Bước 5 Checklist

### 5.1. Báo Cáo Hoàn Thành
- [ ] Executive Summary
- [ ] Task Details
- [ ] What Was Done
- [ ] Files Changed
- [ ] Metrics
- [ ] Verification Summary
- [ ] Known Issues
- [ ] Next Steps

### 5.2. Bằng Chứng
- [ ] Code Evidence
- [ ] Test Evidence
- [ ] Build Evidence
- [ ] Documentation Evidence
- [ ] Runtime Evidence

### 5.3. Lưu Trữ
- [ ] Reports (Markdown)
- [ ] Logs (execution, audit)
- [ ] Screenshots
- [ ] Test results
- [ ] Backup files

## Output Bước 5
- [ ] Completion Report
- [ ] Evidence Index
- [ ] Artifacts Archive
```

---

## 🎯 QUALITY GATES

### Gate 1: Trước Khi Implement
```
✓ Task understanding clear
✓ Affected files identified
✓ Risk assessment done
✓ Todo list created
```

### Gate 2: Trước Khi Audit
```
✓ All code implemented
✓ Build passes
✓ Tests pass
✓ Linting clean
```

### Gate 3: Trước Khi Deliver
```
✓ Audit PASS
✓ All issues fixed
✓ Documentation complete
✓ Evidence collected
```

---

## 📊 TEMPLATES

### Task Understanding Template
```markdown
## Task: [Name]

**Goal:** [1-2 sentences]

**Deliverables:**
- [ ] D1
- [ ] D2

**Constraints:**
- [C1]
- [C2]

**Questions:**
- [Q1]
```

### Audit Request Template
```markdown
# Audit Request

## Context
[Summary]

## Changes
[File list]

## Scope
- [ ] Code review
- [ ] Tests
- [ ] Docs

## Criteria
[List]

## Required Output
[List]
```

### Completion Report Template
```markdown
# Completion Report

## Summary
[2-3 sentences]

## Results
- ✅ Deliverable 1
- ✅ Deliverable 2

## Metrics
| Metric | Target | Actual |
|--------|--------|--------|
| M1 | V | V |

## Evidence
1. [Evidence 1](#link)
2. [Evidence 2](#link)
```

---

## ⚡ QUICK COMMANDS

### Search & Verify
```bash
# Find files with pattern
grep -r "pattern" --include="*.rs" .

# Count occurrences
grep -c "pattern" file.txt

# Find files
find . -name "*.rs" -path "*/module/*"

# Check build
cargo build --release 2>&1 | tail -10

# Run tests
cargo test 2>&1 | tail -20
```

### Git Operations
```bash
# See changed files
git status

# View diff
git diff

# Create commit
git commit -m "feat: description"
```

---

## 🚨 RED FLAGS

### Dừng Lại Khi:
- ❌ Requirements không clear sau 2 lần hỏi
- ❌ Technical risk quá cao không đánh giá được
- ❌ Timeline không khả thi
- ❌ Missing critical information
- ❌ Dependencies không available

### Escalate Khi:
- ⚠️ Blocker > 2 hours
- ⚠️ Need architectural decision
- ⚠️ Security concerns
- ⚠️ Breaking changes required

---

## 📈 METRICS TO TRACK

### Quality
- Defect Rate: < 5%
- Test Coverage: > 80%
- Audit Pass Rate: 100%

### Efficiency
- On-Time Delivery: > 90%
- First-Time Right: > 80%
- Rework Rate: < 20%

---

## 🔄 CONTINUOUS IMPROVEMENT

### Sau Mỗi Task, Tự Hỏi:
1. ✅ What went well?
2. ⚠️ What could be improved?
3. 💡 What lessons learned?
4. 🔄 What to do differently next time?

### Update SOP Khi:
- Tìm ra best practice mới
- Phát hiện gap trong process
- Nhận feedback từ stakeholders
- Có tool/template tốt hơn

---

**Version:** 1.0  
**Last Updated:** 2026-03-15  
**Next Review:** Sau 10 tasks

---

## 📎 APPENDIX: EXAMPLE AUDIT PROMPT

```markdown
# Audit Request

## Task Context
[Task name and goal]

## Implementation Summary
- Files changed: X
- Lines added/removed: +Y/-Z
- Key changes: [Brief description]

## Audit Scope
- [x] Code correctness
- [x] Test coverage
- [x] Documentation
- [x] Security
- [x] Performance

## Success Criteria
1. [Criterion 1]
2. [Criterion 2]
3. [Criterion 3]

## Required Evidence
1. Grep/search results
2. Test output
3. Build status
4. File listings

## Output Format
- Issue list with severity
- Evidence for each finding
- Pass/Fail recommendation
```
