import React from "react";
import { MathJax } from "better-react-mathjax";
import { BasicComponent } from "../types";
import { useAppSelector } from "../../state/hooks";
import { renderingOnServerSelector } from "../../state/redux-slices/global";

type MathData = { state: { latex: string } };

export const Math: BasicComponent<MathData> = ({ node }) => {
    const onServer = useAppSelector(renderingOnServerSelector);
    if (onServer) {
        return <span className="process-math">{node.data.state.latex}</span>;
    }
    // better-react-mathjax cannot handle multiple children (it will not update when they change)
    // so create a single string.
    const latexString = `\\(${node.data.state.latex}\\)`;
    return (
        <MathJax inline dynamic>
            {latexString}
        </MathJax>
    );
};