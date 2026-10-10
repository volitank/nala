### Messages migrated from legacy/python/po/zh_CN.po.
### Missing messages fall back to English.

log-error = 错误：

log-notice = 注意：

log-warning = 警告：

prompt-continue = 你想继续吗？

prompt-choice = Y/y N/n

root-required = Nala 需要 root 权限才能执行 { $command }

op-auto-purge = 自动清除

op-auto-remove = 自动删除

op-remove = 删除

op-purge = 清除

op-install = 安装

op-reinstall = 重新安装

op-upgrade = 升级

op-downgrade = 降级

op-configure = 配置

op-held = 保留

reason-kept-back = 保留

summary-title = 摘要

summary-total-download = 总下载大小:

summary-disk-required = 所需空间:

summary-disk-free = 释放空间:

summary-total-download-value = 总下载大小: { $size }

summary-disk-required-value = 所需空间: { $size }

summary-disk-free-value = 释放空间: { $size }

summary-nothing = Nala 无事可做。

summary-essential = 以下软件包必不可少！

summary-remove-essential = 您试图删除必备软件包

summary-use-switch = 如果您确定要这样做，请使用 { $switch }。

summary-reboot = 需要重启。

summary-reboot-packages = 以下软件包需要重启。

table-package = 软件包:

table-version = 版本:

table-size = 大小:

table-old-version = 旧版本:

table-new-version = 新版本:

pkg-no-candidate = { $package } 没有安装候选项。

pkg-not-installed = { $package } 未安装

pkg-already-latest = { $package } 已是最新版本 { $version }

pkg-version-missing = 找不到软件包 { $package } 的版本 { $version }

pkg-cache-missing = { $package } 不在缓存中

pkg-recorded-version-missing = 找不到 { $package } 的版本

pkg-version-cache-missing = 找不到软件包 { $package } 的版本 { $version }

pkg-virtual-providers = { $package } 是一个虚拟软件包，由：

pkg-virtual-one = 您只能选择一个。

download-complete = 下载完成：

download-auth-warning = 以下软件包无法通过身份验证！

download-auth-allowed = 配置设置为允许安装未经身份验证的软件包。

download-exit = 应您的要求退出。

progress-working = [Trabajando]

progress-remaining = 剩余时间

progress-fetched = { $size } 耗时 { $time } 秒 ({ $rate }/s)

tui-pkgs = 软件包

file-read-defaults = 无法读取配置文件：{ $path }。将使用默认值

dpkg-removing = 删除：

dpkg-unpacking = 解包：

dpkg-setting-up = 设置：

dpkg-processing = 处理：

list-virtual = 虚拟软件包

show-local-install = 本地安装

show-more-records = 还有 { $count } 条记录，使用 { $switch } 选项查看它们。

show-no-description = 无描述

show-package = 软件包

show-version = 版本

show-architecture = 架构

show-priority = 优先级

show-essential = 必备

show-section = 标签

show-source = 源码

show-installed-size = 安装大小

show-maintainer = 维护者

show-original-maintainer = 原维护者

show-homepage = 主页

show-origin = 来源

show-provides = 提供

show-description = 描述

show-apt-sources = APT 来源

show-depends = 依赖

show-pre-depends = 预依赖

show-suggests = 建议

show-recommends = 推荐

show-conflicts = 冲突

show-replaces = 取代

show-breaks = 破坏

show-enhances = 增强

show-attr-installed = 已安装

show-attr-upgradable-to = 已安装并可升级到 { $version }

show-attr-upgradable-from = 可以从 { $version } 升级

policy-installed = 已安装：

history-empty = 没有历史记录。

history-not-found = 交易 { $id } 不存在。

fetch-no-mirrors = Nala 找不到任何镜像源。

fetch-release-detect = 检测版本时出现问题。

fetch-sources-written = 软件源已写入 { $file }

fetch-score-help = 分数 是下载 Release 文件所需的毫秒数。

update-upgradable = { $count } 个软件包可以升级，运行 '{ $command }' 以查看它们。

update-no-change = 无变化

update-updated = 更新

update-ignored = 忽略

update-downloading = 下载中

update-processing = 处理

install-downloaded = 下载： { $path }

## Additional messages

unknown = 未知

none = 无

error = 错误

mirrors = 镜像：

log-info = 信息：

log-verbose = 详细信息：

log-debug = 调试：

not-implemented = 尚未实现。

subcommand-missing = 未提供子命令

prompt-invalid = “{ $response }”不是有效的回答

prompt-refused = 用户拒绝确认

release-info-title = 软件源信息已更改

release-info-explanation =
    软件源的标识信息在其 Release 文件中发生了更改。
    请在继续之前检查这些更改。

release-info-repository = 软件源

release-info-distribution = 发行版

release-info-changes = 更改

release-info-confirm = 接受这些更改并继续更新吗？

release-info-help = ({ $yes }) 接受 | ({ $no }/Enter/Esc) 拒绝

