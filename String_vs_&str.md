# String与&str的区别
在刚学习rust的时候，你肯定碰到过这种情况：你试图使用String，但烦人的编译器却告诉你，这个看起来像String的东西，其实并不是String。

我们看个超级简单的例子：函数`greet(name: String)`入参是String，执行后会打印一串字符。
```
fn main() {
  let my_name = "Pascal";
  greet(my_name);
}

fn greet(name: String) {
  println!("Hello, {}!", name);
}
```
编译上面的代码，会报如下错误：
```
error[E0308]: mismatched types
 --> src/main.rs:3:11
  |
3 |     greet(my_name);
  |           ^^^^^^^
  |           |
  |           expected struct `std::string::String`, found `&str`
  |           help: try using a conversion method: `my_name.to_string()`

error: aborting due to previous error

For more information about this error, try `rustc --explain E0308`.
```
幸运的是，Rust编译器的提示信息非常友好。很容易可以看出是类型不匹配造成的，函数期望传入`String`类型，实际传入的是`&str`。编译器甚至还为我们提供了解决方案：将第3行的代码改为`let my_name = "Pascal".to_string()"`。

## String类型
在探索`String`类型之前，我们先研究一下`Rust`是如何存储数据的。

在上一个例子中，我们创建了一个`String`，其内存分布如下所示：
```
                     buffer
                   /   capacity
                 /   /  length
               /   /   /
            +–––+–––+–––+
stack frame │ • │ 8 │ 6 │ <- my_name: String
            +–│–+–––+–––+
              │
            [–│–––––––– capacity –––––––––––]
              │
            +–V–+–––+–––+–––+–––+–––+–––+–––+
       heap │ P │ a │ s │ c │ a │ l │   │   │
            +–––+–––+–––+–––+–––+–––+–––+–––+

            [––––––– length ––––––––]
```
其中`String`对象`my_name`存储在栈中。对象中包含一个指针，指向堆中存储实际数据的buffer，同时也维护了buffer的容量和长度。所以`String`类型的对象大小是固定的。
