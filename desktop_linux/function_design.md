# Desktop Linux Core API 清单与数据流设计

## 目标

本文档整理当前 desktop 端在重写时所依赖的 `core` 能力边界，明确：

1. desktop 需要调用哪些 core API
2. 每个 API 的职责是什么
3. UI 与 core 之间建议如何组织数据流
4. 哪些能力必须进入 desktop 重写第一阶段

当前 desktop 与 core 的调用关系为：

`Desktop UI -> ServiceWrapper -> synap_core::service::SynapService`

也就是说，desktop 当前并未直接散落调用 `core`，而是通过 `desktop/src/core/mod.rs` 集中桥接。

## Core API 清单

面向 desktop 重写，当前真正需要的 `core` 能力可以先收敛成一份最小接口。按职责分组如下。

## 1. 生命周期

- `init(db_path?)`
- 当前实现对应：`SynapService::new(Some(path))`

用途：

- 打开或创建本地数据库
- 初始化 schema
- 初始化搜索索引

现有桥接位置：`desktop/src/core/mod.rs:16-29`

建议桌面重写后仍保留一层 adapter，不让 UI 直接依赖 `SynapService`。

## 2. 列表与检索

- `recent_notes(cursor?, limit?) -> Vec<NoteDTO>`
- `deleted_notes(cursor?, limit?) -> Vec<NoteDTO>`
- `search(query, limit) -> Vec<NoteDTO>`

用途：

- 主列表：最近可见笔记
- 删除列表：已删除笔记
- 搜索结果：按 query 返回笔记

对应 core：

- `get_recent_note`: `core/src/service.rs:524`
- `get_deleted_notes`: `core/src/service.rs:653`
- `search`: `core/src/service.rs:690`

当前行为特点：

- `recent_notes` 只返回“最新版本”的可见笔记
- `deleted_notes` 单独返回删除项
- `search` 当前接的是 note 搜索，不是独立 tag 搜索

## 3. 单条详情与关系

- `get_note(id) -> NoteDTO`
- `replies(parent_id, cursor?, limit) -> Vec<NoteDTO>`
- `origins(note_id) -> Vec<NoteDTO>`
- `other_versions(note_id) -> Vec<NoteDTO>`

用途：

- 详情面板
- 回复列表
- 父链 / 溯源链
- 版本关系

对应 core：

- `get_note`: `core/src/service.rs:432`
- `get_replies`: `core/src/service.rs:449`
- `get_origins`: `core/src/service.rs:586`
- `get_other_versions`: `core/src/service.rs:636`

当前行为特点：

- `get_note` 对已删除笔记按 not found 处理
- `replies` 是子节点列表
- `origins` 是父链
- `other_versions` 是当前版本之外的版本节点

## 4. 写操作

- `create_note(content, tags) -> NoteDTO`
- `reply_note(parent_id, content, tags) -> NoteDTO`
- `edit_note(note_id, content, tags) -> NoteDTO`
- `delete_note(note_id) -> ()`
- `restore_note(note_id) -> ()`

用途：

- 创建根笔记
- 创建回复
- 生成新版本
- 删除
- 恢复

对应 core：

- `create_note`: `core/src/service.rs:1030`
- `reply_note`: `core/src/service.rs:1042`
- `edit_note`: `core/src/service.rs:1067`
- `delete_note`: `core/src/service.rs:1089`
- `restore_note`: `core/src/service.rs:1101`

这里最重要的产品语义：

- `edit_note` 不是覆盖原笔记，而是生成新版本

## DTO 清单

当前 desktop 只消费一种核心 DTO：

```rust
pub struct NoteDTO {
    pub id: String,
    pub content: String,
    pub tags: Vec<String>,
    pub created_at: u64,
    pub deleted: bool,
}
```

位置：`core/src/dto.rs:6-13`

桌面端重写时，UI 层至少可以围绕这个 DTO 建两个视图模型：

- `NoteListItemViewModel`
- `NoteDetailViewModel`

但底层仍建议直接保留 `NoteDTO` 作为 adapter 输出，避免过早引入新的协议层。

## 建议的 Desktop Core Adapter

如果要重写 desktop，建议把现在的 `ServiceWrapper` 明确升级成一个桌面专用的 core adapter 接口，而不是让 UI 直接知道 `SynapService`。

可以抽象成：

