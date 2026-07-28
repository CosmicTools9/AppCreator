/**
 * OpenAPI TypeScript Client Generator
 *
 * Generates type-safe API clients and React Query hooks from OpenAPI specifications.
 */

import type { ApiClient, ApiClientConfig } from "./client.js";
import { createApiClient } from "./client.js";

// ============================================
// OpenAPI Types
// ============================================

export interface OpenAPISpec {
  openapi: string;
  info: {
    title: string;
    version: string;
  };
  paths: Record<string, OpenAPIPathItem>;
  components?: {
    schemas?: Record<string, OpenAPISchema>;
  };
}

export interface OpenAPIPathItem {
  get?: OpenAPIOperation;
  post?: OpenAPIOperation;
  put?: OpenAPIOperation;
  patch?: OpenAPIOperation;
  delete?: OpenAPIOperation;
  parameters?: OpenAPIParameter[];
}

export interface OpenAPIOperation {
  operationId?: string;
  summary?: string;
  description?: string;
  parameters?: OpenAPIParameter[];
  requestBody?: OpenAPIRequestBody;
  responses?: Record<string, OpenAPIResponse>;
  tags?: string[];
}

export interface OpenAPIParameter {
  name: string;
  in: "query" | "path" | "header" | "cookie";
  required?: boolean;
  schema?: OpenAPISchema;
}

export interface OpenAPIRequestBody {
  content?: Record<string, { schema?: OpenAPISchema }>;
  required?: boolean;
}

export interface OpenAPIResponse {
  description?: string;
  content?: Record<string, { schema?: OpenAPISchema }>;
}

export interface OpenAPISchema {
  type?: string;
  format?: string;
  properties?: Record<string, OpenAPISchema>;
  items?: OpenAPISchema;
  required?: string[];
  enum?: string[];
  $ref?: string;
  nullable?: boolean;
  description?: string;
}

// ============================================
// Code Generation
// ============================================

export interface GeneratedClientConfig {
  /** Base URL for API */
  baseURL?: string;
  /** Client configuration */
  clientConfig?: ApiClientConfig;
}

/**
 * Generates a type-safe API client class from OpenAPI spec
 *
 * @example
 * ```typescript
 * const clientCode = generateClientFromOpenAPI(spec, {
 *   className: "UserApiClient",
 *   baseURL: "/api/v1"
 * });
 * ```
 */
export function generateClientFromOpenAPI(
  spec: OpenAPISpec,
  options: {
    className: string;
    baseURL?: string;
  }
): string {
  const { className, baseURL = "" } = options;

  const operations = extractOperations(spec);
  const types = generateTypes(spec);

  const methods = operations
    .map((op) => generateMethodSignature(op, baseURL))
    .join("\n\n  ");

  return `
// Generated from OpenAPI spec: ${spec.info.title} v${spec.info.version}
// DO NOT EDIT - This file is auto-generated

import { ApiClient, createApiClient, type ApiClientConfig } from "@alioth/api";

${types}

export class ${className} {
  private client: ApiClient;

  constructor(config?: ApiClientConfig) {
    this.client = createApiClient({
      baseURL: "${baseURL}",
      ...config,
    });
  }

  ${methods}

  // Access to raw client for advanced usage
  get raw(): ApiClient {
    return this.client;
  }
}

export function create${className}(config?: ApiClientConfig): ${className} {
  return new ${className}(config);
}
`;
}

interface ExtractedOperation {
  method: string;
  path: string;
  operationId: string;
  parameters: OpenAPIParameter[];
  hasRequestBody: boolean;
  responseType?: string;
  requestType?: string;
  summary?: string;
}

function extractOperations(spec: OpenAPISpec): ExtractedOperation[] {
  const operations: ExtractedOperation[] = [];

  for (const [path, pathItem] of Object.entries(spec.paths)) {
    for (const method of ["get", "post", "put", "patch", "delete"] as const) {
      const operation = pathItem[method];
      if (!operation) continue;

      const operationId =
        operation.operationId ||
        `${method}${path.replace(/[^a-zA-Z0-9]/g, "")}`;

      operations.push({
        method: method.toUpperCase(),
        path,
        operationId: sanitizeMethodName(operationId),
        parameters: [...(pathItem.parameters || []), ...(operation.parameters || [])],
        hasRequestBody: !!operation.requestBody,
        summary: operation.summary,
        responseType: extractResponseType(operation),
        requestType: extractRequestType(operation),
      });
    }
  }

  return operations;
}

