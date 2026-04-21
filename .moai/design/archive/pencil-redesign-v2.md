# Pencil 재설계 실행 계획 (moai-termail.pen)

Pencil MCP 연결 복구 후 (Claude Desktop 재시작 필요), 아래 `batch_design` 오퍼레이션을 순차 적용해 moai-termail.pen 을 재설계한다.

파일: `/Users/goos/MoAI/moai-adk-go/pencil/moai-termail.pen`

## 실행 원칙

- 1 배치 = 최대 25 ops
- 기존 frame 삭제는 **금지**. 수정 / 추가만.
- 각 배치는 독립적 — 실패 시 해당 배치 rollback 후 다음 배치 진행.
- 검증: 각 배치 후 `get_screenshot` 으로 시각 확인 + `snapshot_layout problemsOnly=true` 로 레이아웃 이슈 점검.

---

## Batch 1 — Frame 01 Empty State 추가 (**최우선**)

**목표**: Main Workspace 의 3-pane 중앙에 워크스페이스 0개 일 때 표시될 Empty State 프레임 삽입.

현재 Frame 01 (`6rM07`) 의 Body (`Jk4CN`) 아래에 `EmptyStateFrame` 신규.

```javascript
// batch_design operations (filePath: "/Users/goos/MoAI/moai-adk-go/pencil/moai-termail.pen")

// 1. Empty state container (3-pane 중앙의 오버레이 역할, 워크스페이스 0 시 표시)
emptyContainer=I("Jk4CN",{
  type:"frame",
  name:"EmptyState",
  width:"fill_container",
  height:"fill_container",
  fill:"$bg-base",
  layout:"vertical",
  alignItems:"center",
  justifyContent:"center",
  gap:32,
  padding:48
})

// 2. Icon (sprout)
emptyIcon=I(emptyContainer,{
  type:"icon_font",
  iconFontFamily:"lucide",
  iconFontName:"sprout",
  width:56,
  height:56,
  fill:"$accent-moai"
})

// 3. Title
emptyTitle=I(emptyContainer,{
  type:"text",
  content:"Welcome to MoAI Studio",
  fontFamily:"$font-sans",
  fontSize:28,
  fontWeight:"bold",
  fill:"$fg-primary"
})

// 4. Subtitle
emptySubtitle=I(emptyContainer,{
  type:"text",
  content:"SPEC-first native shell for Claude Code agents",
  fontFamily:"$font-sans",
  fontSize:15,
  fontWeight:"normal",
  fill:"$fg-muted"
})

// 5. Primary CTA
emptyPrimaryBtn=I(emptyContainer,{
  type:"frame",
  name:"primaryCTA",
  cornerRadius:10,
  fill:"$accent-moai",
  padding:[16,28],
  layout:"horizontal",
  alignItems:"center",
  gap:10
})
plusIcon=I(emptyPrimaryBtn,{type:"icon_font",iconFontFamily:"lucide",iconFontName:"plus",width:16,height:16,fill:"#FFFFFF"})
plusLabel=I(emptyPrimaryBtn,{type:"text",content:"Create First Workspace",fontFamily:"$font-sans",fontSize:14,fontWeight:"semibold",fill:"#FFFFFF"})

// 6. Secondary CTAs (2 side-by-side)
secondaryRow=I(emptyContainer,{type:"frame",name:"secondaryRow",layout:"horizontal",gap:12})

sampleBtn=I(secondaryRow,{type:"frame",name:"sampleBtn",cornerRadius:8,fill:"$bg-surface",stroke:{fill:"$border-subtle",thickness:1},padding:[12,20],layout:"horizontal",alignItems:"center",gap:10})
sampleIcon=I(sampleBtn,{type:"icon_font",iconFontFamily:"lucide",iconFontName:"rocket",width:14,height:14,fill:"$fg-secondary"})
sampleLabel=I(sampleBtn,{type:"text",content:"Start Sample",fontFamily:"$font-sans",fontSize:13,fontWeight:"medium",fill:"$fg-primary"})

recentBtn=I(secondaryRow,{type:"frame",name:"recentBtn",cornerRadius:8,fill:"$bg-surface",stroke:{fill:"$border-subtle",thickness:1},padding:[12,20],layout:"horizontal",alignItems:"center",gap:10})
recentIcon=I(recentBtn,{type:"icon_font",iconFontFamily:"lucide",iconFontName:"folder-open",width:14,height:14,fill:"$fg-secondary"})
recentLabel=I(recentBtn,{type:"text",content:"Open Recent",fontFamily:"$font-sans",fontSize:13,fontWeight:"medium",fill:"$fg-primary"})

// 7. Tip
emptyTip=I(emptyContainer,{
  type:"text",
  content:"Tip: ⌘K opens Command Palette anytime",
  fontFamily:"Geist Mono",
  fontSize:11,
  fontWeight:"normal",
  fill:"$fg-muted"
})
```

