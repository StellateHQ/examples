import {
  ConstValueNode,
  DocumentNode,
  FieldNode,
  FragmentDefinitionNode,
  GraphQLAbstractType,
  GraphQLBoolean,
  GraphQLFloat,
  GraphQLIncludeDirective,
  GraphQLInputType,
  GraphQLInt,
  GraphQLLeafType,
  GraphQLObjectType,
  GraphQLOutputType,
  GraphQLSkipDirective,
  GraphQLString,
  Kind,
  OperationDefinitionNode,
  OperationTypeNode,
  SelectionNode,
  SelectionSetNode,
  TypeNode,
  buildClientSchema,
  isEnumType,
  isInputObjectType,
  isInputType,
  isInterfaceType,
  isLeafType,
  isListType,
  isNonNullType,
  isObjectType,
  isScalarType,
  isUnionType
} from 'graphql';
import introspection from '../../introspection.json';
import { Resolvable, resolvable } from 'lib/resolvable';

const SCHEMA = buildClientSchema(introspection as any);

export function executeRequest(
  document: DocumentNode,
  variableValues: Record<string, unknown> | undefined,
  initialValue: unknown,
  previousResult?: Record<string, any>
): Record<string, any> {
  let operation: OperationDefinitionNode | undefined = undefined;
  const fragments: Record<string, FragmentDefinitionNode> = {};

  for (const definition of document.definitions) {
    switch (definition.kind) {
      case Kind.OPERATION_DEFINITION: {
        if (operation) {
          throw new Error('Document must contain exactly one operation');
        } else {
          operation = definition;
        }
        break;
      }
      case Kind.FRAGMENT_DEFINITION: {
        fragments[definition.name.value] = definition;
        break;
      }
    }
  }

  if (!operation) {
    throw new Error('Document must contain exactly one operation');
  }

  const coercedVariableValues = coerceVariableValues(operation, variableValues);

  const rootType = (() => {
    switch (operation.operation) {
      case OperationTypeNode.QUERY:
        return SCHEMA.getQueryType();
      case OperationTypeNode.MUTATION:
        return SCHEMA.getMutationType();
      case OperationTypeNode.SUBSCRIPTION:
        throw new Error('Subscriptions are not supported');
    }
  })();

  if (!isObjectType(rootType)) {
    throw new Error('Missing root operation type');
  }

  const selectionSet = operation.selectionSet;

  return executeSelectionSet(
    fragments,
    selectionSet,
    rootType,
    initialValue,
    previousResult,
    coercedVariableValues
  );
}

function coerceVariableValues(
  operation: OperationDefinitionNode,
  variableValues: Record<string, unknown> = {}
): Record<string, any> {
  const coercedValues: Record<string, any> = {};

  for (const variableDefinition of operation.variableDefinitions ?? []) {
    const variableName = variableDefinition.variable.name.value;
    const variableType = variableDefinition.type;
    const defaultValue = variableDefinition.defaultValue;

    const hasValue = variableName in variableValues;
    const value = variableValues[variableName];

    if (!hasValue && defaultValue) {
      coercedValues[variableName] = constValueNodeToJs(defaultValue);
    } else if (variableType.kind === Kind.NON_NULL_TYPE && (!hasValue || value === null)) {
      throw new Error(`Variable $${variableName} must not be null`);
    } else if (hasValue) {
      if (value === null) {
        coercedValues[variableName] = null;
      } else {
        coercedValues[variableName] = inputCoersion(variableType, value);
      }
    }
  }

  return coercedValues;
}

function constValueNodeToJs(value: ConstValueNode): any {
  switch (value.kind) {
    case Kind.INT:
      return GraphQLInt.parseLiteral(value);
    case Kind.FLOAT:
      return GraphQLFloat.parseLiteral(value);
    case Kind.STRING:
      return GraphQLString.parseLiteral(value);
    case Kind.BOOLEAN:
      return GraphQLBoolean.parseLiteral(value);
    case Kind.NULL:
      return null;
    case Kind.ENUM:
      return value.value;
    case Kind.LIST:
      return value.values.map(constValueNodeToJs);
    case Kind.OBJECT:
      const obj: any = {};
      for (const field of value.fields) obj[field.name.value] = constValueNodeToJs(field.value);
      return obj;
  }
}

function inputCoersion(variableType: TypeNode, value: any): any {
  switch (variableType.kind) {
    case Kind.NON_NULL_TYPE: {
      return inputCoersion(variableType.type, value);
    }
    case Kind.LIST_TYPE: {
      return;
    }
    case Kind.NAMED_TYPE: {
      const schemaType = SCHEMA.getType(variableType.name.value);
      if (!schemaType) throw new Error(`Unknown type ${variableType.name.value}`);
      if (!isInputType(schemaType)) throw new Error(`${schemaType.name} is not an input type`);
      return coerceSchemaType(schemaType, value);
    }
  }
}

