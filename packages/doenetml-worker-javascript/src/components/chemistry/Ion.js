import InlineComponent from "../abstract/InlineComponent";
import me from "math-expressions";
import {
    returnSelectedStyleStateVariableDefinition,
    returnTextStyleDescriptionDefinitions,
} from "@doenet/utils";
import { getDataForAtom } from "../../utils/chemistry";

export default class Ion extends InlineComponent {
    static componentType = "ion";
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

        attributes.charge = {
            createComponentOfType: "integer",
        };

        return attributes;
    }

    static returnChildGroups() {
        return [
            {
                group: "atoms",
                componentTypes: ["atom"],
            },
        ];
    }

    static returnStateVariableDefinitions() {
        let stateVariableDefinitions = super.returnStateVariableDefinitions();

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
                atomChild: {
                    dependencyType: "child",
                    childGroups: ["atoms"],
                    variableNames: ["atomicNumber"],
                },
                atomicNumberShadow: {
                    dependencyType: "stateVariable",
                    variableName: "atomicNumberShadow",
                },
            }),
            definition: function ({ dependencyValues }) {
                let symbol = null,
                    atomicNumber = null;
                if (dependencyValues.atomChild.length > 0) {
                    atomicNumber =
                        dependencyValues.atomChild[0].stateValues.atomicNumber;
                } else if (dependencyValues.symbolAttr) {
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

        stateVariableDefinitions.charge = {
            public: true,
            shadowingInstructions: {
                createComponentOfType: "integer",
            },
            returnDependencies: () => ({
                dataForAtom: {
                    dependencyType: "stateVariable",
                    variableName: "dataForAtom",
                },
                chargeAttr: {
                    dependencyType: "attributeComponent",
                    attributeName: "charge",
                    variableNames: ["value"],
                },
            }),
            definition({ dependencyValues }) {
                let charge;

                if (dependencyValues.chargeAttr) {
                    charge = dependencyValues.chargeAttr.stateValues.value;
                    if (!Number.isFinite(charge)) {
                        charge = 0;
                    }
                } else if (dependencyValues.dataForAtom) {
                    charge =
                        dependencyValues.dataForAtom["Charge of Common Ion"] ||
                        0;
                } else {
                    charge = 0;
                }

                return { setValue: { charge } };
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
                charge: {
                    dependencyType: "stateVariable",
                    variableName: "charge",
                },
                group: {
                    dependencyType: "stateVariable",
                    variableName: "group",
                },
                atomicNumber: {
                    dependencyType: "stateVariable",
                    variableName: "atomicNumber",
                },
            }),
            definition({ dependencyValues }) {
                let name;

                if (dependencyValues.dataForAtom) {
                    name = dependencyValues.dataForAtom.Name;
                    if (dependencyValues.charge < 0) {
                        let mapping = {
                            Hydrogen: "Hydride",
                            Oxygen: "Oxide",
                            Sulfur: "Sulfide",
                            Nitrogen: "Nitride",
                            Phosphorus: "Phosphide",
                            Carbon: "Carbide",
                        };
                        let len = name.length;
                        if (name.substring(len - 3, len) === "ine") {
                            name = name.substring(0, len - 3) + "ide";
                        } else if (name in mapping) {
                            name = mapping[name];
                        }
                    } else if (
                        dependencyValues.charge > 0 &&
                        ((dependencyValues.group >= 3 &&
                            dependencyValues.group <= 12) ||
                            dependencyValues.group === 101 ||
                            dependencyValues.group === 102 ||
                            [13, 31, 49, 50, 81, 82, 83, 84].includes(
                                dependencyValues.atomicNumber,
                            ))
                    ) {
                        let suffix = "";
                        switch (dependencyValues.charge) {
                            case 1:
                                suffix = " (I)";
                                break;
                            case 2:
                                suffix = " (II)";
                                break;
                            case 3:
                                suffix = " (III)";
                                break;
                            case 4:
                                suffix = " (IV)";
                                break;
                            case 5:
                                suffix = " (V)";
                                break;
                            case 6:
                                suffix = " (VI)";
                                break;
                            case 7:
                                suffix = " (VII)";
                                break;
                            case 8:
                                suffix = " (VIII)";
                                break;
                        }
                        name += suffix;
                    }
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
                charge: {
                    dependencyType: "stateVariable",
                    variableName: "charge",
                },
            }),
            definition({ dependencyValues }) {
                let tree;

                if (dependencyValues.symbol) {
                    tree = dependencyValues.symbol;

                    if (dependencyValues.charge === -1) {
                        tree = ["^", tree, "-"];
                    } else if (dependencyValues.charge === 1) {
                        tree = ["^", tree, "+"];
                    } else if (dependencyValues.charge < 0) {
                        tree = [
                            "^",
                            tree,
                            Math.abs(dependencyValues.charge) + "-",
                        ];
                    } else if (dependencyValues.charge > 0) {
                        tree = [
                            "^",
                            tree,
                            Math.abs(dependencyValues.charge) + "+",
                        ];
                    }
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
                charge: {
                    dependencyType: "stateVariable",
                    variableName: "charge",
                },
            }),
            definition({ dependencyValues }) {
                let latex;
                if (dependencyValues.symbol) {
                    latex = `\\text{${dependencyValues.symbol}}`;
                    if (dependencyValues.charge === -1) {
                        latex = latex + `^-`;
                    } else if (dependencyValues.charge === 1) {
                        latex = latex + `^+`;
                    } else if (dependencyValues.charge < 0) {
                        latex =
                            latex + `^{${Math.abs(dependencyValues.charge)}-}`;
                    } else if (dependencyValues.charge > 0) {
                        latex =
                            latex + `^{${Math.abs(dependencyValues.charge)}+}`;
                    }
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
                charge: {
                    dependencyType: "stateVariable",
                    variableName: "charge",
                },
            }),
            definition({ dependencyValues }) {
                let text;
                if (dependencyValues.symbol) {
                    text = dependencyValues.symbol;
                    if (dependencyValues.charge === -1) {
                        text = text + `^-`;
                    } else if (dependencyValues.charge === 1) {
                        text = text + `^+`;
                    } else if (dependencyValues.charge < 0) {
                        text =
                            text + `^(${Math.abs(dependencyValues.charge)}-)`;
                    } else if (dependencyValues.charge > 0) {
                        text =
                            text + `^(${Math.abs(dependencyValues.charge)}+)`;
                    }
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

    static adapters = ["math", "name"];
}
