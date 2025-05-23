import CompositeComponent from "./abstract/CompositeComponent";
import { postProcessCopy } from "../utils/copy";
import me from "math-expressions";
import { processAssignNames } from "../utils/naming";
import { returnGroupIntoComponentTypeSeparatedBySpacesOutsideParens } from "./commonsugar/lists";

export default class Sort extends CompositeComponent {
    static componentType = "sort";

    static allowInSchemaAsComponent = ["_inline", "_block", "_graphical"];

    static stateVariableToEvaluateAfterReplacements =
        "readyToExpandWhenResolved";
    static assignNamesToReplacements = true;

    static createAttributesObject() {
        let attributes = super.createAttributesObject();

        attributes.assignNamesSkip = {
            createPrimitiveOfType: "number",
        };

        attributes.sortVectorsBy = {
            createComponentOfType: "text",
            createStateVariable: "sortVectorsBy",
            defaultValue: "displacement",
            public: true,
            toLowerCase: true,
            validValues: ["displacement", "tail"],
        };

        attributes.sortByComponent = {
            createComponentOfType: "integer",
            createStateVariable: "sortByComponent",
            defaultValue: "1",
            public: true,
        };

        attributes.sortByProp = {
            createPrimitiveOfType: "string",
        };

        attributes.type = {
            createPrimitiveOfType: "string",
        };

        attributes.asList = {
            createPrimitiveOfType: "boolean",
            createStateVariable: "asList",
            defaultValue: true,
        };

        return attributes;
    }

    static returnSugarInstructions() {
        let sugarInstructions = super.returnSugarInstructions();

        function breakStringsMacrosIntoTypeBySpaces({
            matchedChildren,
            componentAttributes,
            componentInfoObjects,
        }) {
            // only if all children are strings or macros
            if (
                !matchedChildren.every(
                    (child) =>
                        typeof child === "string" ||
                        child.doenetAttributes?.createdFromMacro,
                )
            ) {
                return { success: false };
            }

            let type;
            if (componentAttributes.type) {
                type = componentAttributes.type;
            } else {
                return { success: false };
            }

            if (!["math", "text", "number", "boolean"].includes(type)) {
                console.warn(`Invalid type ${type}`);
                return { success: false };
            }

            // break any string by white space and wrap pieces with type
            let groupIntoComponentTypesSeparatedBySpaces =
                returnGroupIntoComponentTypeSeparatedBySpacesOutsideParens({
                    componentType: type,
                    forceComponentType: true,
                });
            let result = groupIntoComponentTypesSeparatedBySpaces({
                matchedChildren,
                componentInfoObjects,
            });

            if (result.success) {
                let newChildren = result.newChildren;

                let newAttributes = {
                    addLevelToAssignNames: {
                        primitive: true,
                    },
                };

                return {
                    success: true,
                    newChildren,
                    newAttributes,
                };
            } else {
                return { success: false };
            }
        }

        sugarInstructions.push({
            replacementFunction: breakStringsMacrosIntoTypeBySpaces,
        });

        return sugarInstructions;
    }

    static returnChildGroups() {
        return [
            {
                group: "anything",
                componentTypes: ["_base"],
            },
        ];
    }

