import { ref } from "vue";
import { ElMessage } from "element-plus";

export function useErrorHandler() {
  const error = ref<string | null>(null);
  const isError = ref(false);

  /**
   * Wrap an async operation with standardized error handling.
   * Returns the result on success, or undefined on error.
   * Shows ElMessage.error toast automatically.
   */
  async function withErrorHandling<T>(
    operation: () => Promise<T>,
    options?: { silent?: boolean; toast?: boolean },
  ): Promise<T | undefined> {
    const { silent = false, toast = true } = options ?? {};
    try {
      error.value = null;
      isError.value = false;
      const result = await operation();
      return result;
    } catch (err) {
      const message = extractErrorMessage(err);
      error.value = message;
      isError.value = true;
      if (toast && !silent) {
        ElMessage.error(message);
      }
      return undefined;
    }
  }

  /**
   * Extract a human-readable error message from any error type.
   */
  function extractErrorMessage(err: unknown): string {
    if (err instanceof Error) return err.message;
    if (typeof err === "string") return err;
    if (err && typeof err === "object" && "message" in err) {
      return String((err as { message: unknown }).message);
    }
    return String(err);
  }

  return { withErrorHandling };
}