```rust
pub trait DesktopCore {
    fn init(&self) -> Result<()>;

    fn recent_notes(&self, cursor: Option<&str>, limit: Option<usize>) -> Result<Vec<NoteDTO>>;
    fn deleted_notes(&self, cursor: Option<&str>, limit: Option<usize>) -> Result<Vec<NoteDTO>>;
    fn search(&self, query: &str, limit: usize) -> Result<Vec<NoteDTO>>;

    fn get_note(&self, id: &str) -> Result<NoteDTO>;
    fn replies(&self, parent_id: &str, cursor: Option<String>, limit: usize) -> Result<Vec<NoteDTO>>;
    fn origins(&self, note_id: &str) -> Result<Vec<NoteDTO>>;
    fn other_versions(&self, note_id: &str) -> Result<Vec<NoteDTO>>;

    fn create_note(&self, content: String, tags: Vec<String>) -> Result<NoteDTO>;
    fn reply_note(&self, parent_id: &str, content: String, tags: Vec<String>) -> Result<NoteDTO>;
    fn edit_note(&self, note_id: &str, content: String, tags: Vec<String>) -> Result<NoteDTO>;
    fn delete_note(&self, note_id: &str) -> Result<()>;
    fn restore_note(&self, note_id: &str) -> Result<()>;
}
```

好处：

- 新 desktop UI 不直接绑死 `OnceCell<Mutex<SynapService>>`
- 后面可替换成本地 mock、异步 facade、远端 bridge
- 方便先重写 UI，再替换底层实现

## 数据流图

下面按当前 desktop 的真实数据流整理成可迁移的图。

## 1. 启动流

```text
App Start
  -> DesktopCore.init()
  -> load_home()
      -> recent_notes(None, 50)
      -> deleted_notes(None, 50)
      -> if notes not empty:
           get_note(first.id)
           replies(first.id, None, 20)
           origins(first.id)
           other_versions(first.id)
  -> render Home
```

含义：

- 启动后先加载主列表和删除列表
- 默认自动选中第一条可见笔记
- 详情区依赖二次加载，不是从列表直接拼出来

## 2. 主列表 / 搜索流

```text
SearchQueryChanged
  -> update local query state only

RunSearch
  -> if query empty:
       recent_notes(None, 50)
     else:
       search(query, 50)
  -> deleted_notes(None, 50)
  -> if current selection still valid:
       reload selection detail
     else:
       clear selection or auto-select first note
  -> render
```

这里可以拆成两个层级：

- 本地 UI 状态：
  - `search_query`
- 远端 / core 状态：
  - `notes`
  - `deleted_notes`
  - `selected_note + relations`

## 3. 选中详情流

```text
SelectNote(id)
  -> get_note(id)
  -> replies(id, None, 20)
  -> origins(id)
  -> other_versions(id)
  -> update detail panel
  -> render
```

这个流说明当前详情是聚合读模型，不是单接口返回的完整 detail payload。

如果重写 desktop，建议把这个聚合显式命名成：

- `load_note_detail(id)`

内部再组合四个 core 调用。

## 4. 新建笔记流

```text
CreateNote(content, tags)
  -> create_note(content, tags)
  -> refresh lists
      -> recent_notes / deleted_notes
  -> select created note
      -> get_note(id)
      -> replies(id)
      -> origins(id)
      -> other_versions(id)
  -> render
```

重点：

- 写完以后不是局部补丁更新
- 当前实现是“写操作后整页 refresh，再重新选中”

这对重写很重要，因为可以决定是否继续用这个简单策略，还是改成乐观更新 / 局部更新。

## 5. 回复流

```text
ReplyToSelected(parent_id, content, tags)
  -> reply_note(parent_id, content, tags)
  -> refresh lists
  -> select new reply
  -> reload detail aggregates
  -> render
```

特点：

- 回复创建后会把焦点切到新回复

## 6. 编辑生成新版本流

```text
SaveNewVersion(note_id, content, tags)
  -> edit_note(note_id, content, tags)
  -> refresh lists
  -> select new version
  -> reload detail aggregates
  -> render
```

最关键的语义：

- 目标不是更新当前 note
- 而是返回一个新的 version note
- UI 选择应跳到新版本

这意味着桌面重写时，详情页交互文案和状态机都要围绕“版本演化”设计，而不是传统笔记应用的“保存覆盖”。

## 7. 删除流

```text
DeleteSelected(note_id)
  -> delete_note(note_id)
  -> clear selection
  -> refresh lists
  -> optionally auto-select first live note
  -> render
```

现状：

- 删除后从主列表移除
- 进入删除列表
- 当前选中会清掉

## 8. 恢复流

```text
RestoreNote(note_id)
  -> restore_note(note_id)
  -> refresh lists
  -> select restored note
  -> reload detail aggregates
  -> render
```

## 推荐的数据边界

