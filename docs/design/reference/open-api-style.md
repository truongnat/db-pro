# Common UI Design System Specification

## Style Direction: Warm Minimalism + Editorial UI + Developer Tool Aesthetic

Mục tiêu của design system này là xây dựng một hệ thống UI có cảm giác hiện đại, cao cấp, nhẹ, rõ ràng và tập trung vào nội dung, lấy cảm hứng từ cách ChatGPT/Codex tổ chức giao diện.

Không clone trực tiếp bất kỳ sản phẩm nào. Hãy học theo triết lý thiết kế:

**Content first → Typography second → Spacing third → Decoration last.**

UI cần tạo cảm giác:

- Minimal
- Clean
- Premium
- Calm
- Intelligent
- Developer-friendly
- Editorial
- Neutral
- Dense vừa phải nhưng không bí
- Ít decoration
- Ít card
- Ít màu
- Rất chú trọng typography, spacing và interaction state

---

# 1. Core Design Principles

## 1.1 Content First

Mỗi màn hình phải ưu tiên nội dung hơn khung trang trí.

Không nên dùng quá nhiều:

- Card lồng card
- Border mạnh
- Shadow lớn
- Gradient
- Background nhiều màu
- Accent color quá dày
- Divider quá nhiều

Hierarchy nên đến từ:

1. Typography
2. Spacing
3. Alignment
4. Background contrast nhẹ
5. State
6. Border
7. Shadow

Shadow và màu sắc chỉ được xem là lớp cuối cùng.

---

## 1.2 Neutral-first Design

Khoảng 90–95% UI sử dụng neutral colors.

Accent chỉ dùng cho:

- Primary action
- Selected state
- Active state
- Links quan trọng
- Status đặc biệt
- Highlight
- Focus

Không phủ accent color lên toàn bộ app.

---

## 1.3 Quiet Interface

UI không được “ồn”.

Tránh:

- Border quanh tất cả section
- Tất cả button đều có background
- Tất cả heading đều bold
- Icon nhiều màu
- Card bo quá lớn
- Gradient ở mọi nơi
- Notification badge tràn lan

Một màn hình tốt phải vẫn đẹp ngay cả khi chỉ sử dụng:

- White
- Black
- Gray
- Text
- Space

---

# 2. Design Tokens

## 2.1 Color Tokens

### Light Theme

```css
:root {
  --background: #ffffff;
  --background-subtle: #fafafa;

  --surface: #f7f7f7;
  --surface-2: #f3f3f3;
  --surface-hover: #eeeeee;
  --surface-active: #e8e8e8;

  --text-primary: #0d0d0d;
  --text-secondary: #5f5f5f;
  --text-tertiary: #8a8a8a;
  --text-disabled: #b3b3b3;

  --border-subtle: #eeeeee;
  --border-default: #e2e2e2;
  --border-strong: #d2d2d2;

  --overlay: rgba(0, 0, 0, 0.42);

  --accent: #111111;
  --accent-foreground: #ffffff;

  --success: #16a34a;
  --warning: #d97706;
  --danger: #dc2626;
  --info: #2563eb;
}

```

### Dark Theme

```css
.dark {
  --background: #212121;
  --background-subtle: #1c1c1c;

  --surface: #2a2a2a;
  --surface-2: #303030;
  --surface-hover: #363636;
  --surface-active: #3d3d3d;

  --text-primary: #ececec;
  --text-secondary: #b9b9b9;
  --text-tertiary: #8d8d8d;
  --text-disabled: #666666;

  --border-subtle: #323232;
  --border-default: #414141;
  --border-strong: #505050;

  --overlay: rgba(0, 0, 0, 0.6);

  --accent: #f3f3f3;
  --accent-foreground: #111111;

  --success: #22c55e;
  --warning: #f59e0b;
  --danger: #ef4444;
  --info: #3b82f6;
}

```

Lưu ý:

- Không dùng pure `#000000` tràn lan.
- Không dùng pure white trên dark mode nếu không cần thiết.
- Text phụ luôn phải có độ tương phản thấp hơn text chính.
- Border luôn nhẹ hơn cảm giác “box”.

---

# 3. Typography System

## 3.1 Font Family

Ưu tiên:

```css
--font-sans:
  Inter,
  Geist,
  "SF Pro Text",
  "SF Pro Display",
  system-ui,
  sans-serif;

--font-mono:
  "Geist Mono",
  "JetBrains Mono",
  "SFMono-Regular",
  monospace;

```

Sans dùng cho UI.

