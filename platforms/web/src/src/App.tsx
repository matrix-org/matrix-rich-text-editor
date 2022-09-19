import { useWysiwyg } from "./lib/useWysiwyg";

function App() {
    const { ref, modelRef, testRef, isWysiwygReady, wysiwyg, test: { resetTestCase } } = useWysiwyg();

    return (
        <div className="wrapper">
            <div>
                <div className="editor_container">
                    <button type="button"
                        onClick={(e) => wysiwyg.bold()}
                    >bold
                    </button>
                    <div className="editor" ref={ref} contentEditable={isWysiwygReady} />
                </div>
            </div>
            <h2>Model:</h2>
            <div className="dom" ref={modelRef} />
            <h2>Test case: <button type="button" onClick={() => resetTestCase()}>Start from here</button></h2>
            <div className="testCase" ref={testRef}>
                let mut model = cm("");<br />
                assert_eq!(tx(&amp;model), "");
            </div>
        </div>);
}

export default App;
