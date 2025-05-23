import { JXGObject } from "../jsxgraph-distrib/types";

export type LabelPosition =
    | "upperright"
    | "upperleft"
    | "lowerright"
    | "lowerleft"
    | "top"
    | "bottom"
    | "left"
    | "right"
    | "center";

export function getEffectiveBoundingBox(board: JXGObject) {
    let flippedX = false;
    let flippedY = false;

    let [xmin, ymax, xmax, ymin] = board.getBoundingBox();

    if (xmax < xmin) {
        flippedX = true;
        [xmax, xmin] = [xmin, xmax];
    }
    if (ymax < ymin) {
        flippedY = true;
        [ymax, ymin] = [ymin, ymax];
    }

    return { flippedX, flippedY, xmin, xmax, ymin, ymax };
}

export function getGraphCornerWithBuffer(
    board: JXGObject,
    direction: [number, number],
    buffer = 0.01,
): [number, number] {
    let { flippedX, flippedY, xmin, xmax, ymin, ymax } =
        getEffectiveBoundingBox(board);

    let xSign = flippedX ? -1 : 1;
    let ySign = flippedY ? -1 : 1;

    let xscale = xmax - xmin;
    let yscale = ymax - ymin;

    let xminAdjusted = xmin + xscale * buffer;
    let xmaxAdjusted = xmax - xscale * buffer;
    let yminAdjusted = ymin + yscale * buffer;
    let ymaxAdjusted = ymax - yscale * buffer;

    let x = direction[0] * xSign === 1 ? xmaxAdjusted : xminAdjusted;
    let y = direction[1] * ySign === 1 ? ymaxAdjusted : yminAdjusted;

    return [x, y];
}

export function adjustPointLabelPosition(
    labelPosition: string,
    nearEdgeOfGraph: [number, number],
): LabelPosition {
    if (nearEdgeOfGraph[0] === -1) {
        if (
            labelPosition.substring(
                labelPosition.length - 4,
                labelPosition.length,
            ) === "left"
        ) {
            labelPosition =
                labelPosition.substring(0, labelPosition.length - 4) + "right";
        } else if (labelPosition === "top") {
            labelPosition = "upperright";
        } else if (labelPosition === "bottom") {
            labelPosition = "lowerright";
        }
    } else if (nearEdgeOfGraph[0] === 1) {
        if (
            labelPosition.substring(
                labelPosition.length - 5,
                labelPosition.length,
            ) === "right"
        ) {
            labelPosition =
                labelPosition.substring(0, labelPosition.length - 5) + "left";
        } else if (labelPosition === "top") {
            labelPosition = "upperleft";
        } else if (labelPosition === "bottom") {
            labelPosition = "lowerleft";
        }
    }

    if (nearEdgeOfGraph[1] === -1) {
        if (labelPosition.substring(0, 5) === "lower") {
            labelPosition =
                "upper" + labelPosition.substring(5, labelPosition.length);
        } else if (labelPosition === "left") {
            labelPosition = "upperleft";
        } else if (labelPosition === "right") {
            labelPosition = "upperright";
        }
    } else if (nearEdgeOfGraph[1] === 1) {
        if (labelPosition.substring(0, 5) === "upper") {
            labelPosition =
                "lower" + labelPosition.substring(5, labelPosition.length);
        } else if (labelPosition === "left") {
            labelPosition = "lowerleft";
        } else if (labelPosition === "right") {
            labelPosition = "lowerright";
        }
    }

    return labelPosition as LabelPosition;
}

export function calculatePointLabelAnchor(labelPosition: LabelPosition) {
    let anchorx, anchory, offset;
    if (labelPosition === "upperright") {
        offset = [5, 5];
        anchorx = "left";
        anchory = "bottom";
    } else if (labelPosition === "upperleft") {
        offset = [-5, 5];
        anchorx = "right";
        anchory = "bottom";
    } else if (labelPosition === "lowerright") {
        offset = [5, -5];
        anchorx = "left";
        anchory = "top";
    } else if (labelPosition === "lowerleft") {
        offset = [-5, -5];
        anchorx = "right";
        anchory = "top";
    } else if (labelPosition === "top") {
        offset = [0, 10];
        anchorx = "middle";
        anchory = "bottom";
    } else if (labelPosition === "bottom") {
        offset = [0, -10];
        anchorx = "middle";
        anchory = "top";
    } else if (labelPosition === "left") {
        offset = [-10, 0];
        anchorx = "right";
        anchory = "middle";
    } else {
        // labelPosition === right
        offset = [10, 0];
        anchorx = "left";
        anchory = "middle";
    }
    return { offset, anchorx, anchory };
}

export function normalizePointSize(
    size: number,
    style: "diamond" | "plus" | "square" | "triangle" | string,
) {
    if (style === "diamond") {
        return size * 1.4;
    } else if (style === "plus") {
        return size * 1.2;
    } else if (style === "square") {
        return size * 1.1;
    } else if (style.substring(0, 8) === "triangle") {
        return size * 1.5;
    } else return size;
}

export function normalizePointStyle(
    style:
        | "triangledown"
        | "triangleup"
        | "triangleleft"
        | "triangleright"
        | string,
    offGraphIndicatorSides: [number, number],
) {
    if (offGraphIndicatorSides[1] === -1) {
        return "triangledown";
    } else if (offGraphIndicatorSides[1] === 1) {
        return "triangleup";
    } else if (offGraphIndicatorSides[0] === -1) {
        return "triangleleft";
    } else if (offGraphIndicatorSides[0] === 1) {
        return "triangleright";
    } else if (style === "triangle") {
        return "triangleup";
    } else {
        return style;
    }
}

export function getPositionFromAnchorByCoordinate(
    positionFromAnchor: LabelPosition,
) {
    let anchorx, anchory;
    if (positionFromAnchor === "center") {
        anchorx = "middle";
        anchory = "middle";
    } else if (positionFromAnchor === "lowerleft") {
        anchorx = "right";
        anchory = "top";
    } else if (positionFromAnchor === "lowerright") {
        anchorx = "left";
        anchory = "top";
    } else if (positionFromAnchor === "upperleft") {
        anchorx = "right";
        anchory = "bottom";
    } else if (positionFromAnchor === "upperright") {
        anchorx = "left";
        anchory = "bottom";
    } else if (positionFromAnchor === "bottom") {
        anchorx = "middle";
        anchory = "top";
    } else if (positionFromAnchor === "top") {
        anchorx = "middle";
        anchory = "bottom";
    } else if (positionFromAnchor === "right") {
        anchorx = "left";
        anchory = "middle";
    } else {
        // positionFromAnchor === left
        anchorx = "right";
        anchory = "middle";
    }
    return { anchorx, anchory };
}
