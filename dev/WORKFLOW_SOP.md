# 📋 QUY TRÌNH THỰC THI NHIỆM VỤ CHUẨN (SOP)

**Version:** 1.0  
**Effective Date:** 2026-03-15  
**Scope:** Áp dụng cho tất cả nhiệm vụ phát triển, refactoring, và bảo trì

---

## 🎯 MỤC ĐÍCH

Tài liệu này định nghĩa quy trình chuẩn 5 bước để đảm bảo:
- ✅ Hiểu đúng yêu cầu trước khi thực hiện
- ✅ Thực hiện đầy đủ và chính xác
- ✅ Xác minh độc lập với bằng chứng cụ thể
- ✅ Khắc phục triệt để các vấn đề
- ✅ Tài liệu hóa minh bạch và khoa học

---

## 📊 TỔNG QUAN QUY TRÌNH

```
┌─────────────────────────────────────────────────────────────┐
│  BƯỚC 1: ĐỌC & HIỂU TASK                                    │
│  - Phân tích yêu cầu                                        │
│  - Xác định files liên quan                                 │
│  - Đánh giá rủi ro                                          │
└─────────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────────┐
│  BƯỚC 2: TRIỂN KHAI THỰC HIỆN                                │
│  - Tạo todo list chi tiết                                   │
│  - Implement từng bước                                      │
│  - Verify sau mỗi bước                                      │
└─────────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────────┐
│  BƯỚC 3: AUDIT ĐỘC LẬP (Subagent)                           │
│  - Gọi subagent audit                                       │
│  - Thu thập bằng chứng xác minh                             │
│  - Báo cáo kết quả                                          │
└─────────────────────────────────────────────────────────────┘
                           ↓
                    ┌──────────────┐
                    │  CÓ LỖI?     │
                    └──────────────┘
                      ↓          ↓
                    CÓ          KHÔNG
                      ↓          ↓
┌──────────────────────┐    ┌──────────────────────────────────┐
│  BƯỚC 4: KHẮC PHỤC    │    │  BƯỚC 5: TỔNG HỢP TÀI LIỆU      │
│  - Fix issues         │    │  - Viết báo cáo hoàn thành      │
│  - Quay lại Bước 3    │    │  - Liệt kê bằng chứng           │
└──────────────────────┘    │  - Lưu trữ artifacts            │
                            └──────────────────────────────────┘
```

---

## 📝 CHI TIẾT CÁC BƯỚC

### **BƯỚC 1: ĐỌC & HIỂU TASK** ⏱️ 10-20% thời gian

> **MỤC TIÊU:** Hiểu rõ task đến mức có thể giải thích cho người khác trong 2 phút

---

#### 1.1. PHÂN LOẠI TASK (Task Classification) ⭐ **MỚI**

**Hành động:**
- [ ] Đọc task description 2 lần (lần 1: tổng quan, lần 2: chi tiết)
- [ ] Highlight keywords: must-have, nice-to-have, constraints, deadlines
- [ ] Phân loại theo ma trận dưới đây
- [ ] Chọn workflow phù hợp

**Task Classification Matrix:**

```
┌──────────────────────────────────────────────────────────────────┐
│  TASK TYPE          │ INDICATORS              │ WORKFLOW        │
├──────────────────────────────────────────────────────────────────┤
│  🟢 LOW RISK        │                         │                 │
│  - Docs update      │ - No code changes       │ Light 3-step    │
│  - Typo fixes       │ - Single file           │ - Understand    │
│  - Minor config     │ - No tests needed       │ - Implement     │
│  - Comment updates  │ - No breaking changes   │ - Self-verify   │
├──────────────────────────────────────────────────────────────────┤
│  🟡 MEDIUM RISK     │                         │                 │
│  - New feature      │ - Multiple files        │ Standard 5-step │
│  - Bug fix          │ - Tests needed          │ - Understand    │
│  - Refactor         │ - Some complexity       │ - Implement     │
│  - Config changes   │ - Minor breaking        │ - Audit         │
│                     │                         │ - Fix           │
│                     │                         │ - Report        │
├──────────────────────────────────────────────────────────────────┤
│  🔴 HIGH RISK       │                         │                 │
│  - Security changes │ - Core systems          │ Enhanced 7-step │
│  - Breaking API     │ - Many files (10+)      │ - Understand    │
│  - Architecture     │ - High complexity       │ - Design        │
│  - Data migration   │ - Breaking changes      │ - Implement     │
│  - Performance      │ - Rollback needed       │ - Checkpoint 1  │
│                     │                         │ - Implement     │
│                     │                         │ - Checkpoint 2  │
│                     │                         │ - Oracle Audit  │
│                     │                         │ - Human Review  │
│                     │                         │ - Report        │
└──────────────────────────────────────────────────────────────────┘
```