---

## Batch 2 — Frame 01 Toolbar 레이어 추가

**목표**: TitleBar (`rVuKc`) 와 Body (`Jk4CN`) 사이에 Toolbar 36pt 높이 레이어 삽입.

```javascript
// Toolbar — TitleBar 바로 아래, Body 위
// Parent: 6rM07 (Main Workspace frame). Body 앞에 insert.
toolbar=I("6rM07",{
  type:"frame",
  name:"Toolbar",
  width:"fill_container",
  height:36,
  fill:"$bg-surface",
  stroke:{fill:"$border-subtle",thickness:{bottom:1}},
  padding:[0,12],
  layout:"horizontal",
  alignItems:"center",
  gap:8
})

// Move the newly inserted Toolbar to be child index 1 (between TitleBar and Body)
M(toolbar,"6rM07",1)

// Toolbar buttons — 7 primary actions
tbNew=I(toolbar,{type:"frame",name:"tbNew",cornerRadius:6,fill:"$bg-surface-2",padding:[6,10],layout:"horizontal",alignItems:"center",gap:6})
tbNewIcon=I(tbNew,{type:"icon_font",iconFontFamily:"lucide",iconFontName:"plus",width:13,height:13,fill:"$fg-secondary"})
tbNewLabel=I(tbNew,{type:"text",content:"New",fontFamily:"$font-sans",fontSize:12,fontWeight:"medium",fill:"$fg-primary"})

tbSplitH=I(toolbar,{type:"frame",name:"tbSplitH",cornerRadius:6,fill:"$bg-surface-2",padding:[6,8]})
tbSplitHIcon=I(tbSplitH,{type:"icon_font",iconFontFamily:"lucide",iconFontName:"columns-2",width:13,height:13,fill:"$fg-secondary"})

tbSplitV=I(toolbar,{type:"frame",name:"tbSplitV",cornerRadius:6,fill:"$bg-surface-2",padding:[6,8]})
tbSplitVIcon=I(tbSplitV,{type:"icon_font",iconFontFamily:"lucide",iconFontName:"rows-2",width:13,height:13,fill:"$fg-secondary"})

tbDivider=I(toolbar,{type:"rectangle",width:1,height:18,fill:"$border-subtle"})

tbRun=I(toolbar,{type:"frame",name:"tbRun",cornerRadius:6,fill:"$accent-moai-dim",stroke:{fill:"$accent-moai",thickness:1},padding:[6,10],layout:"horizontal",alignItems:"center",gap:6})
tbRunIcon=I(tbRun,{type:"icon_font",iconFontFamily:"lucide",iconFontName:"play",width:13,height:13,fill:"$accent-moai"})
tbRunLabel=I(tbRun,{type:"text",content:"Run SPEC",fontFamily:"$font-sans",fontSize:12,fontWeight:"semibold",fill:"$accent-moai"})

tbPalette=I(toolbar,{type:"frame",name:"tbPalette",cornerRadius:6,fill:"$bg-surface-2",padding:[6,10],layout:"horizontal",alignItems:"center",gap:6})
tbPaletteIcon=I(tbPalette,{type:"icon_font",iconFontFamily:"lucide",iconFontName:"search",width:13,height:13,fill:"$fg-secondary"})
tbPaletteLabel=I(tbPalette,{type:"text",content:"⌘K",fontFamily:"Geist Mono",fontSize:11,fontWeight:"medium",fill:"$fg-muted"})

// Right spacer
tbSpacer=I(toolbar,{type:"frame",width:"fill_container"})

// Agent status pill
tbAgent=I(toolbar,{type:"frame",name:"tbAgent",cornerRadius:999,fill:"$bg-surface-2",padding:[4,10],layout:"horizontal",alignItems:"center",gap:6})
tbAgentDot=I(tbAgent,{type:"ellipse",width:6,height:6,fill:"$status-success"})
tbAgentLabel=I(tbAgent,{type:"text",content:"Agent: idle",fontFamily:"$font-sans",fontSize:11,fontWeight:"medium",fill:"$fg-secondary"})
```

---

## Batch 3 — Frame 05 Agent Run Viewer 강화