Mono dùng cho:

- Code
- File path
- Command
- Log
- Terminal output
- Key/value technical data
- Diff
- Commit hash

---

## 3.2 Typography Scale

```text
Display
32px / 40px / 600

Page Title
24px / 32px / 600

Section Title
18px / 26px / 600

Subheading
16px / 24px / 500–600

Body
15px / 24px / 400

Body Small
14px / 20px / 400

UI Label
13px / 18px / 500

Caption
12px / 16px / 400

Mono UI
13px / 20px / 400

```

---

## 3.3 Font Weight Rules

Mặc định:

- 400: normal content
- 500: label, control, medium emphasis
- 600: heading
- 700: rất hạn chế

Không dùng bold cho mọi thứ.

Một trang có quá nhiều `font-bold` được coi là sai style.

---

# 4. Spacing System

Sử dụng base grid 4px.

```text
4
8
12
16
20
24
32
40
48
64
80
96

```

Quy tắc:

```text
Icon ↔ Text:
8px

Label ↔ Helper text:
4px

Form fields:
12–16px

Control groups:
16px

Small section gap:
24px

Main section gap:
32–40px

Major layout section:
48–64px

Page top/bottom spacing:
32–64px

```

Không tự nghĩ ra các khoảng kiểu:

- 13px
- 19px
- 27px
- 35px

Trừ trường hợp đặc biệt.

---

# 5. Radius System

```text
radius-xs: 4px
radius-sm: 6px
radius-md: 8px
radius-lg: 12px
radius-xl: 16px
radius-2xl: 24px
radius-full: 999px

```

Component mapping:

```text
Icon button:
6–8px

Normal button:
8px

Input:
8–10px

Dropdown:
10–12px

Popover:
12px

Card:
12–16px

Modal:
16px

Composer / chat input:
20–24px

Avatar:
999px

```

Không dùng `rounded-2xl` cho mọi component.

Radius phải thể hiện hierarchy.

---

# 6. Border Rules

Border mặc định:

```css
border: 1px solid var(--border-default);

```

Nhưng ưu tiên không border nếu spacing hoặc surface đã đủ phân tách.

Dùng border khi:

- Input
- Dialog
- Dropdown
- Table
- Toolbar cần phân vùng
- Selected item
- Code block
- Drag/drop area

Không dùng border bao quanh mọi block.

---

# 7. Shadow System

Shadow rất hạn chế.

Không shadow:

- Normal card
- Sidebar row
- Standard button
- Input
- Content section

Có thể shadow:

- Dialog
- Dropdown
- Popover
- Floating composer
- Command palette

Ví dụ:

```css
box-shadow:
  0 1px 2px rgba(0,0,0,0.04),
  0 8px 24px rgba(0,0,0,0.08);

```

Không dùng shadow kiểu:

- `shadow-xl`
- `shadow-2xl`
- glow

trừ khi có lý do rõ ràng.

---

# 8. Icons

Ưu tiên:

- Lucide
- Icon outline
- Monochrome

Base:

```text
Small: 14px
Default: 16px
Toolbar: 18px
Large: 20px

```

Stroke:

```text
1.6–1.9

```

Không trộn nhiều icon library.

Không dùng emoji thay icon UI chính.

---

# 9. Button System

## Primary Button

```text
Height:
36–40px

Padding:
0 14px

Font:
14px / 500

Radius:
8px

```

Primary có contrast mạnh nhưng không flashy.

---

## Secondary Button

Background mặc định:  
transparent hoặc surface nhẹ.

Hover:  
surface-hover

Border:  
chỉ dùng nếu cần.

---

## Ghost Button

Chỉ icon hoặc icon + text.

Normal:  
transparent

Hover:  
surface-hover

---

## Icon Button

```text
32x32 hoặc 36x36

```

Không tạo button quá lớn nếu chỉ chứa icon.

---

# 10. Inputs

Input cần:

- Clean
- Neutral
- Border nhẹ
- Focus rõ nhưng không neon

```css
height: 40px;
padding: 0 12px;

font-size: 14px;

border-radius: 8px;
border: 1px solid var(--border-default);

background: transparent;

```

Focus:

```css
border-color: var(--border-strong);

box-shadow:
  0 0 0 1px var(--border-strong);

```

Không dùng glow xanh đậm trừ khi product cần.

---

# 11. Form Layout

Form tốt:

```text
Label
Helper / description

Input

12–16px

Label
Input

```

Label:  
13–14px / 500

Helper text:  
12–13px / tertiary

