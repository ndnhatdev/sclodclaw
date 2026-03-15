# Tham khảo lệnh RedClaw

Dựa trên CLI hiện tại (`redclaw --help`).

Xác minh lần cuối: **2026-02-20**.

## Lệnh cấp cao nhất

| Lệnh | Mục đích |
|---|---|
| `onboard` | Khởi tạo workspace/config nhanh hoặc tương tác |
| `agent` | Chạy chat tương tác hoặc chế độ gửi tin nhắn đơn |
| `gateway` | Khởi động gateway webhook và HTTP WhatsApp |
| `daemon` | Khởi động runtime có giám sát (gateway + channels + heartbeat/scheduler tùy chọn) |
| `service` | Quản lý vòng đời dịch vụ cấp hệ điều hành |
| `doctor` | Chạy chẩn đoán và kiểm tra trạng thái |
| `status` | Hiển thị cấu hình và tóm tắt hệ thống |
| `cron` | Quản lý tác vụ định kỳ |
| `models` | Làm mới danh mục model của provider |
| `providers` | Liệt kê ID provider, bí danh và provider đang dùng |
| `channel` | Quản lý kênh và kiểm tra sức khỏe kênh |
| `integrations` | Kiểm tra chi tiết tích hợp |
| `skills` | Liệt kê/cài đặt/gỡ bỏ skills |
| `migrate` | Nhập dữ liệu từ runtime khác (hiện hỗ trợ OpenClaw) |
| `config` | Xuất schema cấu hình dạng máy đọc được |
| `completions` | Tạo script tự hoàn thành cho shell ra stdout |
| `hardware` | Phát hiện và kiểm tra phần cứng USB |
| `peripheral` | Cấu hình và nạp firmware thiết bị ngoại vi |

## Nhóm lệnh

### `onboard`

- `redclaw onboard`
- `redclaw onboard --interactive`
- `redclaw onboard --channels-only`
- `redclaw onboard --api-key <KEY> --provider <ID> --memory <sqlite|lucid|markdown|none>`
- `redclaw onboard --api-key <KEY> --provider <ID> --model <MODEL_ID> --memory <sqlite|lucid|markdown|none>`

### `agent`

- `redclaw agent`
- `redclaw agent -m "Hello"`
- `redclaw agent --provider <ID> --model <MODEL> --temperature <0.0-2.0>`
- `redclaw agent --peripheral <board:path>`

### `gateway` / `daemon`

- `redclaw gateway [--host <HOST>] [--port <PORT>]`
- `redclaw daemon [--host <HOST>] [--port <PORT>]`

### `service`

- `redclaw service install`
- `redclaw service start`
- `redclaw service stop`
- `redclaw service restart`
- `redclaw service status`
- `redclaw service uninstall`

### `cron`

- `redclaw cron list`
- `redclaw cron add <expr> [--tz <IANA_TZ>] <command>`
- `redclaw cron add-at <rfc3339_timestamp> <command>`
- `redclaw cron add-every <every_ms> <command>`
- `redclaw cron once <delay> <command>`
- `redclaw cron remove <id>`
- `redclaw cron pause <id>`
- `redclaw cron resume <id>`

### `models`

- `redclaw models refresh`
- `redclaw models refresh --provider <ID>`
- `redclaw models refresh --force`

`models refresh` hiện hỗ trợ làm mới danh mục trực tiếp cho các provider: `openrouter`, `openai`, `anthropic`, `groq`, `mistral`, `deepseek`, `xai`, `together-ai`, `gemini`, `ollama`, `astrai`, `venice`, `fireworks`, `cohere`, `moonshot`, `glm`, `zai`, `qwen` và `nvidia`.

### `channel`

- `redclaw channel list`
- `redclaw channel start`
- `redclaw channel doctor`
- `redclaw channel bind-telegram <IDENTITY>`
- `redclaw channel add <type> <json>`
- `redclaw channel remove <name>`

Lệnh trong chat khi runtime đang chạy (Telegram/Discord):

- `/models`
- `/models <provider>`
- `/model`
- `/model <model-id>`

Channel runtime cũng theo dõi `config.toml` và tự động áp dụng thay đổi cho:
- `default_provider`
- `default_model`
- `default_temperature`
- `api_key` / `api_url` (cho provider mặc định)
- `reliability.*` cài đặt retry của provider

`add/remove` hiện chuyển hướng về thiết lập có hướng dẫn / cấu hình thủ công (chưa hỗ trợ đầy đủ mutator khai báo).

### `integrations`

- `redclaw integrations info <name>`

### `skills`

- `redclaw skills list`
- `redclaw skills install <source>`
- `redclaw skills remove <name>`

`<source>` chấp nhận git remote (`https://...`, `http://...`, `ssh://...` và `git@host:owner/repo.git`) hoặc đường dẫn cục bộ.

Skill manifest (`SKILL.toml`) hỗ trợ `prompts` và `[[tools]]`; cả hai được đưa vào system prompt của agent khi chạy, giúp model có thể tuân theo hướng dẫn skill mà không cần đọc thủ công.

### `migrate`

- `redclaw migrate openclaw [--source <path>] [--dry-run]`

### `config`

- `redclaw config schema`

`config schema` xuất JSON Schema (draft 2020-12) cho toàn bộ hợp đồng `config.toml` ra stdout.

### `completions`

- `redclaw completions bash`
- `redclaw completions fish`
- `redclaw completions zsh`
- `redclaw completions powershell`
- `redclaw completions elvish`

`completions` chỉ xuất ra stdout để script có thể được source trực tiếp mà không bị lẫn log/cảnh báo.

### `hardware`

- `redclaw hardware discover`
- `redclaw hardware introspect <path>`
- `redclaw hardware info [--chip <chip_name>]`

### `peripheral`

- `redclaw peripheral list`
- `redclaw peripheral add <board> <path>`
- `redclaw peripheral flash [--port <serial_port>]`
- `redclaw peripheral setup-uno-q [--host <ip_or_host>]`
- `redclaw peripheral flash-nucleo`

## Kiểm tra nhanh

Để xác minh nhanh tài liệu với binary hiện tại:

```bash
redclaw --help
redclaw <command> --help
```
