import { useState, useCallback, useEffect, useRef } from "react";

interface ApiState<T> {
  data: T | null;
  loading: boolean;
  error: Error | null;
}

interface UseApiReturn<T> extends ApiState<T> {
  refetch: () => Promise<void>;
  setData: (data: T | null) => void;
  clearError: () => void;
}

export function useFetch<T>(
  fetchFn: () => Promise<T>,
  options: {
    immediate?: boolean;
    onSuccess?: (data: T) => void;
    onError?: (error: Error) => void;
  } = {}
): UseApiReturn<T> {
  const { immediate = true, onSuccess, onError } = options;
  const onSuccessRef = useRef(onSuccess);
  const onErrorRef = useRef(onError);

  useEffect(() => {
    onSuccessRef.current = onSuccess;
    onErrorRef.current = onError;
  }, [onSuccess, onError]);

  const [state, setState] = useState<ApiState<T>>({
    data: null,
    loading: false,
    error: null,
  });

  const execute = useCallback(async () => {
    setState((prev) => ({ ...prev, loading: true, error: null }));

    try {
      const data = await fetchFn();
      setState({ data, loading: false, error: null });
      onSuccessRef.current?.(data);
    } catch (error) {
      const err = error instanceof Error ? error : new Error("Unknown error");
      setState((prev) => ({ ...prev, loading: false, error: err }));
      onErrorRef.current?.(err);
    }
  }, [fetchFn]);

  useEffect(() => {
    if (immediate) {
      execute();
    }
  }, [immediate, execute]);

  return {
    ...state,
    refetch: execute,
    setData: (data: T | null) => setState((prev) => ({ ...prev, data })),
    clearError: () => setState((prev) => ({ ...prev, error: null })),
  };
}

export default useFetch;