function sanitizeMethodName(name: string): string {
  return name.replace(/[^a-zA-Z0-9_]/g, "");
}

function extractResponseType(operation: OpenAPIOperation): string | undefined {
  const successResponse = operation.responses?.["200"] || operation.responses?.["201"];
  if (!successResponse?.content?.["application/json"]?.schema) {
    return "void";
  }
  return schemaToType(successResponse.content["application/json"].schema);
}

function extractRequestType(operation: OpenAPIOperation): string | undefined {
  const schema = operation.requestBody?.content?.["application/json"]?.schema;
  return schema ? schemaToType(schema) : undefined;
}

function generateMethodSignature(
  op: ExtractedOperation,
  baseURL: string
): string {
  const pathParams = op.parameters.filter((p) => p.in === "path");
  const queryParams = op.parameters.filter((p) => p.in === "query");

  const pathParamsSignature = pathParams
    .map((p) => `${p.name}${p.required ? "" : "?"}: ${schemaToType(p.schema)}`)
    .join(", ");

  const queryParamsType = queryParams.length
    ? `params${queryParams.some((p) => !p.required) ? "?" : ""}: { ${queryParams
        .map((p) => `${p.name}${p.required ? "" : "?"}: ${schemaToType(p.schema)}`)
        .join("; ")} }`
    : "";

  const bodyParam = op.hasRequestBody
    ? `data${op.requestType?.includes("undefined") ? "?" : ""}: ${op.requestType || "unknown"}`
    : "";

  const params = [pathParamsSignature, queryParamsType, bodyParam]
    .filter(Boolean)
    .join(", ");

  // Build URL with path params
  let urlPath = op.path;
  for (const param of pathParams) {
    urlPath = urlPath.replace(`{${param.name}}`, `\${${param.name}}`);
  }

  const methodCall = op.method.toLowerCase();
  const bodyArg = op.hasRequestBody ? ", data" : "";
  const queryArg = queryParams.length ? ", { params }" : "";

  const jsDoc = op.summary
    ? `/**\n   * ${op.summary}\n   */\n  `
    : "";

  return `${jsDoc}async ${op.operationId}(${params}): Promise<${op.responseType || "void"}> {
    return this.client.${methodCall}(\`${urlPath}\`${bodyArg}${queryArg});
  }`;
}

function generateTypes(spec: OpenAPISpec): string {
  if (!spec.components?.schemas) return "";

  const types: string[] = [];

  for (const [name, schema] of Object.entries(spec.components.schemas)) {
    types.push(generateTypeDefinition(name, schema));
  }

  return types.join("\n\n");
}

function generateTypeDefinition(name: string, schema: OpenAPISchema): string {
  if (schema.enum) {
    return `export type ${name} = ${schema.enum.map((v) => `"${v}"`).join(" | ")};`;
  }

  if (schema.type === "object" && schema.properties) {
    const props = Object.entries(schema.properties)
      .map(([key, prop]) => {
        const required = schema.required?.includes(key);
        const type = schemaToType(prop);
        return `  ${key}${required ? "" : "?"}: ${type};`;
      })
      .join("\n");

    return `export interface ${name} {\n${props}\n}`;
  }

  return `export type ${name} = ${schemaToType(schema)};`;
}

function schemaToType(schema: OpenAPISchema | undefined): string {
  if (!schema) return "unknown";

  if (schema.$ref) {
    const parts = schema.$ref.split("/");
    return parts[parts.length - 1] || "unknown";
  }

  switch (schema.type) {
    case "string":
      if (schema.enum) {
        return schema.enum.map((v) => `"${v}"`).join(" | ");
      }
      if (schema.format === "date-time" || schema.format === "date") {
        return "string | Date";
      }
      return "string";
    case "integer":
    case "number":
      return "number";
    case "boolean":
      return "boolean";
    case "array":
      if (schema.items) {
        return `${schemaToType(schema.items)}[]`;
      }
      return "unknown[]";
    case "object":
      if (schema.properties) {
        const props = Object.entries(schema.properties)
          .map(([key, prop]) => {
            const required = schema.required?.includes(key);
            return `"${key}"${required ? "" : "?"}: ${schemaToType(prop)}`;
          })
          .join("; ");
        return `{ ${props} }`;
      }
      return "Record<string, unknown>";
    default:
      return "unknown";
  }
}

// ============================================
// React Query Hooks Generator
// ============================================