release-info-accepted = 已接受软件源 { $repository } 的信息更改。

release-info-rejected = 已拒绝软件源 { $repository } 的信息更改。

release-info-noninteractive = 必须明确接受 Release 信息更改。请使用“-o Acquire::AllowReleaseInfoChange=true”重新运行以接受这些更改。

reason-excluded = 已排除

reason-manual-hold = 手动保持

reason-phased = 分阶段更新

reason-phased-percent = 分阶段更新 { $percentage }%

summary-op-count = { $operation } { $count }

summary-op-count-colon = { $operation }：{ $count }

summary-protected = 以下软件包是必需的或受保护的！

summary-remove-protected = 您尝试移除必需的或受保护的软件包

summary-downgrade-assume-yes = 使用 --assume-yes 将会降级软件包。如果确定，请使用“-o APT::Get::allow-downgrades=true”。

summary-held-assume-yes = 使用 --assume-yes 将会更改保持的软件包。如果确定，请使用“-o APT::Get::allow-change-held-packages=true”。

autoremove-config-purge = --remove-config 必须与 --purge 或 autopurge 一起使用。

autoremove-assume-yes = 使用 --assume-yes 时跳过自动移除。请使用 --autoremove 启用它。

table-reason = 原因：

pkg-not-found = 未找到“{ $package }”

pkg-not-found-cache = 在缓存中未找到某些软件包

pkg-mark-install = 无法将“{ $package }”标记为待安装

pkg-reinstall-missing = { $package } 未安装，因此无法重新安装

pkg-invalid-name = 无效的软件包名称：“{ $package }”

pkg-op-conflict = “{ $package }”的操作冲突：{ $operation-a } 与 { $operation-b }

pkg-version-conflict = “{ $package }”的固定版本冲突：{ $version-a } 与 { $version-b }

pkg-invalid-pin = 无效的版本固定：“{ $pin }”

pkg-pin-exact = 固定版本需要准确的软件包名称：“{ $pin }”

pkg-exclude-missing = 排除模式“{ $pattern }”未匹配任何软件包

pkg-virtual-no-providers = { $package } 没有提供者，是纯虚拟软件包

pkg-virtual-select = 选择 { $provider }，而不是虚拟软件包 { $package }

pkg-select = 选择软件包“{ $selected }”，而不是“{ $requested }”

pkg-fix-broken = 无法修复损坏的软件包

pkg-required = 您必须指定一个软件包

download-failed = 下载失败

download-some-missing = 未找到某些软件包。

download-auth-required =
    无法验证某些软件包的身份。
    如果确定，请使用 { $switch }

download-items = 项目

download-source-missing = 无法找到下载“{ $package }”的“{ $version }”版本的来源

download-written = 已将 { $package } 写入 { $path }

download-hash-missing = 未找到“{ $uri }”的哈希值

download-protocol-missing = 未定义协议

download-uri-missing = 未定义 URI

download-malformed = “{ $uri }”的格式不正确！

download-request-failed = “{ $uri }”的下载请求失败

download-stream-failed = 无法从“{ $uri }”读取数据流

download-get = 无法发送下载请求

download-content-length = { $headers } 中不存在 content-length

download-content-str = 将 content-length 转换为字符串失败：{ $headers }

download-content-parse = 解析 content-length 失败：{ $headers }

download-checksum = { $file } 的校验和不匹配

download-no-uris = 无法从任何 URI 下载 { $file }

download-no-space = { $path } 中的可用空间不足：需要 { $required }，可用 { $available }。

download-filename = { $path } 没有有效的文件名！

download-error = 错误：{ $error }

download-select-domain = 为 { $file } 选择 { $domain }

download-start = 开始：{ $uri }，重试次数：{ $retries }

download-finished = 已完成：{ $uri }

progress-nothing = 无需获取任何内容

progress-label = 进度

progress-total = 总计

progress-data = 数据

progress-packages = 软件包

progress-package = 软件包

progress-status = 状态

progress-mirrors = 镜像

progress-speed = 速度

progress-elapsed = 已用时间

progress-finished = 已完成：

progress-update-title = 正在更新软件包列表

progress-download-title = 正在下载软件包

progress-install-title = 正在安装软件包

progress-score-title = 正在测试镜像

progress-last-completed = 最近完成

progress-connections = 连接数

progress-average = 平均

progress-active = 活跃

progress-state = 状态

progress-starting = 正在开始

progress-downloading = 正在下载

progress-idle = 空闲

progress-more-mirrors = 另外 +{ $mirrors } 个镜像 · { $connections } 个活跃连接

tui-upgrade-title = Nala 升级

tui-history-title = Nala 历史记录

tui-help-move = (↑) 上移 | (↓) 下移

tui-help-tabs = (→) 下一个标签页 | (←) 上一个标签页

tui-help-show = (Enter) 显示变更日志 | (s) 显示版本信息