    static returnStateVariableDefinitions() {
        let stateVariableDefinitions = super.returnStateVariableDefinitions();

        stateVariableDefinitions.propName = {
            returnDependencies: () => ({
                propName: {
                    dependencyType: "attributePrimitive",
                    attributeName: "sortByProp",
                },
            }),
            definition: function ({ dependencyValues }) {
                return { setValue: { propName: dependencyValues.propName } };
            },
        };

        stateVariableDefinitions.componentIndicesForValues = {
            returnDependencies: () => ({
                children: {
                    dependencyType: "child",
                    childGroups: ["anything"],
                    variableNames: ["componentIndicesInList"],
                    variablesOptional: true,
                },
            }),
            definition({ dependencyValues }) {
                let componentIndicesForValues = [];
                for (let child of dependencyValues.children) {
                    if (child.stateValues.componentIndicesInList) {
                        componentIndicesForValues.push(
                            ...child.stateValues.componentIndicesInList,
                        );
                    } else {
                        componentIndicesForValues.push(child.componentIdx);
                    }
                }

                return { setValue: { componentIndicesForValues } };
            },
        };

        stateVariableDefinitions.sortedValues = {
            stateVariablesDeterminingDependencies: [
                "componentIndicesForValues",
                "sortByComponent",
                "propName",
            ],
            returnDependencies({ stateValues }) {
                let dependencies = {
                    sortVectorsBy: {
                        dependencyType: "stateVariable",
                        variableName: "sortVectorsBy",
                    },
                    sortByComponent: {
                        dependencyType: "stateVariable",
                        variableName: "sortByComponent",
                    },
                    propName: {
                        dependencyType: "stateVariable",
                        variableName: "propName",
                    },
                };
                if (stateValues.propName) {
                    for (let [
                        ind,
                        cIdx,
                    ] of stateValues.componentIndicesForValues.entries()) {
                        dependencies[`component${ind}`] = {
                            dependencyType: "stateVariable",
                            componentIdx: cIdx,
                            variableName: stateValues.propName,
                            variablesOptional: true,
                            caseInsensitiveVariableMatch: true,
                            publicStateVariablesOnly: true,
                            returnAsComponentObject: true,
                        };
                    }
                } else {
                    for (let [
                        ind,
                        cIdx,
                    ] of stateValues.componentIndicesForValues.entries()) {
                        dependencies[`component${ind}`] = {
                            dependencyType: "multipleStateVariables",
                            componentIdx: cIdx,
                            variableNames: [
                                "value",
                                `x${stateValues.sortByComponent}`,
                                `tailX${stateValues.sortByComponent}`,
                            ],
                            variablesOptional: true,
                        };
                    }
                }
                return dependencies;
            },
            definition({ dependencyValues, componentInfoObjects }) {
                let allValues = [];
                let allAreNumeric = true;
                for (let depName in dependencyValues) {
                    if (depName.substring(0, 9) !== "component") {
                        continue;
                    }
                    let component = dependencyValues[depName];
                    if (dependencyValues.propName) {
                        let value = Object.values(component.stateValues)[0];
                        allValues.push({
                            componentIdx: component.componentIdx,
                            numericalValue: Number(value),
                            textValue: String(value),
                        });
                        if (!Number.isFinite(value)) {
                            allAreNumeric = false;
                        }
                    } else if (
                        componentInfoObjects.isInheritedComponentType({
                            inheritedComponentType: component.componentType,
                            baseComponentType: "number",
                        })
                    ) {
                        allValues.push({
                            componentIdx: component.componentIdx,
                            numericalValue: component.stateValues.value,
                            textValue: String(component.stateValues.value),
                        });
                    } else if (
                        componentInfoObjects.isInheritedComponentType({
                            inheritedComponentType: component.componentType,
                            baseComponentType: "text",
                        })
                    ) {
                        let numericalValue = NaN;
                        let textValue = component.stateValues.value;
                        allValues.push({
                            componentIdx: component.componentIdx,
                            numericalValue,
                            textValue,
                        });
                        allAreNumeric = false;
                    } else if (
                        componentInfoObjects.isInheritedComponentType({
                            inheritedComponentType: component.componentType,
                            baseComponentType: "math",
                        })
                    ) {
                        let numericalValue =
                            component.stateValues.value.evaluate_to_constant();
                        if (Number.isNaN(numericalValue)) {
                            allAreNumeric = false;
                        }
                        allValues.push({
                            componentIdx: component.componentIdx,
                            numericalValue,
                            textValue: component.stateValues.value.toString(),
                        });
                    } else if (
                        componentInfoObjects.isInheritedComponentType({
                            inheritedComponentType: component.componentType,
                            baseComponentType: "point",
                        })
                    ) {
                        let compValue =
                            component.stateValues[
                                `x${dependencyValues.sortByComponent}`
                            ];
                        let numericalValue = NaN;
                        let textValue = "";
                        if (compValue) {
                            numericalValue = compValue.evaluate_to_constant();
                            if (Number.isNaN(numericalValue)) {
                                allAreNumeric = false;
                            }
                            textValue = compValue.toString();
                        }
                        allValues.push({
                            componentIdx: component.componentIdx,
                            numericalValue,
                            textValue,
                        });
                    } else if (
                        componentInfoObjects.isInheritedComponentType({
                            inheritedComponentType: component.componentType,
                            baseComponentType: "vector",
                        })
                    ) {
                        let numericalValue = NaN;
                        let textValue = "";
                        let compValue =
                            component.stateValues[
                                `x${dependencyValues.sortByComponent}`
                            ];
                        if (dependencyValues.sortVectorsBy === "displacement") {
                            compValue =
                                component.stateValues[
                                    `x${dependencyValues.sortByComponent}`
                                ];
                        } else {
                            compValue =
                                component.stateValues[
                                    `tailX${dependencyValues.sortByComponent}`
                                ];
                        }
                        if (compValue) {
                            numericalValue = compValue.evaluate_to_constant();
                            if (Number.isNaN(numericalValue)) {
                                allAreNumeric = false;
                            }
                            textValue = compValue.toString();
                        }
                        allValues.push({
                            componentIdx: component.componentIdx,
                            numericalValue,
                            textValue,
                        });
                    }
                }

                if (allAreNumeric) {
                    allValues.sort(
                        (a, b) => a.numericalValue - b.numericalValue,
                    );
                } else {
                    allValues.sort((a, b) =>
                        a.textValue > b.textValue
                            ? 1
                            : a.textValue < b.textValue
                              ? -1
                              : 0,
                    );
                }

                return {
                    setValue: {
                        sortedValues: allValues,
                    },
                };
            },
        };

        stateVariableDefinitions.readyToExpandWhenResolved = {
            returnDependencies: () => ({
                sortedValues: {
                    dependencyType: "stateVariable",
                    variableName: "sortedValues",
                },
            }),
            markStale: () => ({ updateReplacements: true }),
            definition: function () {
                return { setValue: { readyToExpandWhenResolved: true } };
            },
        };

        return stateVariableDefinitions;
    }