function coerceSchemaType(schemaType: GraphQLInputType, value: any): any {
  if (isNonNullType(schemaType)) {
    if (value === null) {
      throw new Error(`Found null for non-nullable type`);
    }
    return coerceSchemaType(schemaType.ofType, value);
  }

  if (isListType(schemaType)) {
    const valueList = Array.isArray(value) ? value : [value];
    return valueList.map((value) => coerceSchemaType(schemaType.ofType, value));
  }

  if (isScalarType(schemaType)) {
    return schemaType.parseValue(value);
  }

  if (isEnumType(schemaType)) {
    return schemaType.parseValue(value);
  }

  if (isInputObjectType(schemaType)) {
    const obj: any = {};

    const fields = schemaType.getFields();
    for (const fieldName in fields) {
      const field = fields[fieldName]!;
      const fieldValue =
        typeof value === 'object' && value !== null && fieldName in value ? value[fieldName] : null;

      obj[fieldName] = coerceSchemaType(field.type, fieldValue);
    }

    return obj;
  }

  throw new Error('Unexpected schema type');
}

function executeSelectionSet(
  fragments: Record<string, FragmentDefinitionNode>,
  selectionSet: SelectionSetNode,
  objectType: GraphQLObjectType,
  objectValue: any,
  previousValue: any,
  variableValues: Record<string, any>
): Record<string, any> {
  const groupedFieldSet = collectFields(fragments, objectType, selectionSet, variableValues);

  const resultMap: Record<string, any> = {};

  for (const responseKey in groupedFieldSet) {
    const fields = groupedFieldSet[responseKey]!;

    const fieldName = fields[0]!.name.value;
    const fieldType = objectType.getFields()[fieldName]?.type;

    if (fieldType) {
      resultMap[responseKey] = completeValue(
        fragments,
        fieldType,
        fields,
        objectValue[responseKey],
        previousValue?.[responseKey],
        variableValues
      );
    }
  }

  return resultMap;
}

function collectFields(
  fragments: Record<string, FragmentDefinitionNode>,
  objectType: GraphQLObjectType,
  selectionSet: SelectionSetNode,
  variableValues: Record<string, any>,
  visitedFragments: Set<string> = new Set()
): Record<string, FieldNode[]> {
  const groupedFields: Record<string, FieldNode[]> = {};

  for (const selection of selectionSet.selections) {
    const skipDirective = selection.directives?.find(
      (directive) => directive.name.value === GraphQLSkipDirective.name
    );
    if (skipDirective) {
      const ifArg = skipDirective.arguments?.find((argument) => argument.name.value === 'if');
      if (!ifArg) throw new Error('Missing `if` argument for directive `@skip`');
      if (ifArg.value.kind === Kind.BOOLEAN && ifArg.value.value === true) continue;
      if (ifArg.value.kind === Kind.VARIABLE && variableValues[ifArg.value.name.value] === true)
        continue;
    }

    const includeDirective = selection.directives?.find(
      (directive) => directive.name.value === GraphQLIncludeDirective.name
    );
    if (includeDirective) {
      const ifArg = includeDirective.arguments?.find((argument) => argument.name.value === 'if');
      if (!ifArg) throw new Error('Missing `if` argument for directive `@include`');
      if (ifArg.value.kind === Kind.BOOLEAN && ifArg.value.value !== true) continue;
      if (ifArg.value.kind === Kind.VARIABLE && variableValues[ifArg.value.name.value] !== true)
        continue;
    }

    switch (selection.kind) {
      case Kind.FIELD: {
        const responseKey = selection.alias?.value ?? selection.name.value;
        if (responseKey in groupedFields) {
          groupedFields[responseKey]!.push(selection);
        } else {
          groupedFields[responseKey] = [selection];
        }
        break;
      }
      case Kind.FRAGMENT_SPREAD: {
        const fragmentSpreadName = selection.name.value;

        if (visitedFragments.has(fragmentSpreadName)) continue;
        visitedFragments.add(fragmentSpreadName);

        const fragment = fragments[fragmentSpreadName];
        if (!fragment) continue;

        const fragmentType = fragment.typeCondition.name.value;
        if (!doesFragmentTypeApply(objectType, fragmentType)) continue;

        const fragmentSelectionSet = fragment.selectionSet;
        const fragmentGroupedFieldSet = collectFields(
          fragments,
          objectType,
          fragmentSelectionSet,
          variableValues,
          visitedFragments
        );

        for (const responseKey in fragmentGroupedFieldSet) {
          const fragmentGroup = fragmentGroupedFieldSet[responseKey]!;
          if (responseKey in groupedFields) {
            groupedFields[responseKey]!.push(...fragmentGroup);
          } else {
            groupedFields[responseKey] = fragmentGroup;
          }
        }
        break;
      }
      case Kind.INLINE_FRAGMENT: {
        const fragmentType = selection.typeCondition?.name.value;
        if (fragmentType && !doesFragmentTypeApply(objectType, fragmentType)) continue;

        const fragmentSelectionSet = selection.selectionSet;
        const fragmentGroupedFieldSet = collectFields(
          fragments,
          objectType,
          fragmentSelectionSet,
          variableValues,
          visitedFragments
        );

        for (const responseKey in fragmentGroupedFieldSet) {
          const fragmentGroup = fragmentGroupedFieldSet[responseKey]!;
          if (responseKey in groupedFields) {
            groupedFields[responseKey]!.push(...fragmentGroup);
          } else {
            groupedFields[responseKey] = fragmentGroup;
          }
        }
        break;
      }
    }
  }

  return groupedFields;
}

