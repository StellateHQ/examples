import { TadaDocumentNode } from 'gql.tada';
import { GraphQLError, Kind, OperationTypeNode, print } from 'graphql';
import { meros } from 'meros/node';
import { revalidatePath } from 'next/cache';
import { cookies } from 'next/headers';
import http, { IncomingMessage } from 'node:http';
import setValue from 'set-value';
import { resolvable } from '../resolvable';
import { USER_COOKIE } from '../utils';
import { executeRequest } from './execution';

export function gql<Result, Variables>(
  query: TadaDocumentNode<Result, Variables>,
  variables: Variables
): Promise<ExecutionResult<Result>> {
  const userId = cookies().get(USER_COOKIE)?.value;
  const isMutation = query.definitions.some(
    (definition) =>
      definition.kind === Kind.OPERATION_DEFINITION &&
      definition.operation === OperationTypeNode.MUTATION
  );

  const result = resolvable<ExecutionResult<Result>>();

  const request = http.request(
    'http://localhost:8787/graphql',
    {
      method: 'POST',
      headers: {
        'content-type': 'application/json',
        ...(userId ? { 'x-user-id': userId } : {}),
        'x-stellate-e2e': 'true' // Avoid artificial delay
      }
    },
    async (response) => {
      const maybeIterable = await meros(response);
      if (maybeIterable instanceof IncomingMessage) {
        // TODO: handle stream error
        // TODO: handle error on JSON.parse
        let body = '';
        maybeIterable.setEncoding('utf8');
        maybeIterable.on('data', (chunk) => (body += chunk));
        maybeIterable.on('end', () => result.resolve(JSON.parse(body)));
        if (isMutation) revalidatePath('/', 'layout');
        return;
      }

      let isFirst = true;
      const partialResult: ExecutionResult = {};
      let asyncResult: Record<string, any> | undefined = undefined;
      for await (const chunk of maybeIterable) {
        const body = chunk.body as any;

        mergeIncrementalResult(partialResult, body);
        asyncResult = executeRequest(query, variables as any, partialResult.data, asyncResult);

        if (isFirst) {
          isFirst = false;
          result.resolve({ ...partialResult, data: asyncResult as any });
          if (isMutation) revalidatePath('/', 'layout');
        }
      }
    }
  );

  request.write(JSON.stringify({ query: print(query), variables }));
  request.end();

  return result;
}

type ExecutionResult<Result = unknown> = {
  data?: Result;
  errors?: ReadonlyArray<GraphQLError>;
};

type IncrementalResult = {
  data?: Record<string, unknown> | null;
  errors?: ReadonlyArray<GraphQLError>;
  extensions?: Record<string, unknown>;
  hasNext?: boolean;
  path?: ReadonlyArray<string | number>;
  incremental?: ReadonlyArray<IncrementalResult>;
  label?: string;
  items?: ReadonlyArray<Record<string, unknown>> | null;
};

function mergeIncrementalResult(
  executionResult: ExecutionResult,
  incrementalResult: IncrementalResult
): void {
  const path = ['data', ...(incrementalResult.path ?? [])];

  if (incrementalResult.items) {
    for (const item of incrementalResult.items) {
      setValue(executionResult, path.join('.'), item);
      // Increment the last path segment (the array index) to merge the next item at the next index
      (path[path.length - 1] as number) += 1;
    }
  }

  if (incrementalResult.data) {
    setValue(executionResult, path.join('.'), incrementalResult.data, {
      merge: true
    });
  }

  if (incrementalResult.errors) {
    executionResult.errors ||= [];
    (executionResult.errors as GraphQLError[]).push(...incrementalResult.errors);
  }

  if (incrementalResult.extensions) {
    setValue(executionResult, 'extensions', incrementalResult.extensions, {
      merge: true
    });
  }

  if (incrementalResult.incremental) {
    for (const incrementalSubResult of incrementalResult.incremental) {
      mergeIncrementalResult(executionResult, incrementalSubResult);
    }
  }
}