Error:  
12–13px / danger

Không dồn label và field quá sát nhau.

---

# 12. Sidebar

Sidebar là navigation surface, không phải một panel nặng.

Đặc điểm:

- Background hơi khác main area
- Row height nhỏ
- Hover nhẹ
- Selected nhẹ
- Icon 16–18px
- Không border quanh mỗi item

Example:

```text
Workspace

+ New

Search

Chats
  Authentication fix
  UI architecture
  Prompt builder

Projects
  Project Alpha
  Design system

```

Selected item:

- surface-active
- text-primary
- không cần accent xanh

---

# 13. Layout System

Desktop structure:

```text
App Shell
├── Sidebar
├── Main
│   ├── Header
│   └── Content
└── Optional inspector / detail panel

```

Main content max width thường nên giới hạn:

```text
680px
760px
880px
1080px

```

Tuỳ loại màn hình.

Không kéo text content full màn hình 1440px.

Editor/tool UI có thể full width.

---

# 14. Card Philosophy

Card chỉ tồn tại khi thực sự cần group content.

Một card tốt:

```text
Title
Description

Content

```

Không nên:

```text
Card
  Card
    Card

```

Nếu section có thể phân tách bằng spacing, ưu tiên spacing.

---

# 15. Table

Table cần:

- Compact
- Clean
- Border nhẹ
- Horizontal line nhẹ
- Không mỗi cell một box

Header:  
12–13px / 500 / secondary

Row:  
14px

Row height:  
40–44px

Hover:  
surface-hover

---

# 16. Dropdown / Popover

Dropdown:

- radius 10–12px
- shadow nhẹ
- padding 4–6px
- item height 32–36px

Item:

```text
icon
8px
text

shortcut nằm bên phải

```

Selected không cần background accent mạnh.

---

# 17. Modal / Dialog

Dialog:

- max-width hợp lý
- radius 16px
- spacing nhiều hơn dropdown
- không quá nhiều border

Structure:

```text
Title
Description

Content

Actions

```

Actions nên nằm:

- bottom right
- primary cuối cùng

---

# 18. Empty States

Empty state không cần illustration lớn.

Ưu tiên:

```text
Icon nhỏ

No projects yet

Create your first project to get started.

[Create project]

```

Không biến empty state thành landing page.

---

# 19. Loading States

Loading nên nhẹ:

- Skeleton
- Spinner nhỏ
- Inline progress

Không block toàn app nếu chỉ một section đang load.

---

# 20. Toast

Toast:

- nhỏ
- tối giản
- xuất hiện ngắn
- không chiếm quá nhiều attention

Ví dụ:

```text
✓ Changes saved

```

hoặc:

```text
Failed to save changes
Retry

```

---

# 21. Developer-oriented Components

Nếu app có tính chất developer tool, cần common component sau:

```text
CodeBlock
InlineCode
Terminal
DiffViewer
FileTree
FileRow
CommitRow
StatusBadge
LogViewer
CommandInput
KeyboardShortcut
InspectorPanel
ResizablePanel
Tabs
Toolbar
SplitPane
CommandPalette

```

---

# 22. Code Block

Code block:

```text
Header:
language           copy

Code

```

Background:  
surface nhẹ.

Mono:  
13–14px

Line-height:  
20–22px

Không syntax-highlight quá sặc sỡ.

---

# 23. File Tree

Tree item:

```text
Chevron
Icon
Filename

```

Height:  
28–32px

Nested indentation:  
12–16px

Selected:  
surface-active

Không border từng file.

---

# 24. Diff Viewer

Added:  
green tint rất nhẹ

Removed:  
red tint rất nhẹ

Không dùng green/red bão hoà mạnh.

Example:

```text
+ const user = await getUser()
- const user = getUser()

```

---

# 25. Status Badge

Badge chỉ dùng khi information density yêu cầu.

Ví dụ:

- Running
- Passed
- Failed
- Draft
- Active
- Archived

Badge:

- small
- subtle
- muted color
- radius-full

Không biến badge thành nút.

---

# 26. Interaction States

Mọi interactive component phải có:

```text
default
hover
active
focus
disabled
loading

```

Selected state nếu cần.

Hover duration:

```text
120–160ms

```

Modal:

```text
160–220ms

```

Không animation 400–600ms cho UI cơ bản.

---

# 27. Motion

Motion philosophy:

**Fast, subtle, informative.**

Allowed:

- Fade
- Slight scale
- Small translate
- Height expansion

Avoid:

