# dcz (DucaZcript)

This is not completed yet!

A basic compiler programming language that share the similarity with C/C++ lang.

## Installation
Clone this repo and then compile it using cargo

## Dependencies
Newest LLVM (21.1.3)

## Usage
To compile a script file, run:
```sh
$ dcz foobar.dcz -o foobar.o
```

## Sample code
This is a working code for printing a simple "Hello World"
```
extern func printf(i8* str);
func main() -> i32 {
    printf("hello world\n");
    return 0;
}
```
Other samples can be found inside `examples` folder

## Test
Run the test by running: `cargo test`

## Contributing?
Pull requests are welcome. For major changes, please open an issue first
to discuss what you would like to change.

## Roadmap
- [ ] Link object file to executable
- [ ] Support `struct` and `class`
- [ ] Actually complete the AST Checker
- [ ] Support signed and unsigned data type
- [ ] Make it can do 'self-hosting'
- [ ] Support "import" from other file
- [ ] Do some optimization for the executable
- [ ] Make a Wiki page?

## License
[MIT](https://choosealicense.com/licenses/mit/)