tui-help-confirm = (q) 退出 | (y) 开始升级

tui-help-quit = (q) 退出

tui-changelog-missing = 无法找到变更日志 URI

file-remove = 无法移除 { $path }

file-read = 无法读取“{ $path }”

file-parse-defaults = 无法解析 { $path }，使用默认值

config-parse = 无法解析 { $path }。Nala 不会忽略现有的配置文件。如果此问题发生在软件包升级之后，请将其与 { $path }.dpkg-dist 进行比较

file-create = 无法创建“{ $path }”

file-write = 无法写入“{ $path }”

file-replace = 无法替换“{ $path }”

file-deserialize = 无法反序列化“{ $path }”

fs-failed = 对 { $path } 执行 { $operation } 操作失败

fs-failed-target = 执行 { $operation } 操作失败：{ $path } => { $target }

config-option = 不支持选项“{ $option }”

color-expected = 应为颜色名称、0-255 的索引或 #RRGGBB 十六进制字符串

color-index = 颜色索引必须在 0 到 255 之间

color-unknown = 未知颜色“{ $color }”

color-rgb-components = Rgb 需要三个分量

color-rgb-expected = 应为 Rgb 或 Indexed 颜色

color-modifier-expected = 应为类似 "BOLD | ITALIC" 的修饰符字符串或数组

color-modifier-unknown = 未知修饰符“{ $modifier }”

archive-unsupported = 不支持此归档类型

deb-control-missing = 在 { $path } 中未找到 control 文件

hash-unsupported = 不支持哈希类型 { $type }

hash-unavailable =
    无法检查 { $package } { $version } 的完整性。
    此软件包没有可用的哈希值。

dpkg-child-failed = Dpkg 子进程失败：{ $error }

dpkg-status-utf8 = Dpkg 状态文件描述符返回了无效的 UTF-8

dpkg-exit = Dpkg 已退出，退出码为：“{ $code }”

dpkg-poll = 无法轮询子进程

dpkg-read-status = 无法读取状态文件描述符

dpkg-read-pty = 无法从伪终端读取数据

dpkg-write-pty = 无法将标准输入发送到伪终端

show-no-summary = 无摘要

show-size = 大小

show-sha256 = SHA256

show-archive = 归档

show-codename = 代号

show-component = 组件

show-attributes = 属性

show-obsoletes = 废弃

show-attr-local = 本地

show-attr-auto-removable = 可自动移除

show-attr-automatic = 自动

policy-none = 无

policy-candidate = 候选版本：

policy-version-table = 版本表：

policy-no-versions = 无版本。

policy-origin = 来源

policy-package-files = 软件包文件：

policy-pinned = 固定的软件包：

policy-release = 发行

history-cleared = 已清除 { $count } 条历史记录。

history-not-replayable = 历史记录“{ $id }”未被记录为已应用，因此无法重放

history-no-changes = 历史记录“{ $id }”没有可重放的软件包更改

history-cleared-entry = 已清除历史记录 { $id }。

history-id = ID

history-command = 命令

history-date-time = 日期和时间

history-requested-by = 请求者

history-altered = 已更改

history-status = 状态

history-status-applied = 已应用

history-started = 开始时间

history-finished = 完成时间

history-targets = 请求的目标

history-undo-version-missing = 无法撤销“{ $package }”，因为未记录先前的版本

history-undo-config-only = 无法撤销“{ $package }”，因为尚未实现恢复仅保留配置文件的状态

history-undo-installed-version-missing = 无法撤销“{ $package }”，因为未记录先前安装的版本

history-undo-reinstall = 无法撤销“{ $package }”，因为重新安装没有记录逆操作

history-undo-held = 无法撤销保持的软件包“{ $package }”

history-redo-version-missing = 无法重做“{ $package }”，因为未记录操作后的版本

history-redo-reinstall-missing = 无法重做“{ $package }”，因为未记录重新安装的版本

history-redo-held = 无法重做保持的软件包“{ $package }”

history-clear-target = 清除历史记录需要指定记录选择器或 --all

history-config-undo = 无法撤销已配置的软件包“{ $package }”

history-config-redo = 无法重做已配置的软件包“{ $package }”

history-serialize = 无法序列化历史记录

fetch-none-selected = 未选择任何镜像。

fetch-unsupported = 不支持 { $distro }。

fetch-title = Nala 镜像选择

fetch-score = 得分：

fetch-help = 使用 ↓↑ 移动，空格选择/取消选择，Home/End 跳到顶部/底部，q/Enter 退出。

update-item = { $state }：{ $description }

upgrade-exclude-unsafe =
    无法安全地将选定的软件包排除在升级之外。
    { $error }

upgrade-protect = 保护 { $package }，避免执行 { $reason }

upgrade-reason-upgrade = 升级

upgrade-reason-auto-remove = 自动移除

upgrade-config-missing = 没有配置树！
