# JumpServerClient

一个基于 Tauri 2 + Vue 3 的 JumpServer SSH 客户端桌面应用，通过访问密钥（Access Key）登录并拉取资产列表，支持多标签 SSH 终端、快捷指令广播、终端内容搜索、深色/亮色主题与多种终端配色方案。

> 本项目只为了方便 SSH 操作，不能连接其他类型的 JumpServer 资产（仅支持 `platform_type` 为 `linux` 的资产）。

> 本项目只在该版本 JumpServer 下进行测试，其余版本无法保证可用  
> 产品:	JumpServer 社区版 GPLv3  
> 版本:	v3.10.21  

![docs/img-home](docs/img-home.png) 

## 使用方式

1. 正常登录 JumpServer 并获取 apiKey
   ![](docs/img-jumpserver-apikey.png)
2. 打开应用，在登录表单中填写：
    - **JMS URL**：如 `https://jumpserver.abc.com`
    - **Key ID**：访问密钥 ID
    - **Secret**：访问密钥
   ![](docs/img-login.png)
3. 点击 **连接 JumpServer**，验证成功后左侧会加载资产列表并显示当前登录用户。
4. 单击资产列表中的服务器，会在右侧新建一个 SSH 终端标签并自动连接。
5. 使用标签栏的左右滚动按钮、鼠标滚轮或按住空白处拖动来滚动标签；支持关闭、切换、拖拽排序。
6. 在左侧顶部可切换终端配色方案、深色/亮色主题，以及刷新资产列表。
7. 在左侧「快捷指令」区域可新增指令；点击指令发送到当前终端，点击广播按钮发送到所有已连接终端。
8. `Ctrl/Cmd + F` 打开终端内容搜索栏，支持下一个/上一个、区分大小写。
9. 拖动左侧侧边栏分隔条可调整宽度。

所有上述设置（登录信息、标签、排序、侧边栏宽度、主题、配色、快捷指令）都会持久化到本地，下次启动自动恢复。

## 功能特性

### 连接与资产

- **访问密钥登录**：通过 JumpServer URL、KeyID、SecretID 调用 `/api/v1/users/profile/` 完成验证并获取用户信息。
- **资产列表展示**：拉取当前用户有权限的 Linux 资产，按 JumpServer 节点目录层级展示名称（悬停显示 IP），而不是扁平 IP 列表。
- **资产搜索**：左侧列表支持按标题、地址、标签实时模糊搜索。
- **资产拖拽排序**：鼠标拖动资产项即可调整顺序，排序结果自动持久化。
- **资产标签**：为任意资产添加自定义标签，标签会参与搜索过滤并持久化保存。
- **连接状态指示**：资产项与终端 Tab 上均会显示绿色/灰色的连接状态圆点。

### 多标签终端

- **多标签 SSH**：单击资产即新建一个 SSH 终端标签，可同时连接多台服务器；快速连点同一资产不会重复开标签。
- **标签栏交互**：支持左右滚动按钮、鼠标滚轮、空白处按住拖动三种方式滚动标签；支持拖拽排序、中键关闭、激活态高亮与自动滚动到可见区域。
- **终端内容搜索**：`Ctrl/Cmd + F` 打开搜索栏，支持下一个/上一个、区分大小写、增量搜索与高亮计数。

### 快捷指令

- **指令管理**：新增/编辑/删除快捷指令，可绑定到指定资产或保持全局。
- **单终端执行**：点击指令即发送到当前活动终端；若绑定了资产则自动打开该资产 Tab 并在连接后发送。
- **广播执行**：指令右侧的广播按钮可一键将该命令发送到所有已连接的终端，方便批量操作。

### 主题与配色

- **深色/亮色主题**：一键切换应用主题，终端配色会同步跟随。
- **终端配色方案**：内置多种终端配色方案（可在左侧列表头部切换），切换后所有已打开的终端实时更新，并带配色预览。

### 其他

- **设置持久化**：登录信息、用户信息、资产标签、资产排序、侧边栏宽度、主题、终端配色方案、快捷指令全部保存到本地 `~/.jumpserverclient/settings.json`。
- **跨平台打包**：支持 macOS（Apple Silicon arm64 / Intel x64 DMG）与 Windows（x64 NSIS 安装包），并提供 GitHub Actions 自动构建发布。
- **Web 链接可点击**：通过 `WebLinksAddon` 让终端中的 URL 可直接点击打开。
- **Unicode 11**：加载 `Unicode11Addon` 以正确处理宽字符与 emoji 宽度。
- **终端自适应**：基于 xterm.js + FitAddon，窗口 resize 或侧边栏拖动时通过 `ResizeObserver` 防抖后自动 fit，并将新尺寸同步到远端 PTY，避免乱码。


## 技术栈

- **前端**：Vue 3 + Pinia + Vite 5 + naive-ui
- **终端**：xterm.js + FitAddon + SearchAddon + WebLinksAddon + Unicode11Addon
- **后端**：Tauri 2 + Rust（`async-ssh2-russh` 纯 Rust SSH 实现、`reqwest`、`tokio`）
- **持久化**：`tauri-plugin-store`

## 项目结构

