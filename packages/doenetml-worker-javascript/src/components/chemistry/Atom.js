import InlineComponent from "../abstract/InlineComponent";
import me from "math-expressions";
import {
    returnSelectedStyleStateVariableDefinition,
    returnTextStyleDescriptionDefinitions,
} from "@doenet/utils";
import {
    returnRoundingAttributeComponentShadowing,
    returnRoundingAttributes,
    returnRoundingStateVariableDefinitions,
} from "../../utils/rounding";
import { getDataForAtom } from "../../utils/chemistry";

export default class Atom extends InlineComponent {
    static componentType = "atom";
    static rendererType = "math";

    static primaryStateVariableForDefinition = "atomicNumberShadow";

    static createAttributesObject() {
        let attributes = super.createAttributesObject();

        attributes.symbol = {
            createComponentOfType: "text",
        };

        attributes.atomicNumber = {
            createComponentOfType: "integer",
        };

        Object.assign(attributes, returnRoundingAttributes());

        return attributes;
    }

    static returnStateVariableDefinitions() {
        let stateVariableDefinitions = super.returnStateVariableDefinitions();

        Object.assign(
            stateVariableDefinitions,
            returnRoundingStateVariableDefinitions(),
        );

        let selectedStyleDefinition =
            returnSelectedStyleStateVariableDefinition();
        Object.assign(stateVariableDefinitions, selectedStyleDefinition);

        let styleDescriptionDefinitions =
            returnTextStyleDescriptionDefinitions();
        Object.assign(stateVariableDefinitions, styleDescriptionDefinitions);

        // atomicNumberShadow will be null unless atom was created
        // via an adapter or copy prop or from serialized state with coords value
        // In case of adapter or copy prop,
        // given the primaryStateVariableForDefinition static variable,
        // the definition of atomicNumberShadow will be changed to be the value
        // that shadows the component adapted or copied
        stateVariableDefinitions.atomicNumberShadow = {
            defaultValue: null,
            hasEssential: true,
            essentialVarName: "atomicNumber",
            returnDependencies: () => ({}),
            definition: () => ({
                useEssentialOrDefaultValue: {
                    atomicNumberShadow: true,
                },
            }),
            inverseDefinition: async function ({ desiredStateVariableValues }) {
                return {
                    success: true,
                    instructions: [
                        {
                            setEssentialValue: "atomicNumberShadow",
                            value: desiredStateVariableValues.atomicNumberShadow,
                        },
                    ],
                };
            },
        };

        stateVariableDefinitions.dataForAtom = {
            returnDependencies: () => ({
                symbolAttr: {
                    dependencyType: "attributeComponent",
                    attributeName: "symbol",
                    variableNames: ["value"],
                },
                atomicNumberAttr: {
                    dependencyType: "attributeComponent",
                    attributeName: "atomicNumber",
                    variableNames: ["value"],
                },
                atomicNumberShadow: {
                    dependencyType: "stateVariable",
                    variableName: "atomicNumberShadow",
                },
            }),

            definition: function ({ dependencyValues }) {
                let symbol = null,
                    atomicNumber = null;
                if (dependencyValues.symbolAttr) {
                    symbol =
                        dependencyValues.symbolAttr.stateValues.value.toLowerCase();
                } else if (dependencyValues.atomicNumberAttr) {
                    atomicNumber =
                        dependencyValues.atomicNumberAttr.stateValues.value;
                } else if (dependencyValues.atomicNumberShadow) {
                    atomicNumber = dependencyValues.atomicNumberShadow;
                } else {
                    return { setValue: { dataForAtom: null } };
                }

                let dataForAtom = getDataForAtom({ symbol, atomicNumber });
                return { setValue: { dataForAtom } };
            },
        };

        stateVariableDefinitions.atomicNumber = {
            public: true,
            shadowingInstructions: {
                createComponentOfType: "integer",
            },
            returnDependencies: () => ({
                dataForAtom: {
                    dependencyType: "stateVariable",
                    variableName: "dataForAtom",
                },
            }),
            definition({ dependencyValues }) {
                let atomicNumber;

                if (dependencyValues.dataForAtom) {
                    atomicNumber =
                        dependencyValues.dataForAtom["Atomic Number"];
                } else {
                    atomicNumber = null;
                }

                return { setValue: { atomicNumber } };
            },
        };

        stateVariableDefinitions.symbol = {
            public: true,
            shadowingInstructions: {
                createComponentOfType: "text",
            },
            returnDependencies: () => ({
                dataForAtom: {
                    dependencyType: "stateVariable",
                    variableName: "dataForAtom",
                },
            }),
            definition({ dependencyValues }) {
                let symbol;

                if (dependencyValues.dataForAtom) {
                    symbol = dependencyValues.dataForAtom.Symbol;
                } else {
                    symbol = null;
                }

                return { setValue: { symbol } };
            },
        };

        stateVariableDefinitions.name = {
            public: true,
            shadowingInstructions: {
                createComponentOfType: "text",
            },
            returnDependencies: () => ({
                dataForAtom: {
                    dependencyType: "stateVariable",
                    variableName: "dataForAtom",
                },
            }),
            definition({ dependencyValues }) {
                let name;

                if (dependencyValues.dataForAtom) {
                    name = dependencyValues.dataForAtom.Name;
                } else {
                    name = null;
                }

                return { setValue: { name } };
            },
        };

        stateVariableDefinitions.group = {
            public: true,
            shadowingInstructions: {
                createComponentOfType: "integer",
            },
            returnDependencies: () => ({
                dataForAtom: {
                    dependencyType: "stateVariable",
                    variableName: "dataForAtom",
                },
            }),
            definition({ dependencyValues }) {
                let group;

                if (dependencyValues.dataForAtom) {
                    group = dependencyValues.dataForAtom.Group;
                } else {
                    group = null;
                }

                return { setValue: { group } };
            },
        };

        stateVariableDefinitions.atomicMass = {
            public: true,
            shadowingInstructions: {
                createComponentOfType: "number",
                addAttributeComponentsShadowingStateVariables:
                    returnRoundingAttributeComponentShadowing(),
            },
            returnDependencies: () => ({
                dataForAtom: {
                    dependencyType: "stateVariable",
                    variableName: "dataForAtom",
                },
            }),
            definition({ dependencyValues }) {
                let atomicMass;

                if (dependencyValues.dataForAtom) {
                    atomicMass = dependencyValues.dataForAtom["Atomic Mass"];
                } else {
                    atomicMass = null;
                }

                return { setValue: { atomicMass } };
            },
        };

        stateVariableDefinitions.phaseAtSTP = {
            public: true,
            shadowingInstructions: {
                createComponentOfType: "text",
            },
            returnDependencies: () => ({
                dataForAtom: {
                    dependencyType: "stateVariable",
                    variableName: "dataForAtom",
                },
            }),
            definition({ dependencyValues }) {
                let phaseAtSTP;

                if (dependencyValues.dataForAtom) {
                    phaseAtSTP = dependencyValues.dataForAtom["Phase at STP"];
                } else {
                    phaseAtSTP = null;
                }

                return { setValue: { phaseAtSTP } };
            },
        };

        stateVariableDefinitions.chargeOfCommonIon = {
            public: true,
            shadowingInstructions: {
                createComponentOfType: "integer",
            },
            returnDependencies: () => ({
                dataForAtom: {
                    dependencyType: "stateVariable",
                    variableName: "dataForAtom",
                },
            }),
            definition({ dependencyValues }) {
                let chargeOfCommonIon;

                if (dependencyValues.dataForAtom) {
                    chargeOfCommonIon =
                        dependencyValues.dataForAtom["Charge of Common Ion"];
                } else {
                    chargeOfCommonIon = null;
                }

                return { setValue: { chargeOfCommonIon } };
            },
        };

        stateVariableDefinitions.metalCategory = {
            public: true,
            shadowingInstructions: {
                createComponentOfType: "text",
            },
            returnDependencies: () => ({
                dataForAtom: {
                    dependencyType: "stateVariable",
                    variableName: "dataForAtom",
                },
            }),
            definition({ dependencyValues }) {
                let metalCategory;

                if (dependencyValues.dataForAtom) {
                    metalCategory =
                        dependencyValues.dataForAtom[
                            "Metal/Nonmetal/Metalloid"
                        ];
                } else {
                    metalCategory = null;
                }

                return { setValue: { metalCategory } };
            },
        };

        stateVariableDefinitions.groupName = {
            public: true,
            shadowingInstructions: {
                createComponentOfType: "text",
            },
            returnDependencies: () => ({
                dataForAtom: {
                    dependencyType: "stateVariable",
                    variableName: "dataForAtom",
                },
            }),
            definition({ dependencyValues }) {
                let groupName;

                if (dependencyValues.dataForAtom) {
                    groupName = dependencyValues.dataForAtom["Group Name"];
                } else {
                    groupName = null;
                }

                return { setValue: { groupName } };
            },
        };

        stateVariableDefinitions.period = {
            public: true,
            shadowingInstructions: {
                createComponentOfType: "integer",
            },
            returnDependencies: () => ({
                dataForAtom: {
                    dependencyType: "stateVariable",
                    variableName: "dataForAtom",
                },
            }),
            definition({ dependencyValues }) {
                let period;

                if (dependencyValues.dataForAtom) {
                    period = dependencyValues.dataForAtom.Period;
                } else {
                    period = null;
                }

                return { setValue: { period } };
            },
        };

        stateVariableDefinitions.ionizationEnergy = {
            public: true,
            shadowingInstructions: {
                createComponentOfType: "number",
                addAttributeComponentsShadowingStateVariables:
                    returnRoundingAttributeComponentShadowing(),
            },
            returnDependencies: () => ({
                dataForAtom: {
                    dependencyType: "stateVariable",
                    variableName: "dataForAtom",
                },
            }),
            definition({ dependencyValues }) {
                let ionizationEnergy;

                if (dependencyValues.dataForAtom) {
                    ionizationEnergy =
                        dependencyValues.dataForAtom["Ionization Energy"];
                } else {
                    ionizationEnergy = null;
                }

                return { setValue: { ionizationEnergy } };
            },
        };

        stateVariableDefinitions.meltingPoint = {
            public: true,
            shadowingInstructions: {
                createComponentOfType: "number",
                addAttributeComponentsShadowingStateVariables:
                    returnRoundingAttributeComponentShadowing(),
            },
            returnDependencies: () => ({
                dataForAtom: {
                    dependencyType: "stateVariable",
                    variableName: "dataForAtom",
                },
            }),
            definition({ dependencyValues }) {
                let meltingPoint;

                if (dependencyValues.dataForAtom) {
                    meltingPoint =
                        dependencyValues.dataForAtom["Melting Point"];
                } else {
                    meltingPoint = null;
                }

                return { setValue: { meltingPoint } };
            },
        };

        stateVariableDefinitions.boilingPoint = {
            public: true,
            shadowingInstructions: {
                createComponentOfType: "number",
                addAttributeComponentsShadowingStateVariables:
                    returnRoundingAttributeComponentShadowing(),
            },
            returnDependencies: () => ({
                dataForAtom: {
                    dependencyType: "stateVariable",
                    variableName: "dataForAtom",
                },
            }),
            definition({ dependencyValues }) {
                let boilingPoint;

                if (dependencyValues.dataForAtom) {
                    boilingPoint =
                        dependencyValues.dataForAtom["Boiling Point"];
                } else {
                    boilingPoint = null;
                }

                return { setValue: { boilingPoint } };
            },
        };

        stateVariableDefinitions.atomicRadius = {
            public: true,
            shadowingInstructions: {
                createComponentOfType: "integer",
            },
            returnDependencies: () => ({
                dataForAtom: {
                    dependencyType: "stateVariable",
                    variableName: "dataForAtom",
                },
            }),
            definition({ dependencyValues }) {
                let atomicRadius;

                if (dependencyValues.dataForAtom) {
                    atomicRadius =
                        dependencyValues.dataForAtom["Atomic Radius"];
                } else {
                    atomicRadius = null;
                }

                return { setValue: { atomicRadius } };
            },
        };

        stateVariableDefinitions.density = {
            public: true,
            shadowingInstructions: {
                createComponentOfType: "number",
                addAttributeComponentsShadowingStateVariables:
                    returnRoundingAttributeComponentShadowing(),
            },
            returnDependencies: () => ({
                dataForAtom: {
                    dependencyType: "stateVariable",
                    variableName: "dataForAtom",
                },
            }),
            definition({ dependencyValues }) {
                let density;

                if (dependencyValues.dataForAtom) {
                    density = dependencyValues.dataForAtom.Density;
                } else {
                    density = null;
                }

                return { setValue: { density } };
            },
        };

        stateVariableDefinitions.electronegativity = {
            public: true,
            shadowingInstructions: {
                createComponentOfType: "number",
                addAttributeComponentsShadowingStateVariables:
                    returnRoundingAttributeComponentShadowing(),
            },
            returnDependencies: () => ({
                dataForAtom: {
                    dependencyType: "stateVariable",
                    variableName: "dataForAtom",
                },
            }),
            definition({ dependencyValues }) {
                let electronegativity;

                if (dependencyValues.dataForAtom) {
                    electronegativity =
                        dependencyValues.dataForAtom.Electronegativity;
                } else {
                    electronegativity = null;
                }

                return { setValue: { electronegativity } };
            },
        };

        stateVariableDefinitions.electronConfiguration = {
            public: true,
            shadowingInstructions: {
                createComponentOfType: "electronConfiguration",
            },
            returnDependencies: () => ({
                dataForAtom: {
                    dependencyType: "stateVariable",
                    variableName: "dataForAtom",
                },
            }),
            definition({ dependencyValues }) {
                let electronConfiguration;

                if (dependencyValues.dataForAtom) {
                    electronConfiguration =
                        dependencyValues.dataForAtom["Electron Configuration"];
                } else {
                    electronConfiguration = null;
                }

                return { setValue: { electronConfiguration } };
            },
        };

        stateVariableDefinitions.orbitalDiagram = {
            public: true,
            shadowingInstructions: {
                createComponentOfType: "orbitalDiagram",
            },
            returnDependencies: () => ({
                electronConfiguration: {
                    dependencyType: "stateVariable",
                    variableName: "electronConfiguration",
                },
            }),
            definition({ dependencyValues }) {
                let orbitalDiagram;
                if (dependencyValues.electronConfiguration) {
                    orbitalDiagram = electronConfigurationToOrbitalDiagram(
                        dependencyValues.electronConfiguration,
                    );
                } else {
                    orbitalDiagram = null;
                }
                return {
                    setValue: { orbitalDiagram },
                };
            },
        };

        stateVariableDefinitions.math = {
            public: true,
            shadowingInstructions: {
                createComponentOfType: "math",
            },
            returnDependencies: () => ({
                symbol: {
                    dependencyType: "stateVariable",
                    variableName: "symbol",
                },
            }),
            definition({ dependencyValues }) {
                let tree;

                if (dependencyValues.symbol) {
                    tree = dependencyValues.symbol;
                } else {
                    tree = "\uff3f";
                }
                let math = me.fromAst(tree);
                return {
                    setValue: { math },
                };
            },
        };

        stateVariableDefinitions.latex = {
            public: true,
            forRenderer: true,
            shadowingInstructions: {
                createComponentOfType: "latex",
            },
            returnDependencies: () => ({
                symbol: {
                    dependencyType: "stateVariable",
                    variableName: "symbol",
                },
            }),
            definition({ dependencyValues }) {
                let latex;
                if (dependencyValues.symbol) {
                    latex = `\\text{${dependencyValues.symbol}}`;
                } else {
                    latex = "[\\text{Invalid Chemical Symbol}]";
                }
                return {
                    setValue: { latex },
                };
            },
        };

        stateVariableDefinitions.text = {
            public: true,
            shadowingInstructions: {
                createComponentOfType: "text",
            },
            returnDependencies: () => ({
                symbol: {
                    dependencyType: "stateVariable",
                    variableName: "symbol",
                },
            }),
            definition({ dependencyValues }) {
                let text;
                if (dependencyValues.symbol) {
                    text = dependencyValues.symbol;
                } else {
                    text = "[Invalid Chemical Symbol]";
                }
                return {
                    setValue: { text },
                };
            },
        };

        return stateVariableDefinitions;
    }

