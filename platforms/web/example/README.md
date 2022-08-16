# WYSIWYG Web example app

This application provides an example usage of the Matrix WYSIWYG composer.

It is currently based on the WASM generated in `bindings/wysiwyg-wasm` and in
future will be updated to be based on an NPM package of that.

## Running the example

To build the code, in the top-level folder of this repo, run:

```bash
make web
```

Now run a web server serving the files in this directory:

```bash
cd platforms/web/example
python -m http.server
```

Now navigate to [http://0.0.0.0:8000/](http://0.0.0.0:8000/) to play with the
example.

## Testing

To run the tests, perform the above setup, then navigate to
[http://0.0.0.0:8000/test.html](http://0.0.0.0:8000/test.html).
