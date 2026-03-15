# 📘 HƯỚNG DẪN SỬ DỤNG WORKFLOW SOP

**File chính:** `WORKFLOW_SOP.md`  
**Version:** 2.0 (Enhanced Step 1)  
**Updated:** 2026-03-15

---

## 📚 CẤU TRÚC TÀI LIỆU

```
dev/
├── WORKFLOW_SOP.md              # ✅ Tài liệu chính (đầy đủ)
├── TASK_WORKFLOW_CHECKLIST.md   # ✅ Checklist nhanh
├── WORKFLOW_EXAMPLE.md          # ✅ Ví dụ thực tế
└── HUONG_DAN_SU_DUNG_WORKFLOW.md # ✅ File này
```

---

## 🎯 KHI NÀO SỬ DỤNG

### **Khi Bạn Giao Task:**

Nói đơn giản:
```
"[Mô tả task]. Áp dụng workflow SOP."
```

**Ví dụ:**
```
"Tạo API endpoint mới cho user management. Áp dụng workflow SOP."
```

### **Khi Bạn Nhận Task:**

1. Mở `WORKFLOW_SOP.md`
2. Đọc **BƯỚC 1: ĐỌC & HIỂU TASK**
3. Follow 6 sub-steps (1.1 → 1.6)
4. Điền output template
5. Proceed to Step 2

---

## 📋 CÁC BƯỚC THỰC HIỆN

### **TÓM TẮT 5 BƯỚC**

```
BƯỚC 1: ĐỌC & HIỂU (10-20%)
  ↓
BƯỚC 2: TRIỂN KHAI (40-50%)
  ↓
BƯỚC 3: AUDIT (15-20%)
  ↓
BƯỚC 4: KHẮC PHỤC (15-20%)
  ↓
BƯỚC 5: TỔNG HỢP (10-15%)
```

---

### **BƯỚC 1: ĐỌC & HIỂU TASK** ⭐ **QUAN TRỌNG NHẤT**

**Thời gian:** 10-20% tổng thời gian task

**6 Sub-steps:**

#### **1.1. Phân Loại Task** (2 phút)
```
Question: Có code changes không?
→ KHÔNG → 🟢 LOW RISK
→ CÓ → Có breaking changes không?
         → KHÔNG → 🟡 MEDIUM RISK
         → CÓ → 🔴 HIGH RISK
```

**Output:**
```markdown
## Task Classification

**Type:** 🟡 MEDIUM
**Reasoning:** Code changes, multiple files, tests needed
**Workflow:** Standard 5-step
**Time:** 2 hours
```

---

#### **1.2. Chọn Skills** (2 phút)

**Quick Reference:**

| Task Type | Skills | Category |
|-----------|--------|----------|
| Git operations | `git-master` | quick |
| Web UI/Styling | `frontend-ui-ux` | visual-engineering |
| Browser testing | `playwright` | quick |
| Codebase research | `explore` agent | background |
| External docs | `librarian` agent | background |
| Complex architecture | `oracle` agent | consultation |
| Creative solutions | `artistry` agent | non-conventional |

**Output:**
```markdown
## Skills Required

**Primary:**
- git-master (for commits)
- frontend-ui-ux (for styling)

**Load Commands:**
task(category="visual-engineering", load_skills=["frontend-ui-ux"])
```

---

#### **1.3. Tìm Files** (5 phút)

**Search Commands:**

```bash
# Rust code
find src -name "*.rs" -path "*/module/*"
grep -r "pattern" --include="*.rs" src/

# TypeScript/React
find apps/web -name "*.tsx" | xargs grep -l "Component"

# Configuration
find . -name "*.toml" -o -name "*.yaml"

# Documentation
find docs -name "*.md" | xargs grep -l "topic"
```

**Output:**
```markdown
## Affected Files

**Core (modify):**
- src/api/users.rs (new endpoint)
- src/models/user.rs (add fields)

**Tests (update):**
- tests/api_users.rs (integration tests)

**Docs (review):**
- docs/api/users.md
```

---

#### **1.4. Phân Tích Requirements** (5 phút)

**5W1H Framework:**

```
WHAT: Task yêu cầu làm gì?
WHY: Tại sao cần làm?
WHO: Ai sẽ sử dụng?
WHEN: Khi nào cần?
WHERE: Ở đâu áp dụng?
HOW: Làm như thế nào?
```

**Output:**
```markdown
## Requirements Analysis

**WHAT:** Add GET /api/users/:id endpoint
**WHY:** Frontend needs user profile page
**WHO:** Frontend developers, end users
**WHEN:** This sprint
**WHERE:** API server, user module
**HOW:** Rust, Axum framework, JWT auth
```

---

#### **1.5. Đánh Giá Rủi Ro** (3 phút)

**Risk Calculation:**
```
Score = (Code × 1) + (Tests × 2) + (Breaking × 3) + 
        (Rollback × 2) + (Users × 2) + (Security × 4)

1-5:   🟢 LOW
6-12:  🟡 MEDIUM
13+:   🔴 HIGH
```

**Output:**
```markdown
## Risk Assessment

**Risk Level:** 🟡 MEDIUM
**Risk Score:** 8/20

**Factors:**
- Code: 3 files → 2 points
- Tests: Need some → 2 points
- Breaking: None → 0 points
- Rollback: Simple → 1 point
- Users: Some → 2 points
- Security: Low → 1 point
```

---

#### **1.6. Output Template** (3 phút)

**Điền vào template đầy đủ:**

