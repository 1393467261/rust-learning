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
如果一个对象A持有另一个对象B的引用，那么B的作用域一定大于A的作用域，否则A持有的B的引用指向的是一个非法地址。将该规则应用到`Person`类型后，代码如下：
```
struct Person<'a> {
  car: Option<&' a Car>
}
```
其中`Person<'a>`代表我们声明了一个名为`a`的lifetime，`Option<&'a Car>`代表`Car`对象的lifetime是`a`。
声明了lifetime之后，编译器会保证`Person`对象的生命周期小于等于`a`，从而避免了`car`指向非法地址。
>尽管这是一个非常简单的场景，我们也必须声明lifetime，编译器无法自动补齐。
## 功能实现
类型设计已经完成，让我们开始实行功能吧。
```
impl <'a> Person<'a> {
  fn new() -> Person<'a> {
    Person{
      car: None
    }
  }
  
  fn buy_car(&mut self, c: &'a Car) {
    self.car = Some(c);
  }
  
  fn sell_car(&mut self) {
    self.car = None;
  }
}
```
买卖car的功能已经完成！在下文我们还会实现交易car功能。
简单总结一下，为`struct`注入lifetime后，lifetime成为了类型的一部分（完整的类型是`Person<'a>`），与泛型一样。这就是为什么我们使用`impl <'a> Person<'a>`定义实现，和实现带泛型的`Person`如出一辙。而我们也可以为不同的lifetime定义不同的实现，不过这远远超出了目前我们对lifetime的理解。
## 使用引用
现在把目光转向代码吧，下面的代码可以通过编译：
```
fn main() {
  let car = Car{model: "Honda Civic".to_string()};
  let mut bob = Person::new();
  
  bob.buy_car(&car);
  
  println!("{}", bob.car.unwrap().model);
}
```
创建`bob`和`car`的顺序可以颠倒：
```
fn main() {
  let mut bob = Person::new();
  let car = Car{model: "Honda Civic".to_string()};
 
  bob.buy_car(&car); //OK!
 
  println!("{}", bob.car.unwrap().model);
}
```
如果使用旧版编译器，上面的代码是无法通过编译的，因为编译器会认为`car`比`bob`先被回收。我们也可以通过`scope`模拟此场景：
```
fn main() {
    let mut bob = Person::new();
 
    {
        let car = Car{model: "Honda Civic".to_string()};
 
        bob.buy_car(&car); //编译错误
    }
 
    println!("{}", bob.car.unwrap().model);
}
```
`Car`对象'活得'没有`Person`长，显然违反了lifetime规则。
虽然编译器推断lifetime的功能很强大，但并不能覆盖所有的case。看下面的例子：
```
fn main() {
  let ghibli = Car{model: "Maserati Ghibli".to_string()};
  let mut bob = Person::new();
 
  { 
    let civic = Car{model: "Honda Civic".to_string()};
 
    bob.buy_car(&civic); //Error!
    bob.buy_car(&ghibli);
  }
 
  println!("{}", bob.car.unwrap().model);    
}
```
在编译器看来，`civic`'活得'比`bob`短，无法通过编译。但在开发者看来，内作用域之后（即`civic`被回收之后），没有任何地方用到了`civic`，所以代码没有任何问题。造成这种差异的原因是编译器严格遵循我们生命的lifetime规则`buy_car(&mut self, c : &'a Car)`，它只知道`Car`的lifetime是`a`，并且比`self`活得长。
回顾一下rust中的借用规则：当没有mutable借用时，可以同时存在多个immutable借用。所以多个`People`可以同时拥有一个`Car`：
```
fn main() {
    let mut bob = Person::new();
    let mut alice = Person::new();
    let civic = Car{model: "Honda Civic".to_string()};
 
    bob.buy_car(&civic);
    alice.buy_car(&civic);
 
    println!("Bob has: {}", bob.car.unwrap().model);
    println!("Alice has: {}", alice.car.unwrap().model);
}
```
两个人同时拥有一辆车，这显然超出了我们的认知范围。下文会讨论如何让编译器保证一辆车只能有一个主人。
## 实现交易功能
交易涉及到两个`Person`交换他们的cars，实现如下：
```
impl <'a> Person<'a> {
  fn trade_with(&mut self, other: &mut Person<'a>) {
    let tmp = other.car;
    
    other.car = self.car;
    self.car = tmp;
  }
}
```
需要注意的是，`other`的lifetime为`a`也是类型的一部分，完整类型为`&mut Person<'a>`。
我们可以使用`trade_with()`进行交易了：
```
fn main() {
    let civic = Car{model: "Honda Civic".to_string()};
    let ghibli = Car{model: "Maserati Ghibli".to_string()};
 
    let mut bob = Person::new();
    let mut alice = Person::new();
 
    bob.buy_car(&civic);
    alice.buy_car(&ghibli);
 
    bob.trade_with(&mut alice);
 
    println!("Bob has: {}", bob.car.unwrap().model);
    println!("Alice has: {}", alice.car.unwrap().model);
}
```
你可以尝试如下戏耍一下编译器，不过逃不过编译器的法眼。
```
fn main() {
    let mut bob = Person::new();
    let civic = Car{model: "Honda Civic".to_string()};
     
    {
        let ghibli = Car{model: "Maserati Ghibli".to_string()};
        let mut alice = Person::new();
 
        bob.buy_car(&civic);
        alice.buy_car(&ghibli); //Error!
 
        bob.trade_with(&mut alice);
   
        println!("Alice has: {}", alice.car.unwrap().model);
    }
 
    println!("Bob has: {}", bob.car.unwrap().model);
}
```
编译器可以推断出`bob`会成为`Maserati Ghibli`的主人，而`Maserati Ghibli`会先于`bob`回收。
## 使用借用规则
我们先回顾一下，如果一个对象持有另一个对象的引用，那么就要严格遵循借用规则。
只要没有mutable借用，你可以同时借用多个immutable引用：
```
let ghibli = Car{model: "Maserati Ghibli".to_string()};
let mut bob = Person::new();
 
bob.buy_car(&ghibli); //bob borrows ghibli immutably
 
let p1 = &ghibli; //More immutable borrows are OK
let p2 = &ghibli;
```
mutable借用是排他的，不能同时有其他任何借用。
```
let mut ghibli = Car{model: "Maserati Ghibli".to_string()};
let mut bob = Person::new();
 
bob.buy_car(&ghibli); //bob borrows ghibli immutably
 
let p1 = &mut ghibli; //Can't do this.
```
当一个对象被其他人借用时，不能移动该对象，否则其他人访问到的是非法地址。
```
let mut ghibli = Car{model: "Maserati Ghibli".to_string()};
let mut bob = Person::new();
 
bob.buy_car(&ghibli); //bob borrows ghibli
 
let g = ghibli; //Can't move
```
下面是另一个逻辑正确却无法通过编译的例子：
```
let civic = Car{model: "Honda Civic".to_string()};
let mut ghibli = Car{model: "Maserati Ghibli".to_string()};
let mut bob = Person::new();
 
bob.buy_car(&ghibli);
bob.buy_car(&civic);
 
let p1 = &mut ghibli; //Still Can't do this
```
在开发者看来，当我们mutably借用`ghibli`的同时，没有任何其他借用（`bob`已经买了辆新车），但是编译器无法正确推断这种场景。
## 避免共享引用
上文我们提到过两个`Person`可以共享一个`Car`引用，这种设计是不合理的。比如车的主人无法修补（需要mutable借用）漏气的车胎。更合理的设计是`Person`可变借用`Car`，此时车主人可以随意修补车胎了，其他任何人无法借用他的车，immutable借用也不行。
```
struct Car {
    model : String
}
 
struct Person<'a> {
    //Hold a mutable reference
    car:Option<&'a mut Car>
}
 
impl <'a> Person<'a> {
    fn new() -> Person<'a> {
        Person{
            car: None
        }
    }
 
    fn buy_car(&mut self, c : &'a mut Car) {
        self.car = Some(c);
    }
 
    fn sell_car(&mut self) {
        self.car = None;
    }    
    
    fn trade_with<'b>(&mut self, other : &'b mut Person<'a>) {
 
        let tmp = other.car.take();
     
        other.car = self.car.take();
        self.car = tmp;
    } 
}
 
fn main() {
    let mut civic = Car{model: "Honda Civic".to_string()};
    let mut ghibli = Car{model: "Maserati Ghibli".to_string()};
 
    let mut bob = Person::new();
    let mut alice = Person::new();
 
    bob.buy_car(&mut civic);
    alice.buy_car(&mut ghibli);
 
    bob.trade_with(&mut alice);
 
    println!("Bob has: {}", bob.car.unwrap().model);
    println!("Alice has: {}", alice.car.unwrap().model);
}
```
上面的代码可以正常编译。需要注意的是，在`trade_with()`方法中，我们为`other`定义了一个新的lifetime。因为`other`的lifetime可能与`&self`有所不同。
## 结语
好吧，正如我们讨论过的，引用的模型也不是很难。不过这个模型并不总是有效的，比如：
```
fn shop_for_car(p : &mut Person) {
    let car = Car{model: "Mercedes GLK350".to_string()};
 
    p.buy_car(&car); //Error! car doesn't live long enough
}
```
因为`car`活得比`Person`短，那么我们应该如何引用在函数里创建的对象呢？答案是通过Box::new在堆中创建对象，具体会在part III中讨论。
