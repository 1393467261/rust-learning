## 拼接字符串
拼接字符串比较耗性能。了解rust中拼接字符串的方式以及原理，在不同场景下灵活运用，可以达到性能优化的目的。

**rust有三种拼接字符串的方式：**
- move
- clone
- mutating

**示例代码如下：**
```
fn main() {
  by_moving();
  by_cloning();
  by_mutating();
}

fn by_moving() {
  let hello = "hello ".to_string();
  let world = "world!";
  
  // hello move到变量hello_world中
  let hello_world = hello + world;
  // move后，hello不能再被使用
  println!("{}", hello_world);
  // 打印'hello world'
}

fn by_cloning() {
  let hello
}
```
**by_moving的原理：**
- world追加到hello（复用hello的内存）
- hello到所有权转移到hello_world
> 拼接完后，hello不能再被使用