**Decision Tree:**

```
                    ┌─────────────────┐
                    │  Đọc Task       │
                    └─────────────────┘
                           ↓
            ┌──────────────────────────┐
            │  Có code changes không?  │
            └──────────────────────────┘
                   ↓            ↓
                 KHÔNG         CÓ
                   ↓            ↓
            ┌──────────┐  ┌──────────────────┐
            │  LOW     │  │  Có breaking     │
            │  Risk    │  │  changes không?  │
            └──────────┘  └──────────────────┘
                               ↓            ↓
                             KHÔNG         CÓ
                               ↓            ↓
                        ┌──────────┐  ┌──────────┐
                        │  MEDIUM  │  │  HIGH    │
                        │  Risk    │  │  Risk    │
                        └──────────┘  └──────────┘
```

**Output:**
```markdown
## Task Classification

**Type:** [🟢 LOW / 🟡 MEDIUM / 🔴 HIGH]

**Reasoning:**
- [Lý do 1]
- [Lý do 2]

**Selected Workflow:** [Light 3-step / Standard 5-step / Enhanced 7-step]

**Time Estimate:** [X minutes/hours]
```

---

#### 1.2. XÁC ĐỊNH KỸ NĂNG CẦN THIẾT (Skills Mapping) ⭐ **MỚI**

**Hành động:**
- [ ] Dựa vào task type, chọn skills phù hợp
- [ ] Load skills cần thiết trước khi bắt đầu
- [ ] Note skills cần consult (nếu có)

**Skills Matrix by Task Type:**

```
┌─────────────────────────────────────────────────────────────────┐
│  TASK TYPE           │ REQUIRED SKILLS         │ OPTIONAL       │
├─────────────────────────────────────────────────────────────────┤
│  🟢 LOW RISK         │                         │                │
│  - Docs              │ - None (direct tools)   │ - explore      │
│  - Typo            │                         │                │
│  - Minor config    │                         │                │
├─────────────────────────────────────────────────────────────────┤
│  🟡 MEDIUM RISK      │                         │                │
│  - Rust code       │ - git-master            │ - oracle       │
│  - Bug fix         │ - (for commits)         │   (if complex) │
│  - Feature         │                         │                │
│  - TypeScript      │ - frontend-ui-ux        │ - playwright   │
│  - Web UI          │ - (for styling)         │   (for testing)│
│  - Tests           │ - None                  │ - explore      │
│                    │                         │   (for patterns)│
├─────────────────────────────────────────────────────────────────┤
│  🔴 HIGH RISK        │                         │                │
│  - Architecture    │ - oracle                │ - artistry     │
│  - Security        │ - (consultation)        │   (creative    │
│  - Performance     │                         │    solutions)  │
│  - Core refactoring│                         │                │
│  - Firmware        │ - (embedded expertise)  │ - explore      │
│  - Multi-lane      │                         │   (codebase)   │
└─────────────────────────────────────────────────────────────────┘
```

**Skill Selection Guide:**

| Khi Bạn Cần | Sử Dụng Skill | Cách Load |
|-------------|---------------|-----------|
| **Git operations** (commit, rebase, squash) | `git-master` | `task(load_skills=["git-master"])` |
| **Frontend/UI work** (React, CSS, styling) | `frontend-ui-ux` | `task(category="visual-engineering", load_skills=["frontend-ui-ux"])` |
| **Browser automation** (testing, scraping) | `playwright` | `task(load_skills=["playwright"])` |
| **Deep research** (unfamiliar patterns) | `explore` agent | `task(subagent_type="explore", run_in_background=true)` |
| **External references** (docs, libraries) | `librarian` agent | `task(subagent_type="librarian", run_in_background=true)` |
| **Complex architecture** | `oracle` agent | `task(subagent_type="oracle")` |
| **Creative solutions** | `artistry` agent | `task(subagent_type="artistry")` |