桌面端重写时，建议把数据分成 3 层。

## 1. Core Layer

只负责调用 `synap-core`。

输入 / 输出：

- 输入：纯参数
- 输出：`NoteDTO` / `Vec<NoteDTO>` / `Result`

不负责：

- UI 状态
- 文案
- 选中逻辑
- 表单状态

## 2. Domain / ViewModel Layer

负责把多个 core 调用组合成 desktop 可直接消费的模型。

建议至少有两个聚合动作：

```text
load_home(query)
  -> recent_notes or search
  -> deleted_notes

load_note_detail(id)
  -> get_note
  -> replies
  -> origins
  -> other_versions
```

可以定义：

```rust
struct HomeData {
    notes: Vec<NoteDTO>,
    deleted_notes: Vec<NoteDTO>,
}

struct NoteDetailData {
    note: NoteDTO,
    replies: Vec<NoteDTO>,
    origins: Vec<NoteDTO>,
    other_versions: Vec<NoteDTO>,
}
```

## 3. UI State Layer

只保存交互状态。

例如：

- `search_query`
- `compose_content`
- `compose_tags`
- `selected_note_id`
- `detail_content`
- `detail_tags`
- `status`

## 重写时建议保留的核心动作

建议把桌面端最终抽成这些 use case，UI 只调 use case，不直接碰零散 core API。

```text
initialize_app()
load_home(query)
select_note(id)
create_note(content, tags)
reply_to_note(parent_id, content, tags)
create_new_version(note_id, content, tags)
delete_note(note_id)
restore_note(note_id)
```

它们与 core 的映射关系：

- `initialize_app`
  - `init`
  - `load_home("")`

- `load_home(query)`
  - `recent_notes` or `search`
  - `deleted_notes`

- `select_note(id)`
  - `get_note`
  - `replies`
  - `origins`
  - `other_versions`

- `create_note`
  - `create_note`
  - `load_home`
  - `select_note(new_id)`

- `reply_to_note`
  - `reply_note`
  - `load_home`
  - `select_note(new_id)`

- `create_new_version`
  - `edit_note`
  - `load_home`
  - `select_note(new_id)`

- `delete_note`
  - `delete_note`
  - `load_home`

- `restore_note`
  - `restore_note`
  - `load_home`
  - `select_note(restored_id)`

## 一张更完整的数据流图

```text
                    +-------------------+
                    |   Desktop UI      |
                    | search / list /   |
                    | detail / editor   |
                    +---------+---------+
                              |
                              v
                    +-------------------+
                    |  App State / VM   |
                    | query             |
                    | selected_note_id  |
                    | form fields       |
                    | status            |
                    +---------+---------+
                              |
                    +---------+---------+
                    |                   |
                    v                   v
          +------------------+   +------------------+
          |   Home Loader    |   | Detail Loader    |
          | load_home(query) |   | load_note(id)    |
          +--------+---------+   +--------+---------+
                   |                      |
                   v                      v
          +------------------+   +------------------+
          | recent_notes     |   | get_note         |
          | or search        |   | replies          |
          | deleted_notes    |   | origins          |
          +--------+---------+   | other_versions   |
                   |             +--------+---------+
                   +----------------------+
                              |
                              v
                    +-------------------+
                    |  DesktopCore      |
                    |  ServiceWrapper   |
                    +---------+---------+
                              |
                              v
                    +-------------------+
                    | SynapService      |
                    | synap-core        |
                    +---------+---------+
                              |
                              v
                    +-------------------+
                    | redb + indexes    |
                    +-------------------+
```

## 哪些能力是重写第一阶段必须保留的

必须保留：

- `init`
- `recent_notes`
- `search`
- `deleted_notes`
- `get_note`
- `replies`
- `origins`
- `other_versions`
- `create_note`
- `reply_note`
- `edit_note`
- `delete_note`
- `restore_note`

可以暂缓不做：

- timeline session
- tag recommendation
- tag search
- sync
- 更复杂分页

也就是说，desktop 第一版重写完全可以先只围绕“本地笔记 + 关系读取 + 版本编辑 + 删除恢复”闭环来做。

## 结论

当前 desktop 对 core 的依赖很稳定，适合在重写时做成两层：

1. `DesktopCore` 适配层
2. `load_home` / `load_note_detail` 这样的聚合 use case 层

这样 UI 重写时不会被 `SynapService` 的底层细节拖住，同时也保留了现在最关键的业务语义：

- 最近笔记主列表
- 搜索
- 回复关系
- 版本链
- 编辑生成新版本
- 删除 / 恢复
