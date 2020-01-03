# Fractal Explorer

This small Rust application provides the Web Assembly package used for performance comparison between JavaScript and Web Assembly on [whealy.com](http://whealy.com/Rust/mandelbrot.html)

This app is not intended to work stand-alone; instead, it but must be invoked from the JavaScript coding built in to the above web page.

Using [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/) as the compiler and [wasm-bindgen](https://rustwasm.github.io/wasm-bindgen/introduction.html) as the interface to the browser's DOM, this app writes the calculated Mandelbrot and Julia Set images directly to HTML canvas elements.