**Output:**
```markdown
## Skills Required

**Primary Skills:**
- [Skill 1] - [Lý do]
- [Skill 2] - [Lý do]

**Secondary Skills (backup):**
- [Skill 3] - [Khi cần]

**Agents to Consult:**
- [Agent type] - [Mục đích]

**Load Commands:**
```typescript
task(category="...", load_skills=["skill-1", "skill-2"])
task(subagent_type="explore", run_in_background=true)
```
```

---

#### 1.3. XÁC ĐỊNH FILES LIÊN QUAN (File Identification) ⭐ **ENHANCED**

**Hành động:**
- [ ] Search codebase với multiple strategies
- [ ] Đọc documentation của modules liên quan
- [ ] Xác định dependencies
- [ ] List test files cần update
- [ ] Create file dependency map

**Search Strategies by Task Type:**

### **A. Rust Code Changes**

```bash
# 1. Find module directory
find src -name "*.rs" -path "*/module_name/*"

# 2. Search for pattern
grep -r "function_name" --include="*.rs" src/

# 3. Find trait implementations
ast-grep --pattern 'impl Trait for Type { $$$ }' --lang rust

# 4. Find usages (LSP)
lsp_find_references(filePath, line, character)

# 5. Find test files
find tests -name "*.rs" | xargs grep -l "module_name"
```

### **B. TypeScript/React Changes**

```bash
# 1. Find components
find apps/web -name "*.tsx" | xargs grep -l "ComponentName"

# 2. Find imports
grep -r "import.*from.*component" --include="*.ts" --include="*.tsx"

# 3. Find styles
find apps/web -name "*.css" -o -name "*.module.css" | xargs grep -l "className"

# 4. Find tests
find apps/web -name "*.test.tsx" -o -name "*.spec.tsx"
```

### **C. Configuration Changes**

```bash
# 1. Find config files
find . -name "*.toml" -o -name "*.yaml" -o -name "*.yml" -o -name "*.json"

# 2. Search for config key
grep -r "config_key" --include="*.toml" --include="*.yaml" .

# 3. Find code that uses config
grep -r "config.config_key" --include="*.rs" src/
```

### **D. Documentation Changes**

```bash
# 1. Find existing docs
find docs -name "*.md" | xargs grep -l "topic"

# 2. Find related code
grep -r "function_name" --include="*.rs" --include="*.ts" src/ apps/

# 3. Find examples
find examples -name "*.rs" -o -name "*.ts" | xargs grep -l "topic"
```

**Output:**
```markdown
## Affected Files

**Core Files (must modify):**
- `path/to/file1.rs` - [Lý do + line numbers if known]
- `path/to/file2.ts` - [Lý do + line numbers if known]

**Test Files (must update):**
- `tests/test_file.rs` - [Coverage needed]

**Documentation (should update):**
- `docs/feature.md` - [API changes]

**Files to Review (no change):**
- `src/lib.rs` - Check exports
- `Cargo.toml` - Check dependencies

**Dependencies:**
- Module A → Module B → Module C
```

---

#### 1.4. PHÂN TÍCH YÊU CẦU CHI TIẾT (Requirements Analysis) ⭐ **ENHANCED**

**Hành động:**
- [ ] Trả lời 5W1H questions
- [ ] Identify implicit requirements
- [ ] List assumptions
- [ ] Note constraints

**5W1H Framework:**

```
WHAT: Task yêu cầu làm gì?
  - [Mô tả cụ thể, measurable]

WHY: Tại sao cần làm?
  - [Business/technical value]

WHO: Ai sẽ sử dụng kết quả?
  - [End users, developers, systems]

WHEN: Khi nào cần hoàn thành?
  - [Deadline, priority]

WHERE: Ở đâu sẽ áp dụng?
  - [Modules, systems, environments]

HOW: Làm như thế nào?
  - [Approach, constraints, preferences]
```

**Implicit Requirements Checklist:**

