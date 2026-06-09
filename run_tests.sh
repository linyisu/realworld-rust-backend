#!/usr/bin/env bash
set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${YELLOW}=== RealWorld API 测试 ===${NC}"

if lsof -Pi :3000 -sTCP:LISTEN -t >/dev/null 2>&1; then
    echo -e "${GREEN}✓ 后端已在 3000 端口运行${NC}"
else
    echo -e "${RED}✗ 后端未运行！${NC}"
    echo -e "${YELLOW}请先启动后端:${NC}"
    echo "  cd ~/Project/realworld/realworld-backend"
    echo "  cargo run --release"
    exit 1
fi

API_SPEC_DIR=~/Project/realworld/realworld-spec/specs/api
cd "$API_SPEC_DIR"

echo -e "${YELLOW}开始运行 API 测试...${NC}"
echo ""

export HOST="http://localhost:3000"
export UID_VAL="$(date +%s)$$"

echo "测试配置:"
echo "  HOST: $HOST"
echo "  UID: $UID_VAL"
echo ""

hurl --test \
  --jobs 1 \
  --variable "host=$HOST" \
  --variable "uid=$UID_VAL" \
  hurl/auth.hurl \
  hurl/profiles.hurl \
  hurl/articles.hurl \
  hurl/favorites.hurl \
  hurl/feed.hurl \
  hurl/tags.hurl \
  hurl/pagination.hurl \
  hurl/errors_auth.hurl \
  hurl/errors_articles.hurl \
  hurl/errors_authorization.hurl \
  hurl/errors_profiles.hurl

echo ""
echo -e "${GREEN}=== 测试完成 ===${NC}"
