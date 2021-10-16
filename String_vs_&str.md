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
在探索`String`类型之前，我们先研究一下Rust是如何存储数据的。

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
另外`String`也支持动态扩容底层buffer，这也是与`&str`不同之处。例如，我们可以使用`.push_str()`方法追加字符串，不过这可能会触发底层buffer的扩容（mutable变量才可以追加字符串）。
```
let mut my_name = "Pascal".to_string();
my_name.push_str(" Precht");
```
事实上，如果对Rust的Vec<T>有所了解，那么你就可以很容易理解`String`，因为它们的工作原理是一样的，除了`String`需要对UTF-8字符串做特殊处理。
## string slices类型
String slices（即`str`）实际上是一段UTF-8字符串的引用，我们也可以通过string literals来创建string slices。
如果我们需要获取`my_name`中的last name，我们仅需要引用`my_name`的一部分：
```
let mut my_name = "Pascal".to_string();
my_name.push_str(" Precht");
  
let last_name = &my_name[7..];
```
我们创建了下标从7到buffer末尾的一个string slice`my_name`，其内存结构如下：
```
            my_name: String   last_name: &str
            [––––––––––––]    [–––––––]
            +–––+––––+––––+–––+–––+–––+
stack frame │ • │ 16 │ 13 │   │ • │ 6 │ 
            +–│–+––––+––––+–––+–│–+–––+
              │                 │
              │                 +–––––––––+
              │                           │
              │                           │
              │                         [–│––––––– str –––––––––]
            +–V–+–––+–––+–––+–––+–––+–––+–V–+–––+–––+–––+–––+–––+–––+–––+–––+
       heap │ P │ a │ s │ c │ a │ l │   │ P │ r │ e │ c │ h │ t │   │   │   │
            +–––+–––+–––+–––+–––+–––+–––+–––+–––+–––+–––+–––+–––+–––+–––+–––+
```
需要关注的是，`last_name`没有存储buffer的容量，因为它只是指向`String`的一个切片，容量由实际存储数据的`String`来维护。所以string slice（即`str`）是不区分大小的，而且string slice实际是是一个引用，所以类型是`&str`而不是`str`。
到目前为止，我们知道了`String`,`&String`,`str`和`&str`的区别，那么是时候回到原本的问题上了。
## string literals类型
前面提到，有两种方式可以创建string slice：从`String`中创建或从string literals中创建。
带双引号的一段字符串就是string literals：
```
let my_name = "Pascal Precht"; // 注意这是&str而不是String
```
那么问题来了，如果`my_name`是`String`的一个切片，而这个`String`被别人持有，那么究竟是谁持有`String`的所有权呢？
string literals有点特殊，它是作为程序的一部分存储在一段只读内存中。换句话说，存储string literals不依赖堆内存，并且在栈中可以访问那一段只读内存。
```
            my_name: &str
            [–––––––––––]
            +–––+–––+
stack frame │ • │ 6 │ 
            +–│–+–––+
              │                 
              +––+                
                 │
 preallocated  +–V–+–––+–––+–––+–––+–––+
 read-only     │ P │ a │ s │ c │ a │ l │
 memory        +–––+–––+–––+–––+–––+–––+
```
恭喜，我们进一步了解了`String`和`&str`的差异。不过先别着急庆祝，我们还要知道它们各自的使用场景。
## String和&str的最佳实践
很显然，我们需要考虑许多因素。不过通常情况下，当我们不需要所有权，或不需要修改字符串时，应该使用`&str`。有了这个最佳实践，我们可以优化下`greet()`函数：
```
fn greet(name: &str) {
  println!("Hello, {}!", name);
}
```
Wait a minute，万一函数调用方只有`String`，由于某些原因不能将`String`转化为`&str`呢？完全不是问题，rust可以自动把String引用（即`&String`）转化为`&str`。
至此，我们的优化版```greet()```完成了：
```
fn main() {
  let first_name = "Pascal";
  let last_name = "Precht".to_string();

  greet(first_name);
  greet(&last_name); // `last_name` is passed by reference
}

fn greet(name: &str) {
  println!("Hello, {}!", name);
}
```

  
  
  