```
Security:
- [ ] Does this affect authentication/authorization?
- [ ] Are there sensitive data handling implications?
- [ ] Any new attack vectors introduced?

Performance:
- [ ] Will this impact response time?
- [ ] Memory usage implications?
- [ ] Scalability concerns?

Compatibility:
- [ ] Breaking changes to API?
- [ ] Backward compatibility needed?
- [ ] Migration path required?

Testing:
- [ ] What tests need to be added/updated?
- [ ] Integration test coverage?
- [ ] Performance benchmarks needed?

Documentation:
- [ ] API docs need update?
- [ ] User guides affected?
- [ ] Changelog entry needed?
```

**Output:**
```markdown
## Requirements Analysis

### 5W1H

**WHAT:** [Clear description]
**WHY:** [Business value]
**WHO:** [Stakeholders]
**WHEN:** [Timeline]
**WHERE:** [Scope]
**HOW:** [Approach]

### Implicit Requirements
- [Requirement 1]
- [Requirement 2]

### Assumptions
1. [Assumption 1] - [Risk level]
2. [Assumption 2] - [Risk level]

### Questions to Clarify
- [ ] [Question 1]
- [ ] [Question 2]
```

---

#### 1.5. ĐÁNH GIÁ RỦI RO (Risk Assessment) ⭐ **ENHANCED**

**Hành động:**
- [ ] Phân loại risk level
- [ ] Identify breaking changes
- [ ] Note side effects
- [ ] Create rollback plan
- [ ] Define success metrics

**Risk Assessment Matrix:**

```
┌─────────────────────────────────────────────────────────────────┐
│  RISK FACTOR          │ LOW        │ MEDIUM     │ HIGH        │
├─────────────────────────────────────────────────────────────────┤
│  Code Changes         │ 1-2 files  │ 3-5 files  │ 6+ files    │
│  Test Coverage        │ Existing   │ Need some  │ Need many   │
│  Breaking Changes     │ None       │ Minor      │ Major       │
│  Rollback Complexity│ Simple     │ Moderate   │ Complex     │
│  User Impact        │ None       │ Some       │ Significant │
│  Security Impact    │ None       │ Low        │ Critical    │
└─────────────────────────────────────────────────────────────────┘
```

**Risk Calculation:**

```
Risk Score = (Code Changes × 1) + (Test Coverage × 2) + 
             (Breaking Changes × 3) + (Rollback Complexity × 2) + 
             (User Impact × 2) + (Security Impact × 4)

Score 1-5:   🟢 LOW    → Light 3-step workflow
Score 6-12:  🟡 MEDIUM → Standard 5-step workflow
Score 13+:   🔴 HIGH   → Enhanced 7-step workflow
```

**Rollback Plan Template:**

```markdown
## Rollback Plan

### Pre-conditions
- [ ] Backups created
- [ ] Rollback command tested
- [ ] Stakeholders notified

### Rollback Steps
1. **Immediate:** [Stop service/disable feature]
2. **Short-term:** [Revert code/config]
3. **Long-term:** [Restore from backup if needed]

### Rollback Commands
```bash
# Git revert
git revert [commit-hash]

# Config restore
cp dev/backups/config.toml .
```

### Success Criteria
- [ ] Service restored
- [ ] Data integrity verified
- [ ] Users can access normally
```

**Output:**
```markdown
## Risk Assessment

**Risk Level:** [🟢 LOW / 🟡 MEDIUM / 🔴 HIGH]

**Risk Score:** [X] / [Max]

**Risk Factors:**
- Code Changes: [Low/Medium/High]
- Test Coverage: [Low/Medium/High]
- Breaking Changes: [Low/Medium/High]
- Rollback Complexity: [Simple/Moderate/Complex]
- User Impact: [None/Some/Significant]
- Security Impact: [None/Low/Critical]

**Potential Breaking Changes:**
- [ ] API changes
- [ ] Config changes
- [ ] Database schema
- [ ] File formats

**Rollback Plan:** [Link to detailed plan]

**Success Metrics:**
- [Metric 1]
- [Metric 2]
```

---

#### 1.6. OUTPUT TỔNG HỢP BƯỚC 1 ⭐ **MỚI**

**Complete Step 1 Template:**

