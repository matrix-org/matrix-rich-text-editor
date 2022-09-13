export function computeSelectionOffset(node, offset = 0) {
    if (node && node.nodeType === Node.TEXT_NODE) {
        return offset || node.textContent.length
    } else if (node.hasChildNodes()) {
        return Array.from(node.childNodes).map(childNode => computeSelectionOffset(childNode)).reduce((prev, curr) => prev + curr, 0)
    } else {
        return 0
    }
}