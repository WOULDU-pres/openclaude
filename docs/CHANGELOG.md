# 변경 이력 (Changelog)

---

## 2026-02-23 — Phase 2: 보안 통합 + 코드 품질

### 보안 강화 (auth.rs 통합)

**명령 권한 검사 추가** (`src/telegram/message.rs`)
- 모든 텔레그램 명령에 위험도 분류 (Safe / Elevated / Dangerous) 적용
- 비소유자가 위험한 명령 (`!쉘`, `/public`, `/allowed` 등) 실행 시 "Permission denied" 차단
- `/stop`, `/help`는 항상 허용 (Safe 등급)

**경로 샌드박스 추가** (`src/telegram/commands.rs`, `src/telegram/file_ops.rs`)
- `/start`, `/cd`, `/down` 명령에서 홈 디렉토리 밖 접근 차단
- 예: `/cd /etc`나 `/start /root` 시도 시 "Access denied" 메시지 반환

**파일 업로드 보안** (`src/telegram/file_ops.rs`, `src/telegram/message.rs`)
- 업로드 크기 제한: 50MB 초과 시 차단
- 공개 그룹에서 비소유자 파일 업로드 차단

### /cd 경로 영속성 수정 (`src/telegram/commands.rs`)
- `/cd`로 폴더 변경 후 봇을 재시작해도 마지막 경로가 자동 복원됨
- 기존에는 `/start`만 경로를 저장하고, `/cd`는 저장하지 않아 재시작 시 초기화되던 문제 수정

### clippy 경고 전체 정리
변경 전: 27개 경고 → 변경 후: 0개 경고

주요 수정:
- `unwrap()` 남용 → 안전한 에러 처리 패턴으로 교체
- `unsafe` 블록에 안전성 주석 추가
- `strip_prefix` 사용으로 수동 문자열 슬라이싱 제거
- `Default` 트레잇 자동 derive 적용
- 미사용 import 제거

### 커밋 이력
```
74c4078 fix: add permission gate for file uploads and sandbox check for /start
9f39be1 feat: integrate auth checks into telegram handlers (Phase 2)
fc5427f fix: resolve clippy warnings across codebase
```

---

## 2026-02-23 — Phase 1: 구조 개선 + 보안 기반

### codex.rs → claude.rs 리네임
- 파일명: `src/codex.rs` → `src/claude.rs`
- 함수명: `is_codex_available` → `is_claude_available` 등
- 모든 import/참조 업데이트

### auth.rs 보안 모듈 신규 생성 (`src/auth.rs`)
- `PermissionLevel` enum: Owner / Public / Restricted
- `CommandRisk` enum: Safe / Elevated / Dangerous
- `classify_command()`: 명령어별 위험도 자동 분류
- `can_execute()`: 사용자 권한 + 위험도 기반 실행 허용 판단
- `is_path_within_sandbox()`: 경로가 허용된 범위 안에 있는지 검사
- `DEFAULT_UPLOAD_LIMIT`: 업로드 크기 제한 (50MB)
- 7개 테스트 포함

### telegram.rs 모듈 분리
단일 파일 (2,642줄) → 8개 모듈로 분리:

| 파일 | 역할 | 줄 수 |
|------|------|------|
| `mod.rs` | 모듈 선언, 진입점 | 70 |
| `bot.rs` | 공유 상태 구조체 | 51 |
| `storage.rs` | 설정/세션 파일 입출력 | 290 |
| `streaming.rs` | 스트리밍 응답, HTML 변환 | 490 |
| `tools.rs` | 도구 관리 핸들러 | 272 |
| `file_ops.rs` | 파일 전송 핸들러 | 285 |
| `commands.rs` | 명령 핸들러 | 528 |
| `message.rs` | 메시지 라우터, AI 호출 | 776 |

### 입력 필터링 강화 (`src/session.rs`)
- `sanitize_user_input()` 대소문자 무시 필터링으로 변경
- "IGNORE PREVIOUS", "iGnOrE pReViOuS" 등 모든 변형 차단
- 12개 테스트 추가 (총 41개 테스트)

### CI 파이프라인 추가
- `.github/workflows/ci.yml`: PR/Push 시 자동 검사
  - `cargo fmt --check` (코드 포맷)
  - `cargo clippy -D warnings` (코드 품질)
  - `cargo test` (테스트)
  - `cargo audit` (보안 취약점, non-blocking)
- `.rustfmt.toml`: 코드 포맷 설정

### 커밋 이력
```
c65b087 refactor: security hardening, module split, CI pipeline, and test improvements
508a73d docs: sync README commands and analyze cargo audit warnings
```

---

## 프로젝트 구조 (현재)

```
src/
├── app.rs           — 설정 디렉토리명
├── auth.rs          — 권한 모델, 경로 샌드박스
├── claude.rs        — Claude Code CLI 브릿지
├── main.rs          — 진입점, 토큰 관리
├── session.rs       — 세션 데이터, 입력 필터링
└── telegram/
    ├── mod.rs       — 모듈 선언, 진입점
    ├── bot.rs       — 공유 상태 구조체
    ├── storage.rs   — 설정/세션 파일 입출력
    ├── streaming.rs — 스트리밍 응답, HTML 변환
    ├── tools.rs     — 도구 관리 핸들러
    ├── file_ops.rs  — 파일 전송 핸들러
    ├── commands.rs  — 명령 핸들러
    └── message.rs   — 메시지 라우터, AI 호출
```