```markdown
# 📋 BƯỚC 1: ĐỌC & HIỂU TASK

## Task Summary

**Title:** [Task name]
**Received:** [Date/Time]
**From:** [Requester]

---

## 1. Task Classification

**Type:** [🟢 LOW / 🟡 MEDIUM / 🔴 HIGH]

**Reasoning:**
1. [Reason 1]
2. [Reason 2]

**Selected Workflow:** [Light 3-step / Standard 5-step / Enhanced 7-step]

**Time Estimate:** [X hours/minutes]

---

## 2. Skills Required

**Primary Skills:**
- [Skill 1] - [Reason]
- [Skill 2] - [Reason]

**Agents to Consult:**
- [Agent type] - [Purpose]

**Load Commands:**
```typescript
task(category="...", load_skills=["skill-1"])
```

---

## 3. Requirements Analysis

### 5W1H
- **WHAT:** [Description]
- **WHY:** [Value]
- **WHO:** [Users]
- **WHEN:** [Timeline]
- **WHERE:** [Scope]
- **HOW:** [Approach]

### Implicit Requirements
- [Requirement 1]
- [Requirement 2]

### Assumptions
1. [Assumption 1] - [Risk]
2. [Assumption 2] - [Risk]

### Questions
- [ ] [Question 1]
- [ ] [Question 2]

---

## 4. Affected Files

**Core Files (modify):**
- `file1.rs` - [Reason]
- `file2.ts` - [Reason]

**Test Files (update):**
- `test1.rs` - [Coverage]

**Documentation (review):**
- `doc1.md` - [Update if API changes]

**Dependencies:**
```
Module A → Module B → Module C
```

---

## 5. Risk Assessment

**Risk Level:** [🟢 LOW / 🟡 MEDIUM / 🔴 HIGH]

**Risk Score:** [X] / [Max]

**Risk Factors:**
- Code: [L/M/H]
- Tests: [L/M/H]
- Breaking: [L/M/H]
- Rollback: [S/M/C]
- Users: [N/S/S]
- Security: [N/L/C]

**Rollback Plan:** [Brief summary or link]

**Success Metrics:**
- [Metric 1]
- [Metric 2]

---

## ✅ Step 1 Complete

**Ready to Proceed:** [Yes/No]

**If No, Blockers:**
- [Blocker 1]
- [Blocker 2]

**Next Step:** Proceed to Step 2 (Implementation)
```

---

### **BƯỚC 2: TRIỂN KHAI THỰC HIỆN** ⏱️ 40-50% thời gian

#### 2.1. Tạo Todo List Chi Tiết

**Hành động:**
- [ ] Break task thành atomic sub-tasks
- [ ] Estimate thời gian cho mỗi sub-task
- [ ] Xác định dependencies giữa sub-tasks
- [ ] Tạo todo list với `todowrite` tool

**Template:**
```markdown
## Implementation Plan

### Phase 1: Preparation
- [ ] Sub-task 1.1
- [ ] Sub-task 1.2

### Phase 2: Core Implementation
- [ ] Sub-task 2.1
- [ ] Sub-task 2.2

### Phase 3: Testing
- [ ] Sub-task 3.1
- [ ] Sub-task 3.2

### Phase 4: Documentation
- [ ] Sub-task 4.1
```

#### 2.2. Implement Từng Bước

**Best Practices:**
- ✅ Đánh dấu `in_progress` trước khi bắt đầu
- ✅ Commit nhỏ, thường xuyên (nếu dùng git)
- ✅ Chạy tests sau mỗi thay đổi lớn
- ✅ Đánh dấu `completed` ngay khi xong

**Code Quality Gates:**
```bash
# Rust
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test

# TypeScript
npm run lint
npm run type-check
npm run test

# Python
ruff check .
pytest tests/
```

#### 2.3. Verify Sau Mỗi Bước

**Checklist:**
- [ ] Code compile/build thành công
- [ ] Tests pass (existing + new)
- [ ] Linting clean
- [ ] Type checking pass
- [ ] Manual testing (nếu applicable)