**목표**: `arH` 에 play/pause/stop 컨트롤 추가, detailCard 에 "Follow this agent" 토글 추가.

```javascript
// Add Run Controls to arH (6kYdu > arLeft > arH)
runControls=I("FwE2g",{type:"frame",name:"runControls",layout:"horizontal",gap:6})
rcPlay=I(runControls,{type:"frame",name:"rcPlay",cornerRadius:6,fill:"$bg-surface-2",padding:6})
rcPlayIcon=I(rcPlay,{type:"icon_font",iconFontFamily:"lucide",iconFontName:"play",width:13,height:13,fill:"$status-success"})
rcPause=I(runControls,{type:"frame",name:"rcPause",cornerRadius:6,fill:"$bg-surface-2",padding:6})
rcPauseIcon=I(rcPause,{type:"icon_font",iconFontFamily:"lucide",iconFontName:"pause",width:13,height:13,fill:"$fg-secondary"})
rcStop=I(runControls,{type:"frame",name:"rcStop",cornerRadius:6,fill:"$bg-surface-2",padding:6})
rcStopIcon=I(rcStop,{type:"icon_font",iconFontFamily:"lucide",iconFontName:"square",width:13,height:13,fill:"$status-danger"})

// Token Breakdown card in arRight (opO4c) after chartCard
tokenCard=I("opO4c",{type:"frame",name:"tokenCard",cornerRadius:12,fill:"$bg-surface-2",stroke:{fill:"$border-subtle",thickness:1},padding:16,layout:"vertical",gap:10,width:"fill_container"})
tokenLabel=I(tokenCard,{type:"text",content:"TOKEN BREAKDOWN",fontFamily:"Inter",fontSize:9,fontWeight:"bold",fill:"$fg-muted",letterSpacing:1})
tokenBars=I(tokenCard,{type:"frame",name:"tokenBars",layout:"horizontal",height:20,gap:2})
tkInput=I(tokenBars,{type:"rectangle",width:180,height:20,fill:"$status-info",cornerRadius:2})
tkOutput=I(tokenBars,{type:"rectangle",width:120,height:20,fill:"$accent-moai",cornerRadius:2})
tkCached=I(tokenBars,{type:"rectangle",width:60,height:20,fill:"$status-success",cornerRadius:2})
tkReasoning=I(tokenBars,{type:"rectangle",width:40,height:20,fill:"$status-warning",cornerRadius:2})

tokenLegend=I(tokenCard,{type:"frame",name:"tokenLegend",layout:"horizontal",gap:12})
legInput=I(tokenLegend,{type:"text",content:"Input 1,800",fontFamily:"Geist Mono",fontSize:10,fill:"$fg-secondary"})
legOutput=I(tokenLegend,{type:"text",content:"Out 1,200",fontFamily:"Geist Mono",fontSize:10,fill:"$fg-secondary"})
legCached=I(tokenLegend,{type:"text",content:"Cached 600",fontFamily:"Geist Mono",fontSize:10,fill:"$fg-secondary"})
legReason=I(tokenLegend,{type:"text",content:"Reason 400",fontFamily:"Geist Mono",fontSize:10,fill:"$fg-secondary"})

// Follow Agent toggle in detailCard (GTMBF)
followToggle=I("GTMBF",{type:"frame",name:"followToggle",cornerRadius:6,fill:"$bg-base",stroke:{fill:"$border-subtle",thickness:1},padding:[8,12],layout:"horizontal",alignItems:"center",gap:10})
ftIcon=I(followToggle,{type:"icon_font",iconFontFamily:"lucide",iconFontName:"crosshair",width:13,height:13,fill:"$accent-moai"})
ftLabel=I(followToggle,{type:"text",content:"Follow this agent",fontFamily:"$font-sans",fontSize:12,fontWeight:"medium",fill:"$fg-primary"})
ftSwitch=I(followToggle,{type:"frame",name:"ftSwitch",cornerRadius:999,fill:"$accent-moai",width:28,height:16})
ftKnob=I(ftSwitch,{type:"ellipse",width:12,height:12,fill:"#FFFFFF",x:14,y:2})
```

---

## Batch 4 — Frame 08 Command Palette 섹션 그룹화

**목표**: `results_body` (jLUNw) 를 섹션 기반으로 재구성.

기존 자식 비우고 5 섹션 재생성:

