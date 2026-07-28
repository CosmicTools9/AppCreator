// API bridge — re-exports @alioth/api as a relative-import alias.
// This lets ontology hooks stay inside the hooks package without
// requiring consumers to add @alioth/api as a direct dependency.
//
// The actual implementation is a re-export; consumers who need richer
// client config should import directly from @alioth/api.

export { apiClient, createApiClient, ApiClient } from "@alioth/api";
export type { ApiClientConfig } from "@alioth/api";
export type {
  ApiResponse,
  PaginatedData,
  ListQueryParams,
  PaginationParams,
} from "@alioth/types";
