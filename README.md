# openclaude

**Telegram에서 AI(Claude)와 대화하며 서버의 코드를 읽고, 수정하고, 실행할 수 있는 도구입니다.**

쉽게 말해, 텔레그램 채팅창이 곧 AI 코딩 터미널이 됩니다.

---

## 이런 걸 할 수 있어요

- 텔레그램에서 "이 파일 수정해줘"라고 말하면 AI가 직접 코드를 수정
- `!git status` 같은 쉘 명령을 텔레그램에서 바로 실행
- 서버에 있는 파일을 텔레그램으로 다운로드하거나, 텔레그램에서 서버로 업로드
- 그룹 채팅에서 팀원들과 함께 AI 사용 가능

---

## 설치하기 (처음부터 따라하기)

### 1단계: 필요한 것 준비

아래 3가지가 필요합니다. 하나씩 확인하세요.

**A. Telegram Bot 만들기**

1. 텔레그램에서 [@BotFather](https://t.me/BotFather) 검색 후 대화 시작
2. `/newbot` 입력
3. 봇 이름과 사용자명 입력 (예: `MyClaude`, `myclaude_bot`)
4. 발급된 토큰을 복사해두세요 (예: `123456789:ABCdefGHIjklMNOpqrsTUVwxyz`)

**B. Claude Code CLI 설치**

```bash
# Claude Code 설치 (Node.js 18+ 필요)
npm install -g @anthropic-ai/claude-code

# 설치 확인
claude --version
```

> Claude Code가 처음이라면 `claude` 명령 실행 후 로그인을 먼저 완료하세요.

**C. Rust 설치**

```bash
# Rust 설치 (1분 소요)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 터미널 재시작 후 확인
cargo --version
```

---

### 2단계: openclaude 빌드

```bash
# 소스 받기
git clone https://github.com/YOUR_USERNAME/openclaude.git
cd openclaude

# 빌드 (처음에 2-3분 소요)
cargo build --release
```

빌드가 끝나면 `./target/release/openclaude` 파일이 생성됩니다.

**어디서든 실행하고 싶다면** (선택):
```bash
cargo install --path . --force
```
이제 어디서든 `openclaude` 명령을 사용할 수 있습니다.

---

### 3단계: 실행하기

```bash
# 최초 실행 (토큰 입력)
openclaude ~/my-project --token "여기에_봇토큰_붙여넣기"
```

성공하면 이렇게 표시됩니다:
```
openclaude 0.1.0
project_dir: /home/user/my-project
status: connecting Telegram bot...
```

**이후에는 토큰 없이 실행** (자동 저장됨):
```bash
openclaude ~/my-project
```

> 토큰은 `~/.openclaude/config.json`에 안전하게 저장됩니다 (권한 600).

---

## 사용 방법 (텔레그램 명령어)

봇을 실행한 뒤, 텔레그램에서 봇에게 메시지를 보내면 됩니다.

### 기본 사용

| 입력 | 설명 |
|------|------|
| 일반 텍스트 | AI에게 질문하거나 작업 요청 |
| `!명령어` | 쉘 명령 실행 (예: `!ls -la`, `!git status`) |
| `/help` | 도움말 보기 |

### 세션 (작업 폴더) 관리

| 명령 | 설명 | 예시 |
|------|------|------|
| `/start 경로` | 작업 폴더 지정 후 시작 | `/start ~/my-project` |
| `/start` | 기본 폴더로 시작 | |
| `/cd 경로` | 작업 폴더 변경 (세션 유지) | `/cd src/` |
| `/pwd` | 현재 작업 경로 확인 | |
| `/clear` | AI 대화 기록 삭제 | |
| `/stop` | AI 응답 중단 | |

### 파일 전송

| 명령 | 설명 | 예시 |
|------|------|------|
| `/down 파일경로` | 서버 파일을 텔레그램으로 다운로드 | `/down README.md` |
| 파일/사진 전송 | 텔레그램에서 서버로 업로드 | 파일 첨부 후 전송 |

> 업로드 크기 제한: 50MB

### 도구(Tool) 관리

Claude Code가 사용하는 도구를 제어할 수 있습니다.

| 명령 | 설명 |
|------|------|
| `/availabletools` | 사용 가능한 도구 전체 목록 |
| `/allowedtools` | 현재 허용된 도구 목록 |
| `/allowed +이름` | 도구 추가 (예: `/allowed +Bash`) |
| `/allowed -이름` | 도구 제거 |

### 그룹 채팅

그룹에서 봇을 사용하려면:

1. 봇을 그룹에 초대
2. 소유자가 `/public on` 입력
3. 이제 그룹 멤버 모두 `;메시지` 형태로 AI 사용 가능

| 명령 | 설명 |
|------|------|
| `;메시지` | 그룹에서 AI에게 메시지 전송 |
| `/public on` | 모든 멤버 사용 허용 (소유자만 가능) |
| `/public off` | 소유자만 사용 (기본값) |

---

## 보안

- **소유자 자동 등록**: 봇에게 처음 메시지를 보낸 사람이 소유자로 등록됩니다
- **명령 권한 분류**: 위험한 명령 (`!쉘`, `/public` 등)은 소유자만 실행 가능
- **경로 제한**: `/start`, `/cd`, `/down`은 홈 디렉토리 안에서만 동작
- **업로드 제한**: 파일 업로드 50MB 제한
- **그룹 채팅**: `/public on` 이전까지 소유자만 사용 가능. 공개 후에도 비소유자는 읽기 전용 명령만 허용

---

## 실행 옵션

```bash
openclaude [프로젝트_경로] [옵션]
```

| 옵션 | 설명 |
|------|------|
| `--token "토큰"` | Telegram Bot 토큰 지정 |
| `--madmax` | Claude Code 권한 확인 우회 (주의: 모든 작업을 확인 없이 실행) |

### 토큰 우선순위

1. `--token` 옵션
2. `OPENCLAUDE_TELEGRAM_TOKEN` 환경변수
3. `TELEGRAM_BOT_TOKEN` 환경변수
4. `~/.openclaude/config.json` 저장값

---

## 저장 파일 위치

| 파일 | 용도 |
|------|------|
| `~/.openclaude/config.json` | 봇 토큰 (자동 저장) |
| `~/.openclaude/bot_settings.json` | 소유자 정보, 세션 매핑 |
| `~/.openclaude/sessions/*.json` | 대화 기록 |

---

## 문제 해결

**"Telegram token not found"**
→ 토큰을 입력하세요: `openclaude ~/my-project --token "토큰"`

**"Invalid project directory"**
→ 존재하는 폴더 경로를 입력하세요: `ls ~/my-project`로 확인

**"Access denied: outside the allowed path sandbox"**
→ 홈 디렉토리(`~`) 밖의 경로는 접근할 수 없습니다

**AI가 응답하지 않을 때**
→ `/stop`으로 중단 후 다시 시도. Claude Code CLI가 정상 작동하는지 확인: `claude --version`

**빌드 에러**
→ Rust 최신 버전 확인: `rustup update`

---

## 라이선스

MIT