function doesFragmentTypeApply(objectType: GraphQLObjectType, fragmentType: string): boolean {
  const fragmentSchemaType = SCHEMA.getType(fragmentType);
  if (!fragmentSchemaType) throw new Error(`Unknown type ${fragmentType}`);

  if (isObjectType(fragmentSchemaType)) {
    return objectType.name === fragmentSchemaType.name;
  } else if (isInterfaceType(fragmentSchemaType)) {
    return objectType
      .getInterfaces()
      .some((interfaceType) => interfaceType.name === fragmentSchemaType.name);
  } else if (isUnionType(fragmentSchemaType)) {
    return fragmentSchemaType.getTypes().some((memberType) => memberType.name === objectType.name);
  } else {
    throw new Error(`Fragment type ${fragmentType} must be object, interface, or union`);
  }
}

function completeValue(
  fragments: Record<string, FragmentDefinitionNode>,
  fieldType: GraphQLOutputType,
  fields: FieldNode[],
  result: any,
  previous: any,
  variableValues: Record<string, any>
): any {
  if (result === undefined) return previous instanceof Promise ? previous : resolvable();

  if (isNonNullType(fieldType)) {
    const innerType = fieldType.ofType;
    const completedResult = completeValue(
      fragments,
      innerType,
      fields,
      result,
      previous,
      variableValues
    );
    if (completedResult === null)
      throw new Error(`Field ${fields[0]!.name.value} cannot return null`);
    return completedResult;
  }

  if (result === null) {
    if (previous instanceof Promise) {
      (previous as Resolvable).resolve(null);
    }
    return null;
  }

  if (isListType(fieldType)) {
    if (!Array.isArray(result))
      throw new Error(`Field ${fields[0]!.name.value} must return a list of values`);
    const innerType = fieldType.ofType;
    const resultList = result.map((resultItem, i) =>
      completeValue(fragments, innerType, fields, resultItem, previous?.[i], variableValues)
    );
    if (previous instanceof Promise) {
      (previous as Resolvable).resolve(resultList);
    }
    return resultList;
  }

  if (isLeafType(fieldType)) {
    const coercedResult = coerceResult(fieldType, result);
    if (previous instanceof Promise) {
      (previous as Resolvable).resolve(coercedResult);
    }
    return coercedResult;
  }

  const objectType = isObjectType(fieldType) ? fieldType : resolveAbstractType(fieldType, result);
  const subSelectionSet = mergeSelectionSets(fields);
  const fieldResult = executeSelectionSet(
    fragments,
    subSelectionSet,
    objectType,
    result,
    previous,
    variableValues
  );
  if (previous instanceof Promise) {
    (previous as Resolvable).resolve(fieldResult);
  }
  return fieldResult;
}

function coerceResult(leafType: GraphQLLeafType, value: any): any {
  // TODO: logic for missing leaf when using defer
  if (value === null) throw new Error(`Cannot return null for type ${leafType.name}`);
  return leafType.parseValue(value);
}

function mergeSelectionSets(fields: FieldNode[]): SelectionSetNode {
  const selections: SelectionNode[] = [];
  for (const field of fields) {
    const fieldSelectionSet = field.selectionSet?.selections;
    if (!fieldSelectionSet) continue;
    selections.push(...fieldSelectionSet);
  }
  return { kind: Kind.SELECTION_SET, selections };
}

function resolveAbstractType(abstractType: GraphQLAbstractType, result: any): GraphQLObjectType {
  if ('__typename' in result) {
    const type = SCHEMA.getType(result.__typename);
    if (!type) throw new Error(`Unknown type ${result}`);
    if (!isObjectType(type)) throw new Error(`Type ${result} is not an object type`);

    if (
      isInterfaceType(abstractType) &&
      type.getInterfaces().every((interfaceType) => interfaceType.name !== abstractType.name)
    )
      throw new Error(`Object type ${type.name} does not implement interface ${abstractType.name}`);

    if (
      isUnionType(abstractType) &&
      abstractType.getTypes().every((memberType) => memberType.name !== type.name)
    )
      throw new Error(
        `Object type ${type.name} is not a member of union type ${abstractType.name}`
      );

    return type;
  }

  throw new Error(
    `Cannot resolve abstract type ${abstractType.name}, make sure to include __typename fields`
  );
}
