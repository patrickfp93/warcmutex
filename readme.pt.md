# WarcMutex

[![Crates.io](https://img.shields.io/crates/v/warcmutex.svg)](https://crates.io/crates/warcmutex)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Descrição

A crate WarcMutex é uma biblioteca em Rust que oferece uma macro atributo para mods, structs e impls. O propósito dessa biblioteca é gerar um wrapper que permite que a struct tenha a capacidade de ser utilizada com o controle de referência assíncrona chamado Arc e o Mutex para controle de mutação assíncrona.

## Instalação

Para utilizar a crate WarcMutex, adicione a seguinte dependência ao seu `Cargo.toml`:

```toml
[dependencies]
warcmutex = "1.0.0"
```

## Exemplo de Uso

Aqui está um exemplo simples de uso do WarcMutex:

```rust
#[warcmutex]
pub struct MyStruct {
    value: usize,
}

#[warcmutex]
impl MyStruct {
    pub fn new() -> Self {
            Self{
                value: 0
            }
    }
    pub fn reset(&mut self) {
        self.value = 0;
    }
    pub fn value_mut(&mut self) -> &mut usize{
        &mut self.value
    }
    pub fn get_value(&self) -> usize {
        self.value
    }
}
```
Após a aplicação do atributo, o código é transformado em:

```rust
pub struct MyStructBase {
    value: usize,
}

impl MyStructBase {
    pub fn new() -> Self {
            Self{
                value: 0
            }
    }
    fn reset(&mut self) {
        self.value = 0;
    }   
    pub fn value_mut(&mut self) -> &mut usize{
        &mut self.value
    }
    fn get_value(&self) -> usize {
        self.value
    }
}

pub struct MyStruct {
    base: MyStructrc<Mutex<MyStructBase>>,
}

impl MyStruct {
    pub fn new() -> Self {
        return Self{
            base: Arc::new(Mutex::new(MyStructBase::new())),
        };
    }
    pub fn reset(&mut self) {
        self.base.lock().unwrap().reset();
    }
    pub fn get_value(&self) -> usize {
        return self.base.lock().unwrap().get_value();
    }
}
impl MyStruct {
    pub fn lock(&mut self) -> LockResult<MutexGuard<'_, MyStructBase>>{
        return self.base.lock();
    }
}

impl Clone for MyStruct{
    fn clone(&self) -> Self {
        return Self { base: self.base.clone() };
    }
}

unsafe impl Send for MyStruct{}
unsafe impl Sync for MyStruct{}
```
Após utilizar o atributo `#[warcmutex]`, a struct A será automaticamente reescrita com a adição de um campo base que contém um `Arc<Mutex<ABase>>`. As funções da struct A serão então implementadas para acessar o campo base de forma segura.

### Método lock
Assim como o método presente no `Mutex<T>` a sua função para bloqueio de utlização dando acesso á funções que retornam referências como no exemplo abaixo:
```rust
fn main(){
    use my_module::MyStruct;
    let mut a = MyStruct::new();
    *a.lock().unwrap().value_mut() = 10;
    assert_eq!(a.get_value(), 10);
}
``` 
### Módulos
É possível simplificar o uso do `#[warcmutex]` colocando o mesmo como atributo de módulo que dará o mesmo efeito como no exemplo anterior:
```rust
use warcmutex::warcmutex;
    #[warcmutex]
    mod my_module {
        /// other mods, structs and/or impls...
    }

```
> No uso em módulo será incluido todas as structs, impl e mods se exceções.

> O uso do atributo pode não dar certo com outros atributos


## Contribuição

O projeto WarcMutex é mantido principalmente por um único programador apelidado de PFP, mas aceita contribuições da comunidade. No entanto, é importante que as contribuições estejam dentro do escopo da função principal do projeto.

## Licença

Este projeto está licenciado sob a licença MIT. Consulte o arquivo [LICENSE](LICENSE) para obter mais detalhes.

