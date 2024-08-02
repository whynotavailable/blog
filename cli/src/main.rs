fn main() {
    println!("Hello, world!");
    let markdown_input = "
```js
let d = 123;
```";
    let parser = pulldown_cmark::Parser::new(markdown_input);

    // Write to a new String buffer.
    let mut html_output = String::new();
    pulldown_cmark::html::push_html(&mut html_output, parser);
    println!("{}", html_output)
}