export interface ReactQueryHooksConfig {
  /** API client class name */
  clientClass: string;
  /** Base key for React Query */
  queryKeyBase: string;
}

/**
 * Generates React Query hooks from OpenAPI spec
 *
 * @example
 * ```typescript
 * const hooksCode = generateReactQueryHooks(spec, {
 *   clientClass: "UserApiClient",
 *   queryKeyBase: "users"
 * });
 * ```
 */
export function generateReactQueryHooks(
  spec: OpenAPISpec,
  config: ReactQueryHooksConfig
): string {
  const { clientClass, queryKeyBase } = config;

  const operations = extractOperations(spec);

  const queryHooks = operations
    .filter((op) => op.method === "GET" && !op.operationId.includes("delete"))
    .map((op) => generateQueryHook(op, clientClass, queryKeyBase))
    .join("\n\n");

  const mutationHooks = operations
    .filter((op) => ["POST", "PUT", "PATCH", "DELETE"].includes(op.method))
    .map((op) => generateMutationHook(op, clientClass, queryKeyBase))
    .join("\n\n");

  return `
// Generated React Query hooks from OpenAPI spec: ${spec.info.title}
// DO NOT EDIT - This file is auto-generated

import {
  useQuery,
  useMutation,
  useQueryClient,
  type UseQueryOptions,
  type UseMutationOptions,
  type QueryKey,
} from "@tanstack/react-query";
import { ${clientClass} } from "./client.js";

// Query Hooks
${queryHooks}

// Mutation Hooks
${mutationHooks}
`;
}

function generateQueryHook(
  op: ExtractedOperation,
  clientClass: string,
  queryKeyBase: string
): string {
  const hookName = `use${capitalize(op.operationId)}`;
  const queryKey = `["${queryKeyBase}", "${op.operationId}"]`;

  const pathParams = op.parameters.filter((p) => p.in === "path");
  const queryParams = op.parameters.filter((p) => p.in === "query");

  const paramsType = [
    ...pathParams.map((p) => `${p.name}: ${schemaToType(p.schema)}`),
    ...(queryParams.length ? [`params?: { ${queryParams.map((p) => `${p.name}?: ${schemaToType(p.schema)}`).join("; ")} }`] : []),
  ].join("; ");

  const clientMethod = `new ${clientClass}().${op.operationId}`;

  const argsList = [
    ...pathParams.map((p) => p.name),
    ...(queryParams.length ? ["params"] : []),
  ].join(", ");

  return `
export function ${hookName}(
  ${paramsType ? `{ ${paramsType} }: { ${paramsType} },` : ""}
  options?: Omit<UseQueryOptions<${op.responseType || "void"}, Error>, "queryKey" | "queryFn">
) {
  return useQuery({
    queryKey: ${queryKey}${pathParams.length || queryParams.length ? `.concat(${argsList})` : ""},
    queryFn: () => ${clientMethod}(${argsList}),
    ...options,
  });
}`;
}

function generateMutationHook(
  op: ExtractedOperation,
  clientClass: string,
  queryKeyBase: string
): string {
  const hookName = `use${capitalize(op.operationId)}`;
  const mutationKey = `["${queryKeyBase}", "${op.operationId}"]`;

  const pathParams = op.parameters.filter((p) => p.in === "path");
  const hasBody = op.hasRequestBody;

  const paramsType = [
    ...pathParams.map((p) => `${p.name}?: ${schemaToType(p.schema)}`),
    ...(hasBody ? [`data?: ${op.requestType || "unknown"}`] : []),
  ].join("; ");

  const clientMethod = `new ${clientClass}().${op.operationId}`;

  const argsList = [
    ...pathParams.map((p) => `variables.${p.name}`),
    ...(hasBody ? ["variables.data"] : []),
  ].filter(Boolean).join(", ");

  return `
export function ${hookName}(
  options?: Omit<UseMutationOptions<${op.responseType || "void"}, Error, { ${paramsType} }>, "mutationKey" | "mutationFn">
) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationKey: ${mutationKey},
    mutationFn: (variables: { ${paramsType} }) =>
      ${clientMethod}(${argsList}),
    onSuccess: () => {
      // Invalidate related queries
      queryClient.invalidateQueries({ queryKey: ["${queryKeyBase}"] });
    },
    ...options,
  });
}`;
}

function capitalize(str: string): string {
  return str.charAt(0).toUpperCase() + str.slice(1);
}
