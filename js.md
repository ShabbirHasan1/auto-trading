您是否尝试过重载下标运算符 `[]` 获取自定义类型的切片？

比如，我们拥有一个切片 `let array: &[S]`，我想实现 `array[n]` 返回 `&S`，并且 `&array[a..b]` 返回切片 `&[S]`。

您可能会觉得，这不是一个很简单的需求吗？

虽然您随手就能实现，但是对于一些小白来说可能一头雾水。

在 Rust 中，下标运算符`[]`是这样定义的。

```rust
pub trait Index<Idx: ?Sized> {
    /// The returned type after indexing.
    #[stable(feature = "rust1", since = "1.0.0")]
    type Output: ?Sized;

    /// Performs the indexing (`container[index]`) operation.
    ///
    /// # Panics
    ///
    /// May panic if the index is out of bounds.
    #[stable(feature = "rust1", since = "1.0.0")]
    #[track_caller]
    fn index(&self, index: Idx) -> &Self::Output;
}
```

我们发现，函数的返回值类型是 `&Self::Output`，所以他无法返回值类型，而只能返回引用类型。

这确实是令人感到沮丧的，他使得我们无法使用一些奇技淫巧。

幸运的是，我们可以通过一些黑魔法来绕过这个限制。

```rust
#[derive(Debug)]
struct S {
    inner: [f64],
}

impl core::ops::Deref for S {
    type Target = [f64];

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl core::ops::Index<usize> for S {
    type Output = f64;

    fn index(&self, index: usize) -> &Self::Output {
        self.inner.get(index).unwrap_or(&f64::NAN)
    }
}

impl core::ops::Index<std::ops::Range<usize>> for S {
    type Output = S;

    fn index(&self, index: std::ops::Range<usize>) -> &Self::Output {
        unsafe {
            let this = std::mem::transmute::<_, &[f64]>(self);
            let temp = this.get(index).unwrap_or(&[]);
            std::mem::transmute(temp)
        }
    }
}

#[derive(Debug)]
struct B<'a> {
    inner: &'a S,
}

impl<'a> core::ops::Deref for B<'a> {
    type Target = [f64];

    fn deref(&self) -> &Self::Target {
        self.inner
    }
}

impl<'a> core::ops::Index<usize> for B<'a> {
    type Output = f64;

    fn index(&self, index: usize) -> &Self::Output {
        self.inner.get(index).unwrap_or(&f64::NAN)
    }
}

impl<'a> core::ops::Index<std::ops::Range<usize>> for B<'a> {
    type Output = S;

    fn index(&self, index: std::ops::Range<usize>) -> &Self::Output {
        unsafe {
            let temp = self.inner.inner.get(index).unwrap_or(&[]);
            std::mem::transmute(temp)
        }
    }
}

#[test]
fn main() {
    let array = [1f64, 2f64, 3f64, 4f64, 5f64, 6f64, 7f64];
    let ptr = array.as_slice();
    unsafe {
        let p = B {
            inner: std::mem::transmute(ptr),
        };
        let a = &p[1..6];
        let b = a[1];
        let c = &a[2..5];
        let d = c[2];
        let e = &p[114514..1919810];
        let f = c[114514];
        println!("{:?}", a.len());
        println!("{:?}", b);
        println!("{:?}", c.len());
        println!("{:?}", d);
        println!("{:?}", e.len());
        println!("{:?}", f);
    }
}
```

在这个例子中，我们实现了对 `&[f64]` 进行下标检查，在下标越界的时候返回 `f64::NAN`，在切片下标越界时返回 `&[]`。

这个例子的运行结果如下。

```js
5
3.0
3
6.0
0
NaN
```

看起来还是有点懵对吧，让我们逐步解释每个部分的含义。

```rust
#[derive(Debug)]
struct S {
    inner: [f64],
}
```

定义了一个名为 `S` 的结构体，其中有一个字段 `inner`，类型为 `[f64]`，表示一个由 `f64` 元素组成的动态大小数组。

```rust
impl core::ops::Deref for S {
    type Target = [f64];

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
```

这是一个 `Deref trait` 的实现，用于将类型 `S` 解引用为 `[f64]` 类型。这样，当我们使用 `&s` 得到 `S` 的引用时，可以直接通过 `*` 运算符来访问 `inner` 字段。

```rust
impl core::ops::Index<usize> for S {
    type Output = f64;

    fn index(&self, index: usize) -> &Self::Output {
        self.inner.get(index).unwrap_or(&f64::NAN)
    }
}
```

这是一个针对 `S` 类型的索引操作的实现，允许使用索引运算符 `[]` 来访问 `S` 类型中的元素。如果索引超出了数组的范围，将返回一个 `f64::NAN` 的引用。

```rust
impl core::ops::Index<std::ops::Range<usize>> for S {
    type Output = S;

    fn index(&self, index: std::ops::Range<usize>) -> &Self::Output {
        unsafe {
            let this = std::mem::transmute::<_, &[f64]>(self);
            let temp = this.get(index).unwrap_or(&[]);
            std::mem::transmute(temp)
        }
    }
}
```

这是另一个针对 S 类型的索引操作的实现，允许使用切片操作符 `..` 来获取 `S` 类型的子数组。注意到这里使用了 `unsafe` 块，这是因为在 Rust 中，切片的实际表示方式是一个指向数据的指针和长度的组合，而 `S` 类型是一个具有连续内存布局的结构体，所以可以通过类型转换将切片视为 `S` 类型的实例。

```rust
#[derive(Debug)]
struct B<'a> {
    inner: &'a S,
}
```

定义了一个名为 `B` 的结构体，其中有一个字段 `inner`，类型为 `&'a S`，表示一个指向 `S` 类型的引用。

```rust
impl<'a> core::ops::Deref for B<'a> {
    type Target = [f64];

    fn deref(&self) -> &Self::Target {
        self.inner
    }
}
```

这是一个 `Deref trait` 的实现，用于将类型 `B` 解引用为 `[f64]` 类型。这样，当我们使用 `&b` 得到 `B` 的引用时，可以直接通过 `*` 运算符来访问 `inner` 字段。

```rust
impl<'a> core::ops::Index<usize> for B<'a> {
    type Output = f64;

    fn index(&self, index: usize) -> &Self::Output {
        self.inner.get(index).unwrap_or(&f64::NAN)
    }
}
```

这是一个针对 `B` 类型的索引操作的实现，与之前的 `S` 类型类似，允许使用索引运算符 `[]` 来访问 `B` 类型中的元素。如果索引超出了数组的范围，将返回一个 `f64::NAN` 的引用。

```rust
impl<'a> core::ops::Index<std::ops::Range<usize>> for B<'a> {
    type Output = S;

    fn index(&self, index: std::ops::Range<usize>) -> &Self::Output {
        unsafe {
            let temp = self.inner.inner.get(index).unwrap_or(&[]);
            std::mem::transmute(temp)
        }
    }
}
```

这是另一个针对 `B` 类型的索引操作的实现，允许使用切片操作符 `..` 来获取 `B` 类型的子数组。同样地，这里使用了 `unsafe` 块，并通过类型转换将切片视为 `S` 类型的实例。

以上，如果代码有 bug，欢迎各位编程大神在评论区中指出。