```markdown
# 📋 BƯỚC 1: ĐỌC & HIỂU TASK

## 1. Classification
- Type: 🟡 MEDIUM
- Workflow: Standard 5-step
- Time: 2 hours

## 2. Skills
- git-master, frontend-ui-ux

## 3. Requirements
- 5W1H answered...

## 4. Files
- src/api/users.rs
- tests/api_users.rs

## 5. Risk
- Score: 8/20 → MEDIUM

## ✅ Ready to Proceed
```

---

### **BƯỚC 2-5: TÓM TẮT**

#### **BƯỚC 2: TRIỂN KHAI** (40-50%)
```
1. Tạo todo list với todowrite tool
2. Implement từng bước
3. Verify sau mỗi bước (build, test, lint)
```

#### **BƯỚC 3: AUDIT** (15-20%)
```
1. Gọi subagent audit
2. Thu thập bằng chứng (grep, build output, tests)
3. Báo cáo PASS/FAIL
```

#### **BƯỚC 4: KHẮC PHỤC** (15-20%)
```
1. Nếu có issues → Fix
2. Re-audit
3. Get PASS confirmation
```

#### **BƯỚC 5: TỔNG HỢP** (10-15%)
```
1. Viết completion report
2. Liệt kê bằng chứng
3. Lưu trữ artifacts
```

---

## 🎯 VÍ DỤ THỰC TẾ

### **Example 1: Simple Bug Fix**

**Task:** "Fix null pointer in user service"

**Step 1 Output:**
```markdown
## 1. Classification
- Type: 🟡 MEDIUM (code change, tests needed)
- Workflow: Standard 5-step
- Time: 1 hour

## 2. Skills
- git-master

## 3. Files
- src/services/user.rs (fix bug)
- tests/user_service.rs (add test)

## 4. Risk
- Score: 6/20 → MEDIUM

✅ Proceed to Step 2
```

---

### **Example 2: New Feature**

**Task:** "Add user profile endpoint"

**Step 1 Output:**
```markdown
## 1. Classification
- Type: 🟡 MEDIUM (new feature, multiple files)
- Workflow: Standard 5-step
- Time: 4 hours

## 2. Skills
- git-master
- explore agent (find patterns)

## 3. Files
- src/api/users.rs (new endpoint)
- src/models/user.rs (add fields)
- tests/api_users.rs (tests)
- docs/api/users.md (docs)

## 4. Risk
- Score: 10/20 → MEDIUM

✅ Proceed to Step 2
```

---

### **Example 3: Security Patch**

**Task:** "Update authentication to OAuth2"

**Step 1 Output:**
```markdown
## 1. Classification
- Type: 🔴 HIGH (security, breaking changes)
- Workflow: Enhanced 7-step
- Time: 2 days

## 2. Skills
- oracle agent (security review)
- git-master
- explore agent

## 3. Files
- src/auth/*.rs (multiple files)
- src/config/schema.rs
- tests/auth/*.rs
- docs/security/*.md

## 4. Risk
- Score: 18/20 → HIGH

⚠️ Need stakeholder approval before Step 2
```

---

## 🔧 QUICK COMMANDS

### **Search Files:**
```bash
# Rust
find src -name "*.rs" -path "*/module/*"
grep -r "pattern" --include="*.rs" src/

# TypeScript
find apps/web -name "*.tsx" | xargs grep -l "Component"

# Config
find . -name "*.toml" -o -name "*.yaml"

# Docs
find docs -name "*.md" | xargs grep -l "topic"
```

### **Verify Changes:**
```bash
# Check for pattern
grep -c "pattern" file.txt

# Build
cargo build --release 2>&1 | tail -10

# Test
cargo test 2>&1 | tail -20
```

---

## 📊 METRICS

### **Quality Metrics:**
- Requirements missed: < 5%
- Wrong files modified: < 2%
- Rework needed: < 5%
- First-time-right: > 90%

### **Efficiency Metrics:**
- Step 1 time: 10-20% total
- On-time delivery: > 90%
- Audit pass rate: 100%

---

## ⚠️ LƯU Ý QUAN TRỌNG

### **Khi Nào Hỏi Lại:**
- Requirements không clear sau 2 lần hỏi
- Technical risk quá cao
- Timeline không khả thi
- Missing critical information

### **Khi Nào Tự Làm:**
- Requirements clear
- Files xác định được
- Risk Low/Medium
- Có thể audit được

---

## 📎 TÀI LIỆU LIÊN QUAN

1. **WORKFLOW_SOP.md** - Full SOP (28KB)
2. **TASK_WORKFLOW_CHECKLIST.md** - Quick checklist (6.9KB)
3. **WORKFLOW_EXAMPLE.md** - Real example (13KB)
4. **HUONG_DAN_SU_DUNG_WORKFLOW.md** - File này

---

## 🚀 BẮT ĐẦU NGAY

### **30-Second Guide:**

1. Nhận task → Mở `WORKFLOW_SOP.md`
2. Đến **BƯỚC 1** → Follow 6 sub-steps
3. Điền output template
4. ✅ Proceed to Step 2

### **5-Minute Guide:**

1. Read Step 1 section (2 min)
2. Classify task (2 min)
3. Select skills & find files (1 min)

### **10-Minute Guide:**

1. Complete all 6 sub-steps
2. Fill output template
3. Get approval if needed
4. Start implementation

---

**Version:** 2.0  
**Created:** 2026-03-15  
**Status:** Ready to Use ✅  
**Next Review:** Sau 20 tasks
