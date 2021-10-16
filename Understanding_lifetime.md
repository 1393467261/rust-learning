# 深入理解lifetime（上）
是什么让rust与众多语言比，更加与众不同呢？当然是lifetime。同时lifetime也是rust最难理解的一部分，不过十分值得学习。因为它帮助我们解决了编程中难以绕过的两道坎：
- 内存管理 - rust的内存管理无需`GC`和`引用计数`。编译器可以计算每一个对象的lifetime并且在正确的地方释放它，并且能推断代码是否存在内存安全问题，比如重复释放、释放后使用等等。
- 竞态条件 - rust有着严格的所有权规则。对于一个对象，允许多线程读、单线程写，从而解决了多线程下数据安全问题。
## 示例
我们先设计一个文本编辑器类型`TextEditor`，其包含一个`String`类型的私有成员变量。用户可以读取编辑器的文本内容，但不能直接通过调用`String`的方法修改内容。
## 开始吧
最简单的版本：
```
pub struct TextEditor {
  text: String // 私有成员变量
}

impl TextEditor {
  pub fn new() -> TextEditor {
    TextEditor {
      text: String::new()
    }
  }
  
  // 修改文本
  pub fn add_char(&mut self, ch: char) {
    self.text.push(ch);
  }
}

fn main() {
  let mut editor = TextEditor::new();
  
  editor.add_char('a');
  editor.add_char('b');
  editor.add_char('c');
}
```
## 读取文本
上面的代码可以正常编译（得到了编译器的认可，是不是很酷），不过现在用户还无法读取文本内容。
现在，我们需要为`TextEditor`添加一个读取`text`成员变量的方法，我们有以下几种方法可选择：
### 方法1-cloning
克隆变量`text`并返回
```
impl TextEditor {
  pub fn get_text_clone(&self) -> String {
    return self.text.clone();
  }
}
```
克隆一个`String`需要在申请新的堆内存并且拷贝原始内容。这种方式很容易实现，在文本较小时完全可以接受。当文本较大时，克隆会消耗更多CPU、内存。当然，java是这样实现的，不过在rust世界里，必须追求性能最大化。
### 方法2-copying
事实上这种方法并不可取，因为不会得到编译器的允许。但为了内容完整性，我们先忽略编译问题。当我们返回`text`的一个copy时会发生什么呢？Copying一个`String`变量确实会创建一个新的`String`，但两个变量共享一个底层buffer。
```
impl TextEditor {
  pub fn get_text_copy(&self) -> String {
    return self.text;
  }
}

let my_txt = editor.get_text_copy();
```
为什么rust不允许上面的代码呢？因为`text`的所有权转移到了`my_txt`上，导致无法再次该方法。rust转移所有权是为了解决重复释放的问题。如果没有这个规则，就可能存在多个共享一个buffer的`String`对象，每一个对象都会释放一次buffer。
### 方法3-返回引用
在这种方法中，我们返回`text`变量的引用。这样不仅提高了性能，而且返回的是不可变引用，正好满足只读的需求。完美。
```
impl TextEditor {
  //Other methods omitted ...
  
  pub fn get_text(&self) -> &String {
    return &self.text;
  }
}
  
//Use the method
let mut editor = TextEditor::new();
  
editor.add_char('a');
editor.add_char('b');
editor.add_char('c');
  
let my_txt = editor.get_text();
  
println!("{}", my_txt);
```
上面的代码看起来十分好理解，但背后的逻辑却十分复杂。因为你无形中借助了lifetime，你甚至都不知道吧。
让我们来谈谈返回`text`引用会有什么严重的后果吧！如果变量`editor`被回收，而`my_txt`还在使用，那么`my_txt`指向了一段被回收的内存，著名的`Use after free`被触发了。
为了解决`Use after free`问题，lifetime闪亮登场！在上面的代码中，编译器悄悄地植入了lifetime声明，以保证`editor`变量一定会'活得'比`my_txt`更长，从而避免了`Use after free`。
我们试图模拟`Use after free`，编译器会马上检测到这个问题。
```
fn main() {
  let my_txt;
  
  {
    let mut editor = TextEditor::new();
  
    editor.add_char('a');
    editor.add_char('b');
    editor.add_char('c');
  
    my_txt = editor.get_text();
  } //Variable editor gets destroyed.
  
  println!("{}", my_txt); //Use after free. Not possible.
}
```
## Lifetime声明
在上文中，编译器帮我们注入lifetime声明，为什么呢？因为我们的case十分简单，很容易推断出返回的`&String`只能来自`&self`，所以`&self`必须'活得'比返回值长。对于这种简单的case，编译器可以自动注入lifetime声明，效果如下：
```
impl TextEditor {
  pub fn get_text<'a>(&'a self) -> &'a String {
    return &self.text;
  }
}
```
这里的'a'代表的是一段时间，即lifetime。命名没有限制，可以是`b`、`my_grandma`。通常我们会使用诸如`a`、`b`、`c`这种名称。
在上面的代码中，`get_text<'a>`代表我们声明了一个叫`'a`的lifetime，`(&'a self)`代表`&self`的lifetime为`'a`，`&'a String`代表返回值的lifetime为`'a`。`get_text`的返回值的lifetime必须小于等于`'a`，而上文代码中的`my_txt`的作用域大于`editor`，违背了这一原则，导致编译失败。
## 引用检查
即使编译器帮我们注入了lifetime，`Use after free`还是无法完全避免。比如当`TextEditor`手动回收`text`时，`my_txt`指向的是被回收的内存。来看具体的代码吧，首先我们为`TextEditor`新增一个方法：
```
impl TextEditor {
  pub fn reset(&mut self) {
    self.text = String::new();
  }
}
```
然后我们尝试干一些让程序崩溃的事：
```
let mut editor = TextEditor::new();

editor.add_char('a');
editor.add_char('b');

let my_txt =editor.get_text();

editor.reset();

println!("{}", my_txt); // Use after free
```
调用`reset()`后，`my_txt`指向的`String`被回收了，访问`my_txt`会得到完全不符预期的结果。幸运的是，rust的引用检查避免了这一切的发生。引用检查的工作原理如下。
`text`是`String`的所有者，`my_txt`只是向`text`借了一个引用。在rust中，如果你借用了某对象的成员变量，那么代表你也同时借用了该对象。换句话说，`my_txt`借用了`editor`。当你调用`editor.reset()`时，你又借用了`editor`的mutable引用。这显然违背了借用原则-借用mutable引用的前提是没有其他借用，immutable借用也不行。
让我们看看另一种场景：
```
fn main() {
    let mut editor = TextEditor::new();
 
    editor.add_char('a');
    editor.add_char('b');
 
    let my_txt = editor.get_text();
 
    println!("{}", my_txt);
 
    editor.add_char('c'); //编译错误
 
    println!("{}", my_txt);
}
```
记住，`my_txt`是不可变&String引用，当这个借用还在有效期时，我们不能修改`String`。当一个对象没有其他任何借用时，才能被修改。在并发编程中，借助这个规则，你会尝到很多甜头的。
最后，我们看看正确的版本吧。
```
fn main() {
  let mut editor = TextEditor::new();
  
  editor.add_char('a');
  editor.add_char('a');
  
  let my_txt = editor.get_text(); // 只读借用开始
  
  println!("{}", my_txt); // 只读借用结束
  
  editor.add_char('c'); // 借用开始
}
```
在这个版本中，编译器推断出`my_txt`在`println!()`后便不再使用了，即借用结束了，可以进行修改操作。
## 对比其他语言
如果你熟悉java或Objective-C，你也许想知道为什么在rust中才有lifetime。这很好解释。java通过GC避免了`Use after free`问题，只要有任何变量指向`String`，就不会被回收。Objective-C则是通过引用计数解决此问题。在我们的例子中，有两个引用-一个是`text`本身，一个是`my_txt`，当引用计数为0时，也就是`text`和`my_txt`都不再使用时，`String`才会被回收。
GC的runtime一般不会很轻量，有时还会导致'整个世界'处于暂停状态。引用计数会产生额外的负载，并且不能保证100%可靠。rust的lifetime则是优雅地解决了内存回收问题，完全不借助任何runtime。