**Output:**
```markdown
## Implementation Log

### Step 1: [Name]
- Status: ✅ Complete
- Files: `file1.rs`, `file2.ts`
- Tests: 5 passed
- Notes: [Ghi chú]

### Step 2: [Name]
- Status: ✅ Complete
- Files: `file3.rs`
- Tests: 3 passed
- Notes: [Ghi chú]
```

---

### **BƯỚC 3: AUDIT ĐỘC LẬP (Subagent)** ⏱️ 15-20% thời gian

#### 3.1. Chuẩn Bị Audit

**Hành động:**
- [ ] Thu thập tất cả files đã thay đổi
- [ ] Chuẩn bị danh sách changes cần verify
- [ ] Xác định criteria để pass audit

**Audit Criteria Template:**
```markdown
## Audit Criteria

**Functional:**
- [ ] Feature hoạt động đúng yêu cầu
- [ ] Edge cases được xử lý
- [ ] Error handling đầy đủ

**Code Quality:**
- [ ] Follow coding conventions
- [ ] No code smells
- [ ] Proper error handling

**Testing:**
- [ ] Unit tests đầy đủ
- [ ] Integration tests pass
- [ ] Coverage > X%

**Documentation:**
- [ ] Code comments
- [ ] API documentation
- [ ] User documentation
```

#### 3.2. Gọi Subagent Audit

**Prompt Template:**
```markdown
# Audit Request

## Context
[Tóm tắt task và mục tiêu]

## Changes Made
[List các files và changes chính]

## Audit Scope
- [ ] Code review
- [ ] Test verification
- [ ] Documentation review
- [ ] Security check
- [ ] Performance check

## Success Criteria
[Liệt kê criteria để pass audit]

## Required Output
1. List of issues found (nếu có)
2. Evidence for each issue
3. Severity rating (Critical/High/Medium/Low)
4. Recommendations for fixes
```

**Example:**
```markdown
# Audit Request: Rebranding ZeroClaw → RedClaw

## Context
Đã thực hiện rebranding toàn bộ codebase từ "zeroclaw" sang "redclaw"

## Changes Made
- 327 files modified
- 7,266 replacements
- Patterns: zeroclaw, ZeroClaw, ZEROCLAW

## Audit Scope
- [x] Verify no "zeroclaw" remaining in source code
- [x] Verify all "redclaw" replacements correct
- [x] Verify backups created
- [x] Verify documentation updated

## Success Criteria
1. 0 occurrences of "zeroclaw" in src/**/*.rs
2. All README files contain "redclaw"
3. Backup files exist in dev/backups/
4. Build passes: cargo build --release
5. Tests pass: cargo test

## Required Output
1. Grep results showing 0 "zeroclaw" in source
2. Grep results showing "redclaw" count
3. Backup directory listing
4. Build/test output
```

#### 3.3. Thu Thập Bằng Chứng Xác Minh

**Evidence Types:**
```
✓ Command output (grep, find, etc.)
✓ Test results (pass/fail counts)
✓ Build output (success/failure)
✓ File listings (before/after)
✓ Screenshots (UI changes)
✓ Logs (runtime behavior)
```

**Evidence Template:**
```markdown
## Verification Evidence

### Evidence 1: Source Code Check
**Command:**
```bash
find src -name "*.rs" -exec grep -l "zeroclaw" {} \;
```

**Output:**
```
(no output)
```

**Conclusion:** ✅ No "zeroclaw" in source files

### Evidence 2: Replacement Count
**Command:**
```bash
grep -c "redclaw" README.md
```

**Output:**
```
83
```

**Conclusion:** ✅ 83 occurrences of "redclaw"

### Evidence 3: Build Status
**Command:**
```bash
cargo build --release 2>&1 | tail -5
```

**Output:**
```
Finished release [optimized] target(s)
```

**Conclusion:** ✅ Build successful
```

#### 3.4. Báo Cáo Kết Quả Audit

**Report Template:**
```markdown
# Audit Report

## Summary
- **Task:** [Task name]
- **Auditor:** [Subagent name]
- **Date:** [Date]
- **Status:** PASS/FAIL

## Findings

### Critical Issues (0)
None

### High Issues (0)
None

### Medium Issues (0)
None

### Low Issues (0)
None

## Verification Results

| Criteria | Expected | Actual | Status |
|----------|----------|--------|--------|
| Criterion 1 | Value | Value | ✅/❌ |
| Criterion 2 | Value | Value | ✅/❌ |

## Conclusion
[Overall assessment and recommendation]
```

