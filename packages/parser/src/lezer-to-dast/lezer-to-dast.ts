import { SyntaxNode, TreeBuffer } from "@lezer/common";
import {
    CloseTag,
    OpenTag,
    TagName,
} from "../generated-assets/lezer-doenet.terms";
import { parser } from "../generated-assets/lezer-doenet";
import {
    DastAttribute,
    DastDoctype,
    DastElementContent,
    DastRoot,
    DastRootContent,
    LezerSyntaxNodeName,
} from "../types";
import { createErrorNode } from "./create-error-node";
import {
    attributeValueText,
    createOffsetToPositionMap,
    entityToString,
    extractContent,
    extractDoctypeInfo,
    findFirstErrorInChild,
    getLezerChildren,
    lezerNodeToPosition,
    textNodeToText,
} from "./lezer-to-dast-utils";

/**
 * Convert a lezer `SyntaxNode` into a DAST tree.
 */
export function lezerToDast(
    node: SyntaxNode | string,
    source?: string,
): DastRoot {
    if (typeof node === "string") {
        const tree = parser.parse(node);
        source = node;
        return _lezerToDast(tree.topNode, source);
    }
    if (source == null) {
        throw new Error(
            `If you provide a SyntaxNode, you must also provide the source string`,
        );
    }
    return _lezerToDast(node, source);
}