```javascript
// Clear results_body children first (read IDs and delete)
// This requires prior read; skip delete and just add new children at the bottom
// Better: insert new grouped sections into jLUNw

sec1=I("jLUNw",{type:"frame",name:"sec_favorites",layout:"vertical",gap:0,padding:[4,0]})
sec1Label=I(sec1,{type:"text",content:"⭐ FAVORITES",fontFamily:"Inter",fontSize:9,fontWeight:"bold",fill:"$accent-moai",letterSpacing:1,padding:[6,12]})
sec1Row1=I(sec1,{type:"frame",name:"fav1",cornerRadius:6,fill:"$bg-surface-3",padding:[10,12],layout:"horizontal",alignItems:"center",gap:12})
sec1Icon=I(sec1Row1,{type:"icon_font",iconFontFamily:"lucide",iconFontName:"play",width:14,height:14,fill:"$accent-moai"})
sec1Text=I(sec1Row1,{type:"text",content:"Run SPEC-M2-UX-001",fontFamily:"$font-sans",fontSize:13,fontWeight:"medium",fill:"$fg-primary"})

sec2=I("jLUNw",{type:"frame",name:"sec_commands",layout:"vertical",gap:0,padding:[4,0]})
sec2Label=I(sec2,{type:"text",content:"⚡ COMMANDS",fontFamily:"Inter",fontSize:9,fontWeight:"bold",fill:"$fg-muted",letterSpacing:1,padding:[6,12]})
sec2Row1=I(sec2,{type:"frame",name:"cmd1",cornerRadius:6,padding:[10,12],layout:"horizontal",alignItems:"center",gap:12})
sec2Icon=I(sec2Row1,{type:"icon_font",iconFontFamily:"lucide",iconFontName:"columns-2",width:13,height:13,fill:"$fg-secondary"})
sec2Text=I(sec2Row1,{type:"text",content:"Split Pane Horizontally",fontFamily:"$font-sans",fontSize:13,fill:"$fg-primary"})
sec2Shortcut=I(sec2Row1,{type:"text",content:"⌘\\",fontFamily:"Geist Mono",fontSize:11,fill:"$fg-muted"})
```

---

## Batch 5 — Frame 13 Mission Control 신규 생성

**목표**: 전체 새 frame. 위치: 기존 frames 옆 빈 공간.

먼저 `find_empty_space_on_canvas` 로 위치 찾기:

```
find_empty_space_on_canvas(filePath, width:1600, height:1000, padding:100, direction:"right")
```

반환된 x/y 사용 (예: x=5200, y=0):

```javascript
mc=I("document",{
  type:"frame",
  name:"13. Mission Control",
  width:1600,
  height:1000,
  x:5200,
  y:0,
  cornerRadius:12,
  fill:"$bg-base",
  layout:"vertical",
  clip:true
})

// ... (full frame layout, similar structure to other frames)
// Left 160pt filter sidebar, center grid, top bar, bottom summary
```

(작업량 커서 별도 batch — Batch 5a, 5b, 5c 로 분할)

---

## Batch 6-7 — Frame 14/15/16/17/18 신규 프레임

각 신규 프레임은 2-3 배치 필요.

- Batch 6: Frame 14 Agent Thread
- Batch 7: Frame 15 Context Panel + Frame 16 Diff Review
- Batch 8: Frame 17 Memory Viewer + Frame 18 Hooks & MCP Panel

---

## 검증 체크리스트

각 배치 완료 후:

```
// 시각 검증
get_screenshot(filePath, nodeId: <해당 프레임 id>)

// 레이아웃 문제 스캔
snapshot_layout(filePath, parentId: <해당 프레임 id>, problemsOnly: true, maxDepth: 4)
```

기대:
- `problemsOnly: true` → empty results
- 스크린샷에서 clip/overflow/misalignment 없음
- 디자인 토큰 (`$bg-base` 등) 정상 resolve

---

## 실행 전제 조건

1. **Claude Desktop 재시작 필수** — 현재 MCP 프로세스가 "antigravity" 캐시 상태로 Pencil.app 연결 불가
2. **Pencil.app 실행 중** — Pencil 앱 자체는 반드시 기동 상태
3. **moai-termail.pen 활성 문서** — get_editor_state 로 확인

재연결 확인 명령:
```
mcp__pencil__get_editor_state(include_schema: false)
→ result.content[0].text 에 "moai-termail.pen" 언급되어야 함
```

---

## 완료 기준

- 12 frames → **18 frames** (기존 + 신규 6)
- 모든 frame 에 Tier 1-4 기능 대응
- Design tokens 일관 적용
- screenshot + layout scan 통과
- `.moai/design/screenshots/` 디렉토리에 각 frame PNG 저장 (문서화용)

---

버전: 1.0.0 · 2026-04-17
