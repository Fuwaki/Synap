# Desktop Linux GTK Design

## Technology Choices

- UI toolkit: `gtk4` via `gtk-rs`
- Rust crate version: `gtk4 = 0.11.2`
- Enabled GTK API level: `v4_22`
- Local system GTK version: `4.22.2`
- Core integration: direct adapter over `synap-core::service::SynapService`

选择依据：

- `gtk4 0.11.2` 是当前 crates.io 上的最新稳定版本
- 本机系统库是 `GTK 4.22.2`，因此项目只启用 `v4_22`，不越过系统可用 API
- 先用纯 `gtk4`，避免第一阶段引入 `libadwaita` 等额外 UI 约束

## Phase 1 Scope

第一阶段先完成可运行的桌面骨架，并优先打通真实数据读取：

1. GTK 应用启动与样式系统
2. 左侧操作栏
3. 笔记列表页原型实现
4. 回收站列表页
5. 设置页占位
6. 新建笔记弹窗与保存流
7. 通过 `DesktopCore` 适配层读取 `recent_notes`、`deleted_notes`、`search`
8. 本地 UI 状态管理与列表选择状态

暂不在第一阶段完成：

- 详情编辑器
- 回复链 / 版本链 UI
- 删除 / 恢复操作入口
- 真正的瀑布流布局算法

## Module Layout

```text
desktop_linux/src/
  main.rs        # 启动入口，初始化 core adapter 与 GTK Application
  app.rs         # 主窗口、控件装配、事件绑定、渲染逻辑
  core.rs        # DesktopCore trait + SynapCoreAdapter
  domain.rs      # AppState、HomeData、ContentView、NoteLayout、列表视图模型
  usecase.rs     # load_home 等聚合读取动作
  style.css      # 原型样式
```

## Data Flow

当前实现沿用 `function_design.md` 里的分层，但先落最小闭环：

```text
GTK Widgets
  -> AppState
  -> load_home(query)
  -> DesktopCore
  -> SynapService
```

具体行为：

- 启动时加载 `recent_notes + deleted_notes`
- 点击 `新建笔记` 时立即打开独立模态窗口
- 新建成功后执行 `create_note -> refresh lists -> select created note`
- 搜索时：
  - 笔记页调用 `search`
  - 回收站页先复用 `deleted_notes`，再在 UI 层做本地过滤
- 右侧默认显示笔记列表
- 右侧切到回收站或设置时，左侧显示 `返回笔记列表` 按钮
- 切换右侧内容时保留查询词和列表选择语义

## UI Structure

主界面按原型拆成三块：

1. 左侧 Sidebar
   - `Synap` 品牌标题
   - `新建笔记` 立即动作按钮
   - `返回笔记列表` 条件按钮，仅在右侧不是笔记列表时显示
   - `回收站`
   - `设置`
2. 顶部 Toolbar
   - 页面标题
   - 搜索框
   - 布局模式下拉框
3. 内容区
    - 笔记卡片列表
    - 空状态 / 错误状态占位

4. 新建笔记弹窗
   - `TextView` 输入正文
   - `Entry` 输入可选标签
   - `保存` / `取消`

## Next Milestones

完成第一阶段骨架后，下一步建议顺序：

1. 增加详情面板与 `select_note(id)` 聚合读取
2. 增加新建 / 编辑 / 回复的编辑器窗口或右侧编辑区
3. 接入删除 / 恢复操作
4. 接入 `origins`、`other_versions`、`replies` 关系视图
