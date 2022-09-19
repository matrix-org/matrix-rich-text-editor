import { useWysiwyg } from "./lib/useWysiwyg";

function App() {
    const { ref, modelRef, isWysiwygReady, wysiwyg } = useWysiwyg();

    if (!isWysiwygReady) {
        return <span>Loading...</span>;
    }

    return (<div>
        <button type="button"
            onClick={(e) => wysiwyg.bold()}
        >bold
        </button>
        <div className="editor" ref={ref} contentEditable={true} style={{ borderColor: 'black', border: 'solid', padding: '5px', marginTop: '10px' }} />
        <h2>Model:</h2>
        <div className="dom" ref={modelRef} />
    </div>);
}

export default App;