    static adapters = [
        "math",
        "name",
        {
            stateVariable: "atomicNumber",
            componentType: "ion",
        },
    ];
}

function electronConfigurationToOrbitalDiagram(electronConfiguration) {
    let electronConfig = electronConfiguration.tree;

    if (!(Array.isArray(electronConfig) && electronConfig[0] === "*")) {
        return null;
    }

    electronConfig = electronConfig.slice(1);

    let numRows = electronConfig.length / 2;
    if (!Number.isInteger(numRows)) {
        return null;
    }

    let orbitalDiagram = [];

    for (let rowInd = 0; rowInd < numRows; rowInd++) {
        let electronLevel = electronConfig[2 * rowInd];
        if (!(Number.isInteger(electronLevel) && electronLevel > 0)) {
            return null;
        }

        let infoObj = electronConfig[2 * rowInd + 1];
        if (!(Array.isArray(infoObj) && infoObj[0] === "^")) {
            return null;
        }

        let shellType = infoObj[1];
        let nElectrons = infoObj[2];

        if (
            !(
                ["s", "p", "d", "f"].includes(shellType) &&
                Number.isInteger(nElectrons) &&
                nElectrons > 0
            )
        ) {
            return null;
        }

        let orbitalText = `${electronLevel}${shellType}`;

        let nBoxes;
        if (shellType === "s") {
            nBoxes = 1;
        } else if (shellType === "p") {
            nBoxes = 3;
        } else if (shellType === "d") {
            nBoxes = 5;
        } else {
            nBoxes = 7;
        }

        let boxes = Array(nBoxes).fill("");

        for (
            let electronInd = 0;
            electronInd < Math.min(nBoxes, nElectrons);
            electronInd++
        ) {
            if (nElectrons <= electronInd + nBoxes) {
                boxes[electronInd] = "U";
            } else {
                boxes[electronInd] = "UD";
            }
        }

        orbitalDiagram.push({ orbitalText, boxes });
    }

    return orbitalDiagram;
}
