# opencodex

`opencodex`는 **Telegram ↔ Codex/OMX CLI**를 연결하는 독립형 서버입니다.

핵심 동작:
- `opencodex <project_dir>` 한 줄로 서버 시작
- `--omx` 플래그로 Codex 대신 OMX 바이너리 실행
- `--madmax` 플래그로 approvals/sandbox 완전 우회 실행
- 첫 DM 사용자 Owner 임프린트 (기본 접근 제어)
- 일반 텍스트 → Codex 스트리밍 응답
- `/help`, `/pwd`, `/stop`, `/down <file>`, `!<shell>` 지원
- 세션 저장(기본): `~/.opencodex/sessions/*.json`
- 내부 파일 전송: `opencodex --sendfile <path> --chat <id> --key <hash>`

호환성:
- 레거시 환경변수 `OPENCLAUDE_TELEGRAM_TOKEN`도 계속 지원
- 레거시 경로 `~/.openclaude/*`를 읽고 함께 갱신
- 레거시 바이너리 이름 `openclaude`도 alias로 제공

---

## 1) 사전 준비

1. Telegram Bot Token 발급 (@BotFather)
2. Codex CLI 설치 및 로그인 (`codex --version` 확인, 필요시 `omx --version` 확인)
3. Rust toolchain 설치 (`~/.cargo/bin/cargo --version` 확인)

---

## 2) 빌드

```bash
cd ~/workspace/opencodex
~/.cargo/bin/cargo check
~/.cargo/bin/cargo build --release
```

실행 파일:
```bash
./target/release/opencodex --help
```

원하면 전역 설치:
```bash
~/.cargo/bin/cargo install --path ~/workspace/opencodex --force
```

---

## 3) 실행 방법

토큰 우선순위:
1. `--token <TOKEN>`
2. `OPENCODEX_TELEGRAM_TOKEN`
3. `OPENCLAUDE_TELEGRAM_TOKEN` (legacy)
4. `TELEGRAM_BOT_TOKEN`
5. `~/.opencodex/config.json` 저장값

토큰이 CLI 인자 또는 환경변수로 들어오면 `~/.opencodex/config.json`에 자동 저장됩니다.
실행 전 Telegram `getMe` 검증을 수행하므로 잘못된 토큰이면 즉시 종료됩니다.

### 최초 실행 예시

```bash
opencodex ~/workspace/my-project --token "123456789:ABCDEF..."
```

### OMX로 실행

```bash
opencodex ~/workspace/my-project --omx
```

### madmax 모드로 실행

```bash
opencodex ~/workspace/my-project --madmax
```

### 이후 실행 (토큰 생략 가능)

```bash
opencodex ~/workspace/my-project
```

---

## 4) Telegram 명령

- `/help` : 도움말
- `/pwd` : 현재 작업 경로
- `/stop` : 현재 AI 요청 중단
- `/down <file>` : 파일 다운로드
- `!<command>` : 프로젝트 디렉터리에서 쉘 실행

일반 텍스트 메시지는 Codex로 전달되어 스트리밍 응답됩니다.

---

## 5) 저장 파일

- `~/.opencodex/config.json` : 기본 토큰
- `~/.opencodex/bot_settings.json` : owner, 토큰 해시 매핑, 마지막 세션 정보
- `~/.opencodex/sessions/*.json` : 대화 세션 히스토리
- (호환) `~/.openclaude/*` 경로도 읽고 갱신

---

## 6) OpenSpec

요청에 따라 OpenSpec 초기화 완료:
- `openspec/` 디렉터리 생성
- `.codex/` OpenSpec 명령/스킬 생성

필요 시:
```bash
cd ~/workspace/opencodex
openspec status
```

---

## 7) 참고 문서 복사

요청대로 아래 파일을 루트에 복사 완료:
- `AGENT.md`
- `WORKFLOW.md`
- `AGENTS.md`