- Bounce
- Huge spring
- Large slide
- Fancy transition

---

# 28. Responsive Rules

Desktop:

- Sidebar cố định hoặc resizable
- Content centered hoặc max-width

Tablet:

- Sidebar collapsible
- Toolbar compact

Mobile:

- Navigation thành drawer
- Modal thành bottom sheet nếu hợp lý
- Touch target tối thiểu 40px

Không chỉ scale desktop xuống mobile.

---

# 29. Accessibility

Bắt buộc:

- keyboard navigation
- visible focus
- semantic HTML
- aria-label
- đủ contrast
- không chỉ dùng màu để thể hiện trạng thái
- tooltip cho icon-only button

Touch target:  
minimum 40x40 trên mobile.

---

# 30. Component Architecture

Mọi UI reusable phải đi vào common layer.

Ví dụ structure:

```text
src/
  components/
    common/
      button/
      input/
      textarea/
      select/
      checkbox/
      radio/
      switch/
      dialog/
      popover/
      dropdown-menu/
      tooltip/
      tabs/
      badge/
      avatar/
      card/
      table/
      empty-state/
      skeleton/
      spinner/
      code-block/
      file-tree/
      command-palette/

```

Không duplicate component trong từng feature nếu behavior giống nhau.

---

# 31. Common Component Rules

Một common component phải:

- Không chứa logic business-specific
- Nhận `className`
- Hỗ trợ composition
- Có type rõ ràng
- Hỗ trợ disabled
- Có state standard
- Có dark mode
- Accessible
- Reusable

Ví dụ:

```tsx
<Button variant="primary">
  Save
</Button>

<Button variant="ghost" size="icon">
  <Settings />
</Button>

```

Không tạo:

```tsx
<UserProfileSaveButton />

```

trong common layer.

---

# 32. Recommended Variants

## Button

```text
primary
secondary
ghost
outline
danger
link

```

Sizes:

```text
sm
md
lg
icon

```

---

## Input

```text
default
error
disabled

```

---

## Badge

```text
neutral
success
warning
danger
info

```

---

# 33. CSS / Tailwind Philosophy

Không spam utility ngẫu nhiên ở feature layer.

Nên abstract token.

Ví dụ không nên:

```tsx
<div className="rounded-[13px] border-[#dedede] bg-[#f7f7f7] px-[17px]">

```

Nên:

```tsx
<Card>

```

hoặc:

```tsx
<div className="rounded-lg border-border bg-surface px-4">

```

---

# 34. Recommended Semantic Tokens

```text
bg-background
bg-surface
bg-surface-hover
bg-surface-active

text-primary
text-secondary
text-tertiary

border-default
border-subtle

text-success
text-warning
text-danger

```

Không trực tiếp sử dụng gray-400, gray-500 ở mọi nơi nếu project lớn.

---

# 35. Common Page Shell

Tạo các reusable layout:

```text
AppShell
Page
PageHeader
PageContent
Section
SectionHeader
Sidebar
Toolbar
DetailPanel

```

Example:

```tsx
<Page>
  <PageHeader
    title="Projects"
    description="Manage your projects"
    actions={<Button>New project</Button>}
  />

  <PageContent>
    ...
  </PageContent>
</Page>

```

---

# 36. Page Header

PageHeader cần support:

```text
title
description
breadcrumb
actions
tabs

```

Không để feature tự build page header mỗi nơi một kiểu.

---

# 37. Section

Structure:

```tsx
<Section>
  <SectionHeader
    title=""
    description=""
    action={}
  />

  ...
</Section>

```

Spacing giữa section mặc định 32–40px.

---

# 38. Density

Mặc định density:

**comfortable compact**

Không quá thưa kiểu landing page.

Không quá chật kiểu enterprise legacy app.

Developer tool có thể compact hơn.

---

# 39. Visual Hierarchy

Ưu tiên hierarchy bằng:

```text
font-size
font-weight
spacing
alignment
color tone

```

Sau đó mới:

```text
background
border
shadow

```

---

# 40. Anti-patterns

AI tuyệt đối tránh:

```text
❌ Gradient background mặc định
❌ Glassmorphism
❌ Shadow mọi card
❌ Border mọi section
❌ rounded-2xl mọi component
❌ font-bold mọi title
❌ Icon nhiều màu
❌ Accent color tràn UI
❌ nested cards
❌ spacing ngẫu nhiên
❌ 20 loại gray khác nhau
❌ component duplicate
❌ inline style không cần thiết
❌ hardcoded hex trong feature component
❌ animation dài

```