```
.
├── .github/workflows/build.yml   # macOS + Windows 自动构建发布
├── build/                        # 图标源资源
├── docs/                         # 图标源文件等
├── scripts/                      # 打包辅助脚本
│   ├── mac-arch.mjs              # macOS 架构探测与 .app 路径解析
│   ├── build-dmg.mjs             # 生成 DMG 并复制到 release/
│   ├── fix-mac-icon.mjs          # 修复 macOS 应用图标
│   └── move-app.mjs              # 移动 .app 到 release/
├── src/
│   ├── api/tauri.js              # Tauri invoke 与事件监听封装
│   ├── components/
│   │   ├── MainLayout.vue       # 主布局 + 侧边栏拖动
│   │   ├── SetupForm.vue        # 登录表单
│   │   ├── ServerList.vue       # 资产列表/搜索/标签/快捷指令/配色
│   │   ├── AssetTree.vue        # JumpServer 目录树递归展示
│   │   ├── TerminalPanel.vue    # 多终端容器 + 搜索栏
│   │   └── TerminalTabBar.vue   # 标签栏（滚动/拖拽/关闭）
│   ├── stores/app.js            # Pinia 状态管理
│   ├── styles/
│   │   ├── global.css           # 深色/亮色主题变量
│   │   └── terminal-color-schemes.js  # 终端配色方案集合
│   ├── App.vue
│   └── main.js
├── src-tauri/
│   ├── capabilities/default.json
│   ├── icons/
│   ├── src/
│   │   ├── lib.rs               # JumpServer API、SSH 连接、设置持久化
│   │   └── main.rs              # 应用入口
│   ├── Cargo.toml
│   ├── build.rs
│   └── tauri.conf.json
├── index.html
├── package.json
├── vite.config.js
└── README.md
```

## 开发环境

- Node.js 18+
- pnpm 8+
- Rust 1.77.2+（`rust-toolchain` 稳定通道）
- 打包 Intel DMG 需要 Rust target `x86_64-apple-darwin`（`pnpm build:mac:intel` 会自动安装）

## 安装依赖

```bash
pnpm install
```

> 首次安装会下载 Tauri CLI 并编译 Rust 依赖，请耐心等待。

## 开发调试

```bash
pnpm tauri:dev
```

启动后会自动打开 Tauri 窗口并加载 Vite 开发服务器（`http://localhost:1420`）。

## 构建与打包

### 构建前端

```bash
pnpm build
```

### 打包安装包

```bash
# macOS（输出当前芯片架构的 .app + .dmg）
pnpm build:mac
# Apple Silicon
pnpm build:mac:arm
# Intel
pnpm build:mac:intel
# 同时构建 arm64 与 Intel
pnpm build:mac:all
# 或仅生成 .app（含图标修复并复制到 release/）
pnpm build:onlyapp

# Windows（输出 NSIS .exe 安装包，需在 Windows 主机原生编译）
pnpm build:win
```

当前架构产物位于 `src-tauri/target/release/bundle/`，指定架构时位于 `src-tauri/target/<rust-target>/release/bundle/`。`build:mac*` / `build:onlyapp` 还会额外复制到项目根目录的 `release/`。

### npm scripts 说明

| 脚本 | 说明 |
|------|------|
| `pnpm dev` | 仅启动 Vite 前端开发服务器 |
| `pnpm tauri:dev` | 启动 Tauri 开发模式（前端 + Rust） |
| `pnpm build` | 仅构建前端到 `dist/` |
| `pnpm tauri:build` | Tauri 完整打包（使用 `tauri.conf.json` 中的 targets） |
| `pnpm build:onlyapp` | 仅打包 macOS `.app` 并修复图标、复制到 `release/` |
| `pnpm build:mac` | 打包当前芯片架构的 macOS `.app` + `.dmg`（含图标修复） |
| `pnpm build:mac:arm` | 打包 Apple Silicon（arm64）DMG |
| `pnpm build:mac:intel` | 打包 Intel（x64）DMG，可在 Apple Silicon 上交叉编译 |
| `pnpm build:mac:all` | 依次打包 arm64 与 Intel DMG |
| `pnpm build:win` | 原生编译 Windows NSIS 安装包（需在 Windows 主机运行） |
| `pnpm build:dmg` | 仅执行 DMG 打包脚本（需先有 `.app`） |

## 配置与状态

- 应用设置保存在 `~/.jumpserverclient/settings.json` 中，包含：
  - `jms_url` / `key_id` / `secret` / `user_info`
  - `asset_tags` / `asset_order`
  - `sidebar_width`
  - `theme` / `terminal_color_scheme`
  - `quick_commands`
- 关闭应用时会自动断开所有 SSH 连接。

## 注意事项

- 仅支持连接 `platform_type` 为 `linux` 的资产。
- SSH 连接通过 JumpServer 的 connection-token 协议实现，默认使用 `2222` 端口和 `JMS-<connectionId>` 用户名。
- 调用 JumpServer API 时使用 `danger_accept_invalid_certs(true)`（用于自签名证书内网环境），请仅在可信内网环境中使用。
- `release/` 目录下的 DMG 仅为本地构建产物，正式版本以 GitHub Releases 为准。

## CI/CD

推送到 `v*` 标签时会触发 `.github/workflows/build.yml`，分别在 macOS 和 Windows 构建产物并自动创建 GitHub Release，包含：

- macOS Apple Silicon：`JumpServerClient_<version>_aarch64.dmg`
- macOS Intel：`JumpServerClient_<version>_x64.dmg`
- Windows：`JumpServerClient_<version>_x64-setup.exe`

也可在 Actions 页面手动触发（`workflow_dispatch`）。

## 参与贡献

感谢所有做过贡献的人!

<a href="https://github.com/zhangyunan1994/jumpserver-client/graphs/contributors">
  <img src="https://contrib.rocks/image?repo=zhangyunan1994/jumpserver-client" />
</a>
