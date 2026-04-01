# 定制分支更新指引

本文档用于维护这条基于上游 `master` 的轻定制分支。目标不是保留旧分支历史，而是每次都从最新 `master` 重新落两处明确的定制，避免把旧分支里的无关改动一起带回。

## 本分支只保留的两项定制

1. 默认配置目录和缓存目录改为程序所在目录下的兄弟目录：
   - `config`
   - `cache`
2. `similar_videos` 导出的 JSON 中包含 `thumbnail_path`

当前这两项定制在代码里的真实落点：

- `czkawka_core/src/common/config_cache_path.rs`
- `czkawka_core/src/tools/similar_videos/mod.rs`
- `czkawka_core/src/tools/similar_videos/tests.rs`

## 先记住的章程

1. 不要直接把旧分支整体 rebase、merge 或 cherry-pick 到新 `master`。
2. 旧分支历史里混入过无关提交，真正可参考的只有“行为目标”和局部实现思路，不是整串 commit。
3. 每次更新时都先以最新 `master` 建一个新分支，再手工重放这两项定制。
4. 先补测试，再改实现。即使本机没有 Rust 工具链，也要保持这个顺序。
5. 只动与这两项定制直接相关的文件，不顺手夹带其它功能改动。

## 为什么不能整支硬抬

旧分支里曾出现过这些提交：

- `61f6d38 Use exe-local config and cache dirs by default`
- `86b1c67 Include thumbnail path in similar videos JSON`
- `691cdda Simplify thumbnail generation flow`
- `132eb72 Prefer portable config/cache directory`

其中真正需要重放的核心目标，只有前两项对应的行为。后两项不是这次定制分支的必要组成部分。所以下次更新时，应当把旧提交当作“参考证据”，而不是“直接搬运对象”。

## 标准操作流程

### 1. 同步上游主分支

```powershell
git fetch origin master
git switch master
git pull --ff-only origin master
```

如果本地 `master` 已经和 `origin/master` 一致，也要先确认一遍，不要假设它已经是最新。

### 2. 从最新 `master` 新开工作分支

分支名不要绑定死，可以按日期或版本命名。例如：

```powershell
git switch -c self/pathchange-thumbsave-v3
```

如果你就是要继续沿用旧分支名，先单独备份旧分支，再决定是否重写历史。默认更安全的做法是开新分支。

### 3. 先看旧定制，但只看目标落点

参考命令：

```powershell
git show 61f6d38 -- czkawka_core/src/common/config_cache_path.rs
git show 86b1c67 -- czkawka_core/src/tools/similar_videos/mod.rs czkawka_core/src/tools/similar_videos/tests.rs
```

只看这些文件，确认旧定制的意图：

- `config_cache_path.rs`
  - 默认优先使用 `exe_dir/config` 和 `exe_dir/cache`
  - 只有 exe 同级目录准备失败时，才回退到环境变量或 `ProjectDirs`
- `similar_videos/mod.rs`
  - 去掉 `thumbnail_path` 上的 `#[serde(skip)]`
- `similar_videos/tests.rs`
  - 保留一条序列化测试，确保 `thumbnail_path` 在 JSON 里存在

### 4. 先补测试，再改实现

最低限度应包含下面两条测试：

1. `czkawka_core/src/common/config_cache_path.rs`
   - 测试 exe 同级 `config` / `cache` 会被创建并作为默认目录使用
2. `czkawka_core/src/tools/similar_videos/tests.rs`
   - 测试 `VideosEntry` 序列化时包含 `thumbnail_path`

建议优先跑定向测试，而不是一上来全量测试。

### 5. 再做最小实现

本轮已验证过的最小实现方式如下：

- 在 `config_cache_path.rs` 中增加一个可测试的 exe-local 目录解析辅助函数
- `set_config_cache_path(...)` 先尝试 exe 同级 `config` / `cache`
- 失败后再走 `CZKAWKA_CONFIG_PATH` / `CZKAWKA_CACHE_PATH` / `ProjectDirs`
- 在 `similar_videos/mod.rs` 中仅移除 `thumbnail_path` 的 `#[serde(skip)]`

不要把旧分支中和视频缩略图生成流程、video optimizer、GitHub workflow 相关的改动一起带回来。

## 推荐验证命令

如果本机有 Rust 工具链，优先跑：

```powershell
cargo test -p czkawka_core test_resolve_exe_local_config_and_cache_dirs_prefers_exe_siblings -- --exact
cargo test -p czkawka_core test_videos_entry_serializes_thumbnail_path -- --exact
```

然后再补一次更宽的检查：

```powershell
cargo test -p czkawka_core similar_videos
cargo test -p czkawka_core config_cache_path
git diff --check
```

如果当前机器没有 `cargo`，要明确记录验证边界，不要写成“测试已通过”。

## 提交前检查清单

提交前至少核对下面这些点：

1. `git diff --name-only` 里只应出现这次定制相关文件和文档。
2. `thumbnail_path` 不再带 `#[serde(skip)]`。
3. `config_cache_path.rs` 的默认路径优先级仍然是：
   - exe 同级 `config` / `cache`
   - 环境变量覆盖
   - `ProjectDirs` / Android 默认目录
4. 没有把旧分支里的其它功能改动一起带进来。
5. 最终说明里要分清：
   - 已修改
   - 已写文档
   - 已验证
   - 未验证

## 这次重放的参考事实

本次不是在旧分支上强行抬历史，而是：

1. 先确认 `master` 已更新到 `8847e56`
2. 新建分支 `self/pathchange-thumbsave-v2`
3. 只重放两项目标行为
4. 补了对应测试和本指引文档

下次继续更新时，重复这个流程即可，不要反过来把 `v2` 当成新的上游主线。
