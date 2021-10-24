在编写俄罗斯方块游戏之前，我们先来了解一下什么是`crates`。

## 什么是crates
rust中的packages（可执行文件、库）被称为crates。在`crates.io`中可以找到很多`crates`。在使用`SDL2`这个`crates`来编写俄罗斯方块之前，我们需要安装`SDL2`这个依赖库。

## 安装SDL2

## 初始化Rust项目
借助包管理器`cargo`可以很方便创建一个新项目，只需一行命令`cargo new`即可：
```
cargo new tetris --bin
```
执行完后，项目的目录结构如下：
```
tetris/
|
|- Cargo.toml
|- src/
    |
    |- main.rs
```
> 如果`cargo new`命令没有加`--bin`参数，则创建的是`lib.rs`而不是`main.rs`。

然后往`Cargo.toml`加入以下内容：
```
todo
```
作用是声明项目名称为`tetris`、版本号为`0.0.1`、依赖`sdl2`。

`Cargo`的版本声明遵循**SemVer**规范：
```
[major].[minor].[patch]
```
其中：
- 新特性，不向下兼容，需更新`[major]`
- 新特性，向下兼容，需更新`[minor]`
- bug修复，向下兼容，只更新`[patch]`

## Cargo和crates.io
## docs.rs
## Cargo.toml
我们可以直接从仓库中拉去依赖，这样能使用到最新的特性。
> 仓库版没有`crate.io`的发布版稳定

以使用仓库版`sdl2`为例：
```
todo
```
很简单对吧！`Cargo`还可以运行依赖的单元测试、基准测试，指定编译参数、指定依赖的特性等等。

简单起见，我们先聚焦于最基本的功能。
## modules
在深入讨论`mudules`之前，我们先了解一下文件和`modules`的关系。

一个文件或文件夹实际上就是一个`module`。假如项目结构如下：
```
todo
```
在`main.rs`加入如下代码，可以把`another_file.rs`定义为一个`module`：
```
mod another_file;
```
同时，也就可以访问`another_file.rs`中的一切（声明为public的）。

上述定义module的方式只适用于声明和文件在同一目录，不适用于如下场景：
```
todo
```
此时需要做如下三件事：
- 在`subfolder`下新建一个`mod.rs`
- 在`mod.rs`中声明module`another_file`
```
pub mod another_file;
```
- 在`main.rs`中声明module`subfolder`
```
mod subfolder;
```
> 什么是`mod.rs`？`mod.rs`是rust定义的规范-用于导出module，当你需要从一个文件夹中导入模块时，编译器会从该文件夹下寻找`mod.rs`导出的module。

现在可以使用`another_file`中的API了：
```
use subfolder::another_file::some_function;
```
目前为止，我们介绍了如何创建并使用文件、文件夹这两种module，其实还有一种module：module块，声明方式如下：
```
mod a_module {
  pub struct Foo;
}
```
上述代码创建了一个包含一个公共结构体的module。

学习完如何使用module后，我们开始编写俄罗斯方块吧。
## 俄罗斯方块
首先在`main.rs`中加入如下代码：
```
```