export function _lezerToDast(node: SyntaxNode, source: string): DastRoot {
    if (typeof node === "string") {
        const tree = parser.parse(node);
        source = node;
        node = tree.topNode;
    }
    const offsetMap = createOffsetToPositionMap(source);
    return {
        type: "root",
        children: lezerNodeToDastNode(node),
        position: lezerNodeToPosition(node, offsetMap),
    };

    function lezerNodeToDastNode(node: SyntaxNode): DastRootContent[] {
        if (!node) {
            throw new Error(`Expecting node but got ${JSON.stringify(node)}`);
        }
        const name = node.type.name as LezerSyntaxNodeName;
        switch (name) {
            case "Document":
                return getLezerChildren(node).flatMap(lezerNodeToDastNode);
            case "Element": {
                const openTag =
                    node.getChild("OpenTag") || node.getChild("SelfClosingTag");
                if (!openTag) {
                    console.warn(
                        `Could not find open tag for element ${extractContent(
                            node,
                            source,
                        )}`,
                    );
                    return [];
                }
                const tag = openTag.getChild("TagName");
                const name = tag ? extractContent(tag, source) : "";
                const children: DastElementContent[] = [];
                const attributes: DastAttribute[] = [];
                for (const attrTag of openTag.getChildren("Attribute")) {
                    const error = findFirstErrorInChild(attrTag);
                    if (error) {
                        const errorNode = createErrorNode(
                            error,
                            source,
                            offsetMap,
                        );
                        children.push(errorNode);
                    }
                    const attrName = attrTag.getChild("AttributeName");
                    const attrValue = attrTag.getChild("AttributeValue");
                    if (!attrName) {
                        continue;
                    }
                    const attrChildren: DastAttribute["children"] = attrValue
                        ? [
                              {
                                  type: "text",
                                  value: attributeValueText(attrValue, source),
                              },
                          ]
                        : [];
                    // Attributes with no specified value are assigned the value "true".
                    // E.g. `<foo bar />` is the same as `<foo bar="true" />`
                    attributes.push({
                        type: "attribute",
                        name: extractContent(attrName, source),
                        children: attrChildren,
                        position: lezerNodeToPosition(attrTag, offsetMap),
                    });
                }
                // Children get pushed after attributes so that any attribute errors will
                // appear first.
                children.push(
                    ...getLezerChildren(node).flatMap(
                        (n) => lezerNodeToDastNode(n) as DastElementContent[],
                    ),
                );

                return [
                    {
                        type: "element",
                        name,
                        attributes,
                        children,
                        position: lezerNodeToPosition(node, offsetMap),
                    },
                ];
            }
            case "Text":
                return [
                    {
                        type: "text",
                        value: textNodeToText(node, source),
                        position: lezerNodeToPosition(node, offsetMap),
                    },
                ];
            case "CharacterReference":
            case "EntityReference":
                return [
                    {
                        type: "text",
                        value: entityToString(node, source),
                        position: lezerNodeToPosition(node, offsetMap),
                    },
                ];
            case "ProcessingInst": {
                const fullContent = extractContent(node, source);
                let value = fullContent.slice(2, fullContent.length - 2);
                const match = value.match(/^[\w-]*/);
                const name = match?.[0] || "";
                if (name) {
                    value = value.slice(name.length).trim();
                }
                return [
                    {
                        type: "instruction",
                        name,
                        value,
                        position: lezerNodeToPosition(node, offsetMap),
                    },
                ];
            }
            case "Comment": {
                const fullContent = extractContent(node, source);
                return [
                    {
                        type: "comment",
                        value: fullContent.slice(4, fullContent.length - 3),
                        position: lezerNodeToPosition(node, offsetMap),
                    },
                ];
            }
            case "Cdata": {
                const fullContent = extractContent(node, source);
                return [
                    {
                        type: "cdata",
                        value: fullContent.slice(9, fullContent.length - 3),
                        position: lezerNodeToPosition(node, offsetMap),
                    },
                ];
            }
            case "DoctypeDecl": {
                // DocTypes look like <!DOCTYPE HTML PUBLIC '-//W3C//DTD HTML 4.0 Transitional//EN' 'http://www.w3.org/TR/REC-html40/loose.dtd'>
                const fullContent = extractContent(node, source);
                let value = fullContent.slice(10, fullContent.length - 1);
                const doctypeInfo = extractDoctypeInfo(value);
                return [
                    {
                        type: "doctype",
                        ...doctypeInfo,
                        position: lezerNodeToPosition(node, offsetMap),
                    } as DastDoctype,
                ];
            }
            case "Attribute":
            case "AttributeName":
            case "AttributeValue":
            case "CloseTag":
            case "EndTag":
            case "Is":
            case "OpenTag":
            case "SelfCloseEndTag":
            case "SelfClosingTag":
            case "TagName":
            case "StartTag":
            case "StartCloseTag":
            case "InvalidEntity":
                return [];
            case "MismatchedCloseTag": {
                const parent = node.parent;
                const openTag = parent?.getChild(OpenTag);
                const closeTag = parent?.getChild(CloseTag);
                if (!parent || !openTag) {
                    const message = `Invalid DoenetML: Found closing tag ${extractContent(
                        node,
                        source,
                    )}, but no corresponding opening tag`;
                    return [
                        {
                            type: "error",
                            message,
                            position: lezerNodeToPosition(node, offsetMap),
                        },
                    ];
                }
                // If we have a parent, check to see if we also have a close tag.
                // This could arise in code like `<foo></bar></foo>`
                if (closeTag) {
                    const message = `Invalid DoenetML: Found closing tag ${extractContent(
                        node,
                        source,
                    )}, but no corresponding opening tag`;

                    return [
                        {
                            type: "error",
                            message,
                            position: lezerNodeToPosition(node, offsetMap),
                        },
                    ];
                }
                // In this case, there was an open tag, a mismatched close tag, but no actual close tag.
                // E.g. `<foo>bar</baz>`
                const tagNameTag = openTag.getChild(TagName);
                const openTagName = tagNameTag
                    ? extractContent(tagNameTag, source)
                    : "";
                const message = `Invalid DoenetML: Mismatched closing tag. Expected </${openTagName}>. Found ${extractContent(
                    node,
                    source,
                )}`;

                return [
                    {
                        type: "error",
                        message,
                        position: lezerNodeToPosition(node, offsetMap),
                    },
                ];
            }
            case "MissingCloseTag":
            case "⚠":
                return [];
            default:
                const unhandledName: never = name;
                console.log(
                    `Encountered Lezer node of unknown type ${unhandledName}`,
                );
        }
        return [];
    }
}