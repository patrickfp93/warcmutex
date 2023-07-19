# WarcMutex

[![Crates.io](https://img.shields.io/crates/v/warcmutex.svg)](https://crates.io/crates/warcmutex)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Description

The WarcMutex crate is a Rust library that provides a macro attribute for mods, structs, and impls. The purpose of this library is to generate a wrapper that allows the struct to be used with the asynchronous reference control called Arc and the Mutex for asynchronous mutation control.

## Installation

To use the WarcMutex crate, add the following dependency to your `Cargo.toml`:

```toml
[dependencies]
warcmutex = "1.0.0"
```

## Example Usage

Here's a simple example of using WarcMutex:

```rust
#[warcmutex]
pub struct MyStruct {
    value: usize,
}

#[warcmutex]
impl MyStruct {
    pub fn new() -> Self {
        Self {
            value: 0,
        }
    }

    pub fn reset(&mut self) {
        self.value = 0;
    }

    pub fn value_mut(&mut self) -> &mut usize {
        &mut self.value
    }

    pub fn get_value(&self) -> usize {
        self.value
    }
}
```

After applying the `#[warcmutex]` attribute, the code is transformed into:

```rust
pub struct MyStructBase {
    value: usize,
}

impl MyStructBase {
    pub fn new() -> Self {
        Self {
            value: 0,
        }
    }

    fn reset(&mut self) {
        self.value = 0;
    }

    pub fn value_mut(&mut self) -> &mut usize {
        &mut self.value
    }

    fn get_value(&self) -> usize {
        self.value
    }
}

pub struct MyStruct {
    base: Arc<Mutex<MyStructBase>>,
}

impl MyStruct {
    pub fn new() -> Self {
        Self {
            base: Arc::new(Mutex::new(MyStructBase::new())),
        }
    }

    pub fn reset(&mut self) {
        self.base.lock().unwrap().reset();
    }

    pub fn get_value(&self) -> usize {
        self.base.lock().unwrap().get_value()
    }
}

impl MyStruct {
    pub fn lock(&mut self) -> LockResult<MutexGuard<'_, MyStructBase>> {
        self.base.lock()
    }
}

impl Clone for MyStruct {
    fn clone(&self) -> Self {
        Self {
            base: self.base.clone(),
        }
    }
}

unsafe impl Send for MyStruct {}
unsafe impl Sync for MyStruct {}
```

After using the `#[warcmutex]` attribute, the `MyStruct` will be automatically rewritten with the addition of a `base` field containing an `Arc<Mutex<MyStructBase>>`. The functions of `MyStruct` will then be implemented to safely access the `base` field.

### Method lock
Similar to the method in `Mutex<T>`, this function is used to lock the usage and gain access to functions that return references as shown in the example below:
```rust
fn main() {
    use my_module::MyStruct;
    let mut a = MyStruct::new();
    *a.lock().unwrap().value_mut() = 10;
    assert_eq!(a.get_value(), 10);
}
```
### Modules
You can simplify the use of `#[warcmutex]` by placing it as an attribute for the module, which will have the same effect as in the previous example:
```rust
use warcmutex::warcmutex;

#[warcmutex]
mod my_module {
    /// other mods, structs, and/or impls...
}
```

> When used on a module, all structs, impls, and mods will be included, with exceptions.

> The use of the attribute may not work well with other attributes.

## Contribution

The WarcMutex project is mainly maintained by a single developer known as PFP but welcomes contributions from the community. However, it's essential that contributions stay within the scope of the project's main function.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for more details.