---

### **BƯỚC 4: KHẮC PHỤC** ⏱️ 15-20% thời gian

#### 4.1. Phân Loại Issues

**Priority Matrix:**
```
┌──────────────────────────────────────────────────────────┐
│  P0 - Critical   │ Fix immediately, block deployment    │
│  P1 - High       │ Fix before next step                 │
│  P2 - Medium     │ Fix before final delivery            │
│  P3 - Low        │ Fix when convenient, document        │
└──────────────────────────────────────────────────────────┘
```

#### 4.2. Fix Issues

**Process:**
1. [ ] Tạo todo cho từng issue
2. [ ] Fix theo priority (P0 → P3)
3. [ ] Verify fix với tests
4. [ ] Update implementation log

**Tracking Template:**
```markdown
## Issue Tracking

### Issue #1: [Description]
- **Severity:** [P0/P1/P2/P3]
- **Root Cause:** [Analysis]
- **Fix Applied:** [What was done]
- **Verification:** [How verified]
- **Status:** ✅ Fixed

### Issue #2: [Description]
- **Severity:** [P0/P1/P2/P3]
- **Root Cause:** [Analysis]
- **Fix Applied:** [What was done]
- **Verification:** [How verified]
- **Status:** ✅ Fixed
```

#### 4.3. Quay Lại Bước 3

**Khi nào:**
- ✅ Sau khi fix TẤT CẢ issues
- ✅ Đã verify locally các fixes
- ✅ Sẵn sàng cho re-audit

**Lưu ý:**
- ⚠️ Chỉ re-audit những gì đã fix
- ⚠️ Giữ track của audit history
- ⚠️ Document lessons learned

---

### **BƯỚC 5: TỔNG HỢP TÀI LIỆU** ⏱️ 10-15% thời gian

#### 5.1. Viết Báo Cáo Hoàn Thành

**Structure:**
```markdown
# Task Completion Report

## Executive Summary
[2-3 sentences overview]

## Task Details
- **Task ID:** [ID]
- **Assigned:** [Date]
- **Completed:** [Date]
- **Status:** ✅ Complete

## What Was Done
[Bullet list of major accomplishments]

## Files Changed
[List with brief descriptions]

## Metrics
[Quantitative results]

## Verification
[Summary of audit results]

## Known Issues
[Any remaining issues or technical debt]

## Next Steps
[Recommended follow-ups]
```

#### 5.2. Liệt Kê Bằng Chứng

**Evidence Index:**
```markdown
## Evidence Index

### Code Evidence
1. [Source code verification](#source-verification)
2. [Test results](#test-results)
3. [Build output](#build-output)

### Documentation Evidence
4. [Updated docs](#documentation)
5. [API changes](#api-changes)

### Runtime Evidence
6. [Screenshots](#screenshots)
7. [Logs](#logs)
```

#### 5.3. Lưu Trữ Artifacts

**Artifact Types:**
```
✓ Reports (Markdown, PDF)
✓ Logs (execution, audit)
✓ Screenshots
✓ Test results
✓ Build artifacts
✓ Backup files
```

**Storage Structure:**
```
task-{id}/
├── report.md
├── audit-report.md
├── logs/
│   ├── execution.log
│   └── audit.log
├── evidence/
│   ├── screenshots/
│   └── outputs/
└── backups/
    └── [backup files]
```

#### 5.4. Template Báo Cáo Cuối Cùng

