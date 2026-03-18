import { useState, useEffect, useRef, useCallback } from "react";

type UsePollingResult<T> = {
  data: T | null;
  error: string | null;
  loading: boolean;
};

export function usePolling<T>(
  fetcher: () => Promise<T>,
  intervalMs: number
): UsePollingResult<T> {
  const [data, setData] = useState<T | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const fetcherRef = useRef(fetcher);
  fetcherRef.current = fetcher;

  const tick = useCallback(async () => {
    try {
      const result = await fetcherRef.current();
      setData(result);
      setError(null);
    } catch (err) {
      setError(
        err instanceof Error
          ? err.message
          : "The dashboard could not load backend data."
      );
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    tick();
    const id = window.setInterval(tick, intervalMs);
    return () => window.clearInterval(id);
  }, [tick, intervalMs]);

  return { data, error, loading };
}