---

# 41. Common Components Minimum Set

AI nên implement ít nhất:

```text
Button
IconButton
Input
Textarea
SearchInput
Select
Checkbox
RadioGroup
Switch
Label
FormField
Badge
Avatar
Tooltip
Popover
DropdownMenu
Dialog
Sheet
Tabs
Separator
Skeleton
Spinner
Toast
EmptyState
Card
Table
Pagination
Breadcrumb
PageHeader
SectionHeader
SidebarItem
Toolbar
CommandPalette
CodeBlock
InlineCode

```

---

# 42. Advanced Common Components

Nếu project dạng app lớn / developer tool:

```text
ResizablePanel
SplitPane
Inspector
FileTree
TreeView
DataTable
VirtualList
CommandMenu
SearchOverlay
FilterBar
StatusIndicator
Timeline
ActivityItem
DiffViewer
LogViewer
TerminalBlock
KeyValueList
JSONViewer
MarkdownViewer
EditorToolbar

```

---

# 43. Component Naming Rules

Tên phải generic:

Good:

```text
Button
Dialog
Section
PageHeader
StatusBadge
FileTree

```

Bad:

```text
UserProjectButton
OrderBlueDialog
ProfileGrayCard

```

---

# 44. UX Copy

Copy:

- ngắn
- rõ
- tự nhiên
- tránh enterprise wording

Good:

```text
Save changes
Delete project
Try again
No results found
Create project

```

Avoid:

```text
Proceed with operation
Execute deletion process
No matching records are currently available

```

---

# 45. AI Implementation Behaviour

Khi AI tạo màn hình mới:

1. Kiểm tra common component hiện có.
2. Reuse component trước.
3. Không tạo duplicate.
4. Nếu pattern xuất hiện &gt;= 2 lần, cân nhắc đưa vào common.
5. Không hardcode style riêng nếu token đã có.
6. Không tạo màu/radius/spacing mới nếu không cần.
7. Luôn support light/dark.
8. Luôn giữ spacing theo grid 4px.
9. Luôn giữ typography đúng scale.
10. Luôn ưu tiên content hơn decoration.

---

# 46. AI Refactoring Behaviour

Nếu gặp:

```tsx
<button className="...">

```

ở nhiều nơi, hãy chuyển sang:

```tsx
<Button>

```

Nếu page lặp:

```text
title
description
actions

```

hãy chuyển sang:

```tsx
<PageHeader />

```

Nếu lặp:

```text
label
input
error
description

```

hãy chuyển sang:

```tsx
<FormField />

```

---

# 47. Quality Checklist

Trước khi hoàn thành UI, AI phải tự kiểm tra:

### Typography

- Có quá nhiều bold không?
- Font-size có đúng scale không?
- Text hierarchy có rõ không?

### Spacing

- Có theo grid 4px không?
- Các section có đủ khoảng thở không?

### Color

- Có quá nhiều accent không?
- Neutral có chiếm đa số không?

### Component

- Có duplicate component không?
- Có thể đưa vào common không?

### Layout

- Text content có quá rộng không?
- Alignment có nhất quán không?

### Interaction

- Có hover?
- focus?
- disabled?
- loading?

### Theme

- Light mode OK?
- Dark mode OK?

### Accessibility

- keyboard?
- aria?
- contrast?

---

# 48. Final Visual Goal

UI cuối cùng phải mang cảm giác:

```text
calm
clean
quiet
technical
premium
modern
focused
predictable
consistent

```

Không được mang cảm giác:

```text
flashy
template-like
over-designed
bootstrap-like
enterprise legacy
color-heavy
card-heavy

```

---

# 49. Golden Rule

Khi phân vân giữa:

```text
thêm decoration

```

và

```text
cải thiện typography / spacing

```

luôn ưu tiên typography và spacing.

Khi phân vân giữa:

```text
tạo component mới

```

và

```text
reuse common component

```

luôn ưu tiên reuse.

Khi phân vân giữa:

```text
thêm một card

```

và

```text
dùng whitespace

```

ưu tiên whitespace.

---

# 50. Design Philosophy Summary

Design system này không cố tạo UI đẹp bằng decoration.

Nó tạo UI đẹp thông qua:

```text
Consistent typography
+
Predictable spacing
+
Neutral colors
+
Reusable components
+
Subtle interaction
+
Strong content hierarchy

```

Đây là nguyên tắc cốt lõi AI phải luôn giữ trong mọi màn hình và mọi component.