```markdown
# 📊 BÁO CÁO HOÀN THÀNH NHIỆM VỤ

## 🎯 Task: [Task Name]

**Status:** ✅ COMPLETE  
**Date:** YYYY-MM-DD  
**Time Spent:** X hours

---

## 📋 TÓM TẮT

[Mô tả 2-3 câu về task và kết quả]

---

## ✅ KẾT QUẢ ĐẠT ĐƯỢC

### Deliverables
- [x] Deliverable 1
- [x] Deliverable 2
- [x] Deliverable 3

### Metrics
| Metric | Target | Actual | Status |
|--------|--------|--------|--------|
| Metric 1 | Value | Value | ✅ |
| Metric 2 | Value | Value | ✅ |

---

## 📁 FILES THAY ĐỔI

| File | Changes | Lines |
|------|---------|-------|
| `file1.rs` | [Description] | +X -Y |
| `file2.ts` | [Description] | +X -Y |

**Total:** X files, +Y additions, -Z deletions

---

## 🔍 XÁC MINH & AUDIT

### Audit Results
- **Auditor:** [Subagent]
- **Date:** [Date]
- **Status:** ✅ PASS
- **Issues Found:** 0

### Verification Evidence

#### 1. Source Code
```bash
$ [Command]
[Output]
```
✅ [Conclusion]

#### 2. Tests
```bash
$ cargo test
[Output: X passed]
```
✅ [Conclusion]

#### 3. Build
```bash
$ cargo build --release
[Output: Success]
```
✅ [Conclusion]

---

## 📊 QUY TRÌNH ĐÃ TUÂN THỦ

| Bước | Status | Notes |
|------|--------|-------|
| 1. Đọc & Hiểu | ✅ | [Notes] |
| 2. Triển Khai | ✅ | [Notes] |
| 3. Audit | ✅ | [Notes] |
| 4. Khắc Phục | ✅ | [Notes] |
| 5. Tổng Hợp | ✅ | [Notes] |

---

## ⚠️ LƯU Ý

### Known Issues
- [Issue 1 (nếu có)]
- [Issue 2 (nếu có)]

### Technical Debt
- [Debt 1 (nếu có)]
- [Debt 2 (nếu có)]

### Recommendations
- [Recommendation 1]
- [Recommendation 2]

---

## 📎 PHỤ LỤC

### A. Audit Report
[Link hoặc nội dung audit report]

### B. Test Results
[Link hoặc nội dung test results]

### C. Related Documentation
- [Doc 1](link)
- [Doc 2](link)

---

**Report Generated:** YYYY-MM-DD HH:MM:SS  
**Generated By:** [Agent Name]
```

---

## 🛠️ TOOLS & TEMPLATES

### Todo Management
```markdown
- [ ] Task 1
  - [ ] Sub-task 1.1
  - [ ] Sub-task 1.2
```

### Audit Checklist
```markdown
## Audit Checklist

### Code Quality
- [ ] Follows style guide
- [ ] No code smells
- [ ] Proper error handling

### Testing
- [ ] Unit tests pass
- [ ] Integration tests pass
- [ ] Coverage adequate

### Documentation
- [ ] Code comments
- [ ] API docs updated
- [ ] User docs updated
```

### Evidence Collection
```markdown
## Evidence

### Type: [Command Output/Screenshot/Log]
**Source:** [Command/File/Tool]
**Timestamp:** [Date/Time]
**Content:** [Output/Screenshot]
**Conclusion:** [What this proves]
```

---

## 📈 METRICS & KPIs

### Quality Metrics
- **Defect Rate:** < 5%
- **Test Coverage:** > 80%
- **Audit Pass Rate:** 100%
- **Rework Rate:** < 20%

### Efficiency Metrics
- **On-Time Delivery:** > 90%
- **First-Time Right:** > 80%
- **Documentation Completeness:** 100%

---

## 🔄 CONTINUOUS IMPROVEMENT

### Post-Task Review
Sau mỗi task, tự đánh giá:
1. What went well?
2. What could be improved?
3. What lessons learned?
4. What to do differently next time?

### Process Updates
Cập nhật SOP này khi:
- Tìm ra best practices mới
- Phát hiện gaps trong quy trình
- Nhận feedback từ stakeholders

---

## 📞 ESCALATION

### Khi Nào Cần Escalate
- ⚠️ Blockers > 2 hours
- ⚠️ Requirements unclear sau 2 lần hỏi
- ⚠️ Technical risks quá cao
- ⚠️ Timeline không khả thi

### Escalation Path
1. Ask for clarification
2. Request technical consultation
3. Propose alternative approach
4. Escalate to stakeholder

---

**Document Owner:** [Your Name]  
**Last Updated:** 2026-03-15  
**Next Review:** 2026-04-15  
**Version:** 1.0
