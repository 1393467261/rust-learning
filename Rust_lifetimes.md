# Rust Lifetimes
Lifetime是rust最为引人注目的特性。
Lifetime让并发、内存分配更加简单，让数据更加安全。
不过Lifetime也很棘手，本文会帮助你理解lifetime的概念和语法。
## lifetime是什么？
Rust是唯一一门无需GC、显式`free`就可以完成堆内存回收的编程语言。通过lifetime，它可以判断一个引用是否合法。
Rust会在编译期计算函数出入参中引用类型的实际lifetime，判断是否符合函数的lifetime声明。
所以每一个引用都有一个lifetime，lifetime是类型的一部分。通常，我们可以省略lifetime声明，因为编译器可以推断并且自动声明。尽管如此，只有知道如何声明lifetime后，你才能编写rust代码。
Lifetime在rust中有两个作用：
- 告诉编译器什么时候解引用指针
- 安全共享数据
>现在你暂时无需知道如何定义lifetime
有时，你需要为已存在的lifetime命名，编译器可以从中知道你想要达到的效果。
## Lifetime从哪来？
Lifetime以撇号开头，后面接着名称，一般是单字母。例如`<'a>`、`<'b>`。
Lifetime会出现在这几种类型定义中：`something<'a>`、`Box<something + 'a>`、`&'a something`。
>你只需要在类型定义中声明lifetime
### 函数
函数是lifetime的主要使用场景。lifetime需要先在函数中定义，才能被其他类型定义所使用。