    static async createSerializedReplacements({
        component,
        components,
        componentInfoObjects,
        workspace,
    }) {
        let errors = [];
        let warnings = [];

        let replacements = [];

        let componentsCopied = [];

        for (let valueObj of await component.stateValues.sortedValues) {
            let replacementSource;

            if (valueObj.listInd === undefined) {
                replacementSource = components[valueObj.componentIdx];
            } else {
                let listComponent = components[valueObj.componentIdx];
                replacementSource =
                    listComponent.activeChildren[valueObj.listInd];
            }

            if (replacementSource) {
                componentsCopied.push(replacementSource.componentIdx);

                replacements.push(
                    await replacementSource.serialize({
                        primitiveSourceAttributesToIgnore: ["isResponse"],
                    }),
                );
            }
        }

        workspace.uniqueIdentifiersUsed = [];
        replacements = postProcessCopy({
            serializedComponents: replacements,
            componentIdx: component.componentIdx,
            uniqueIdentifiersUsed: workspace.uniqueIdentifiersUsed,
            addShadowDependencies: true,
            markAsPrimaryShadow: true,
        });

        let processResult = processAssignNames({
            assignNames: component.doenetAttributes.assignNames,
            serializedComponents: replacements,
            parentIdx: component.componentIdx,
            parentCreatesNewNamespace: await component.stateValues.newNamespace,
            componentInfoObjects,
        });
        errors.push(...processResult.errors);
        warnings.push(...processResult.warnings);

        workspace.componentsCopied = componentsCopied;

        return {
            replacements: processResult.serializedComponents,
            errors,
            warnings,
        };
    }

    static async calculateReplacementChanges({
        component,
        components,
        componentInfoObjects,
        workspace,
    }) {
        // TODO: don't yet have a way to return errors and warnings!
        let errors = [];
        let warnings = [];

        let componentsToCopy = [];

        for (let valueObj of await component.stateValues.sortedValues) {
            let replacementSource;

            if (valueObj.listInd === undefined) {
                replacementSource = components[valueObj.componentIdx];
            } else {
                let listComponent = components[valueObj.componentIdx];
                replacementSource =
                    listComponent.activeChildren[valueObj.listInd];
            }

            if (replacementSource) {
                componentsToCopy.push(replacementSource.componentIdx);
            }
        }

        if (
            componentsToCopy.length == workspace.componentsCopied.length &&
            workspace.componentsCopied.every(
                (x, i) => x === componentsToCopy[i],
            )
        ) {
            return [];
        }

        // for now, just recreated
        let replacementResults = await this.createSerializedReplacements({
            component,
            components,
            componentInfoObjects,
            workspace,
        });

        let replacements = replacementResults.replacements;
        errors.push(...replacementResults.errors);
        warnings.push(...replacementResults.warnings);

        let replacementChanges = [
            {
                changeType: "add",
                changeTopLevelReplacements: true,
                firstReplacementInd: 0,
                numberReplacementsToReplace: component.replacements.length,
                serializedReplacements: replacements,
            },
        ];

        return replacementChanges;
    }
}
