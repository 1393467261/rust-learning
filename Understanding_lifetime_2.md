# 深入理解lifetime（下）
在[上一部分](Understanding_lifetime.md)中我们讨论了lifetime所解决的问题，并通过函数的例子展示了其是如何工作的。在这一部分，我们会探索lifetime如何帮助我们构建对象包含关系（一个对象持有另一个对象的引用）。
## 需求
我们需要设计一个`Person`类型，一个person可以有一个`Car`，也可以买卖cars。两个persons之间可以交换他们的cars。
## 类型设计
`Car`类型十分简单。
```
struct Car {
  model: String
}
```
`Person`可以持有`Car`。
```
struct Person {
  car: Option<Car>
}
```
从内存管理上看，这样设计很易懂，但也存在几个问题。一个`Car`严格意义上并不是`Person`的一部分，买卖`Car`需要copy。为了不影响性能，需要避免copy。所以可以考虑让`Person`保存`Car`的引用，不过这会使情况变得复杂得多。
### 持有引用
