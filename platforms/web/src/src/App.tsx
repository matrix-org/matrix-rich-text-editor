import { useWysiwyg } from "./lib/useWysiwyg";

function App() {
    const { ref, modelRef, isWysiwygReady, wysiwyg } = useWysiwyg();

    if (!isWysiwygReady) {
        return <span>Loading...</span>;
    }

    return (
        <div className="wrapper">
            <div>
                <div className="editor_container">
                    <button type="button"
                        onClick={(e) => wysiwyg.bold()}
                    >bold
                    </button>
                    <div className="editor" ref={ref} contentEditable={true} />
                </div>
            </div>
            <h2>Model:</h2>

            <div className="dom" ref={modelRef} />
        </div>);
}

export default App;
