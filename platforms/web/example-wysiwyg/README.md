# example-wysiwyg

A minimal example of how to use matrix-wysiwyg in a React project.

## Set up and run

```bash
npm install
npm run dev
```

and open the URL that is printed on the console.

## Creating this project from scratch

* Create the project:
    ```bash
    npm create vite@latest example-wysiwyg -- --template react-ts
    cd example-wysiwyg
    npm install
    npm add '@matrix-org/matrix-wysiwyg'
    ```
* Edit example-wysiwyg/src/App.tsx to look how it looks in this repo.
* Edit example-wysiwyg/index.html to set the page title and remove favicon.
* Delete example-wysiwyg/public/vite.svg and
  example-wysiwyg/src/assets/react.svg.
* Disable eslint for a couple of files.
* Follow the instructions in "Set up and run".
