#!/bin/bash
# Script xác minh rebranding ZeroClaw → RedClaw

echo "═══════════════════════════════════════════"
echo "  XÁC MINH REBRANDING: ZEROCLAW → REDCLAW"
echo "═══════════════════════════════════════════"
echo ""

# Đếm files còn zeroclaw (trừ backups và reports)
echo "1. Kiểm tra source code Rust (.rs)..."
RS_FILES=$(find src -name "*.rs" -exec grep -l "zeroclaw" {} \; 2>/dev/null | wc -l)
if [ "$RS_FILES" -eq 0 ]; then
    echo "   ✅ Không còn file .rs nào chứa 'zeroclaw'"
else
    echo "   ❌ Còn $RS_FILES file .rs chứa 'zeroclaw'"
fi

# Kiểm tra README
echo ""
echo "2. Kiểm tra README.md..."
README_REDC LAW=$(grep -c "redclaw" README.md 2>/dev/null || echo 0)
if [ "$README_REDC LAW" -gt 50 ]; then
    echo "   ✅ README.md có $README_REDC LAW occurrences của 'redclaw'"
else
    echo "   ⚠️  README.md chỉ có $README_REDC LAW occurrences của 'redclaw'"
fi

# Kiểm tra .env.example
echo ""
echo "3. Kiểm tra .env.example..."
ENV_REDC LAW=$(grep -c "REDCLAW" .env.example 2>/dev/null || echo 0)
if [ "$ENV_REDC LAW" -gt 40 ]; then
    echo "   ✅ .env.example có $ENV_REDC LAW biến REDCLAW"
else
    echo "   ⚠️  .env.example chỉ có $ENV_REDC LAW biến REDCLAW"
fi

# Kiểm tra install.sh
echo ""
echo "4. Kiểm tra install.sh..."
INSTALL_REDC LAW=$(grep -c "REDCLAW" install.sh 2>/dev/null || echo 0)
if [ "$INSTALL_REDC LAW" -gt 20 ]; then
    echo "   ✅ install.sh có $INSTALL_REDC LAW references 'REDCLAW'"
else
    echo "   ⚠️  install.sh chỉ có $INSTALL_REDC LAW references 'REDCLAW'"
fi

# Kiểm tra backups
echo ""
echo "5. Kiểm tra backups..."
BACKUP_COUNT=$(ls -1 dev/backups/ 2>/dev/null | wc -l)
if [ "$BACKUP_COUNT" -gt 300 ]; then
    echo "   ✅ Có $BACKUP_COUNT files backups"
else
    echo "   ⚠️  Chỉ có $BACKUP_COUNT files backups"
fi

# Kiểm tra web UI
echo ""
echo "6. Kiểm tra Web UI..."
WEB_PACKAGE=$(grep "redclaw-web" apps/web/package.json 2>/dev/null | wc -l)
if [ "$WEB_PACKAGE" -gt 0 ]; then
    echo "   ✅ Web package name là 'redclaw-web'"
else
    echo "   ❌ Web package name chưa cập nhật"
fi

echo ""
echo "═══════════════════════════════════════════"
echo "  KẾT QUẢ TỔNG HỢP"
echo "═══════════════════════════════════════════"
echo ""
echo "Xem chi tiết tại:"
echo "  - dev/XAC_MINH_REBRANDING.md"
echo "  - dev/VERIFICATION_REPORT.md"
echo "  - dev/rebrand_report.md"
echo ""
echo "Backups tại:"
echo "  - dev/backups/"
echo ""
